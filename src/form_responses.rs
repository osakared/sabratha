use async_graphql::*;
use turbosql::{Turbosql, select};
use uuid::Uuid;

use crate::forms::{FormField, FormFieldRow, FormRow};

// ---------------------------------------------------------------------------
// Domain types (serde only — stored as JSON inside FormResponseRow)
// ---------------------------------------------------------------------------

/// The value of a single field response, represented as a proper Rust enum
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum FormFieldValue {
    Text(String),
    Number(f64),
    Bool(bool),
    SelectedOptions(Vec<String>),
}

/// A single field's response data, serialized as part of the JSON blob in `FormResponseRow`
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FormFieldResponseData {
    pub id: String,
    pub form_field_id: String,
    pub value: FormFieldValue,
}

impl FormFieldResponseData {
    /// Creates a new `FormFieldResponseData` from a `FormFieldResponseInput`
    pub fn from_input(input: &FormFieldResponseInput) -> Self {
        let value = match &input.value {
            FormFieldValueInput::Text(s) => FormFieldValue::Text(s.clone()),
            FormFieldValueInput::Number(n) => FormFieldValue::Number(*n),
            FormFieldValueInput::Bool(b) => FormFieldValue::Bool(*b),
            FormFieldValueInput::SelectedOptions(opts) => {
                FormFieldValue::SelectedOptions(opts.values.clone())
            }
        };

        Self {
            id: input.id.clone(),
            form_field_id: input.form_field_id.clone(),
            value,
        }
    }
}

// ---------------------------------------------------------------------------
// Database-only struct (Turbosql)
// ---------------------------------------------------------------------------

/// Row backing a form response (SQLite storage only).
/// Field response data is stored as a JSON string in `field_responses_json`.
#[derive(Turbosql, Clone)]
pub struct FormResponseRow {
    pub rowid: Option<i64>,
    pub id: String,
    pub form_id: String,
    /// JSON-serialized `Vec<FormFieldResponseData>`
    pub field_responses_json: String,
}

impl Default for FormResponseRow {
    fn default() -> Self {
        Self {
            rowid: None,
            id: Uuid::new_v4().to_string(),
            form_id: String::default(),
            field_responses_json: "[]".to_string(),
        }
    }
}

impl FormResponseRow {
    /// Creates a `FormResponseRow` for the given `form_id`, inserts into db and returns result or error
    pub fn create(form_id: &String) -> Result<Self, turbosql::Error> {
        let mut row = Self::default();
        row.form_id = form_id.clone();
        match row.insert() {
            Ok(id) => select!(FormResponseRow "WHERE rowid = " id),
            Err(e) => Err(e),
        }
    }

    /// Tries to find a single `FormResponseRow` with given id
    pub fn find(id: &String) -> Result<Self, turbosql::Error> {
        select!(FormResponseRow "WHERE id = " id)
    }

    /// Gets all `FormResponseRow`s
    pub fn query() -> Vec<Self> {
        select!(Vec<FormResponseRow>).unwrap_or(vec![])
    }

    /// Gets all `FormResponseRow`s for a given `Form`
    pub fn query_by_form(form_id: &String) -> Vec<Self> {
        select!(Vec<FormResponseRow> "WHERE form_id = ?", form_id).unwrap_or(vec![])
    }

    /// Deserializes the field responses from the JSON column
    pub fn field_responses(&self) -> Vec<FormFieldResponseData> {
        serde_json::from_str(&self.field_responses_json).unwrap_or_default()
    }

    /// Replaces the field responses and re-serializes to the JSON column
    pub fn set_field_responses(&mut self, data: &[FormFieldResponseData]) {
        self.field_responses_json =
            serde_json::to_string(data).unwrap_or_else(|_| "[]".to_string());
    }

    /// Upserts a single field response entry (by `FormFieldResponseData.id`) into the JSON blob,
    /// persists the row, and returns the updated `FormFieldResponseData`
    pub fn upsert_field_response(
        &mut self,
        data: FormFieldResponseData,
    ) -> Result<FormFieldResponseData, turbosql::Error> {
        let mut entries = self.field_responses();
        if let Some(existing) = entries.iter_mut().find(|e| e.id == data.id) {
            *existing = data.clone();
        } else {
            entries.push(data.clone());
        }
        self.set_field_responses(&entries);
        self.update()?;
        Ok(data)
    }
}

// ---------------------------------------------------------------------------
// GraphQL input types
// ---------------------------------------------------------------------------

/// A list of selected option values, used as the `SelectedOptions` variant in `FormFieldValueInput`
#[derive(InputObject, Clone)]
pub struct SelectedOptionsInput {
    pub values: Vec<String>,
}

/// OneOf input representing the value of a form field response.
/// Clients must provide exactly one variant.
#[derive(OneofObject, Clone)]
pub enum FormFieldValueInput {
    Text(String),
    Number(f64),
    Bool(bool),
    SelectedOptions(SelectedOptionsInput),
}

/// Input type for creating/updating a single field response value
#[derive(InputObject, Clone)]
pub struct FormFieldResponseInput {
    pub id: String,
    pub form_field_id: String,
    pub value: FormFieldValueInput,
}

// ---------------------------------------------------------------------------
// GraphQL output types
// ---------------------------------------------------------------------------

/// GraphQL representation of a single field value within a form response.
/// Wraps a `FormFieldResponseData` and resolves fields from it.
pub struct FormFieldResponse {
    pub data: FormFieldResponseData,
}

impl FormFieldResponse {
    pub fn from_data(data: FormFieldResponseData) -> Self {
        Self { data }
    }
}

#[Object]
impl FormFieldResponse {
    async fn id(&self) -> &str {
        &self.data.id
    }

    async fn form_field_id(&self) -> &str {
        &self.data.form_field_id
    }

    async fn text_value(&self) -> Option<&str> {
        match &self.data.value {
            FormFieldValue::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    async fn number_value(&self) -> Option<f64> {
        match &self.data.value {
            FormFieldValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    async fn bool_value(&self) -> Option<bool> {
        match &self.data.value {
            FormFieldValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    async fn selected_options(&self) -> &[String] {
        match &self.data.value {
            FormFieldValue::SelectedOptions(opts) => opts.as_slice(),
            _ => &[],
        }
    }

    /// Gets the `FormField` this response value corresponds to
    async fn form_field(&self) -> Option<FormField> {
        FormFieldRow::find(&self.data.form_field_id)
            .ok()
            .map(|row| FormField::from_row(&row))
    }
}

/// GraphQL representation of a form response (submission).
/// Holds only the UUID; scalar fields are resolved from the database on demand.
pub struct FormResponse {
    pub id: String,
}

impl FormResponse {
    pub fn from_row(row: &FormResponseRow) -> Self {
        Self { id: row.id.clone() }
    }
}

#[Object]
impl FormResponse {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn form_id(&self) -> Result<String, async_graphql::Error> {
        let row = FormResponseRow::find(&self.id)
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row.form_id)
    }

    /// Gets the `Form` this response belongs to
    async fn form(&self) -> Option<crate::forms::Form> {
        let response_row = FormResponseRow::find(&self.id).ok()?;
        FormRow::find(&response_row.form_id)
            .ok()
            .map(|row| crate::forms::Form::from_row(&row))
    }

    /// Gets child `FormFieldResponse`s
    async fn field_responses(&self) -> Result<Vec<FormFieldResponse>, async_graphql::Error> {
        let row = FormResponseRow::find(&self.id)
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row
            .field_responses()
            .into_iter()
            .map(FormFieldResponse::from_data)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::forms::{FormFieldRow, FormRow};

    use super::{
        FormFieldResponseData, FormFieldResponseInput, FormFieldValue, FormFieldValueInput,
        FormResponseRow, SelectedOptionsInput,
    };

    #[test]
    fn create_form_response() {
        let form = FormRow::create().unwrap();
        let response = FormResponseRow::create(&form.id).unwrap();
        assert_eq!(response.form_id, form.id);
        assert!(!response.id.is_empty());
        assert_eq!(response.field_responses().len(), 0);
    }

    #[test]
    fn form_response_links_to_form() {
        let form = FormRow::create().unwrap();
        let responses = FormResponseRow::query_by_form(&form.id);
        assert_eq!(responses.len(), 0);

        let response = FormResponseRow::create(&form.id).unwrap();
        let responses = FormResponseRow::query_by_form(&form.id);
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].id, response.id);
    }

    #[test]
    fn text_field_response() {
        let form = FormRow::create().unwrap();
        let form_field = FormFieldRow::create(&form.id).unwrap();
        let mut response = FormResponseRow::create(&form.id).unwrap();

        let input = FormFieldResponseInput {
            id: uuid::Uuid::new_v4().to_string(),
            form_field_id: form_field.id.clone(),
            value: FormFieldValueInput::Text("hello".to_string()),
        };

        let data = FormFieldResponseData::from_input(&input);
        let updated = response.upsert_field_response(data).unwrap();
        assert_eq!(updated.value, FormFieldValue::Text("hello".to_string()));

        // Verify it round-trips through the DB
        let reloaded = FormResponseRow::find(&response.id).unwrap();
        let entries = reloaded.field_responses();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, FormFieldValue::Text("hello".to_string()));
    }

    #[test]
    fn number_field_response() {
        let form = FormRow::create().unwrap();
        let form_field = FormFieldRow::create(&form.id).unwrap();
        let mut response = FormResponseRow::create(&form.id).unwrap();

        let input = FormFieldResponseInput {
            id: uuid::Uuid::new_v4().to_string(),
            form_field_id: form_field.id.clone(),
            value: FormFieldValueInput::Number(42.5),
        };

        let data = FormFieldResponseData::from_input(&input);
        let updated = response.upsert_field_response(data).unwrap();
        assert_eq!(updated.value, FormFieldValue::Number(42.5));
    }

    #[test]
    fn bool_field_response() {
        let form = FormRow::create().unwrap();
        let form_field = FormFieldRow::create(&form.id).unwrap();
        let mut response = FormResponseRow::create(&form.id).unwrap();

        let input = FormFieldResponseInput {
            id: uuid::Uuid::new_v4().to_string(),
            form_field_id: form_field.id.clone(),
            value: FormFieldValueInput::Bool(true),
        };

        let data = FormFieldResponseData::from_input(&input);
        let updated = response.upsert_field_response(data).unwrap();
        assert_eq!(updated.value, FormFieldValue::Bool(true));
    }

    #[test]
    fn selected_options_field_response() {
        let form = FormRow::create().unwrap();
        let form_field = FormFieldRow::create(&form.id).unwrap();
        let mut response = FormResponseRow::create(&form.id).unwrap();

        let input = FormFieldResponseInput {
            id: uuid::Uuid::new_v4().to_string(),
            form_field_id: form_field.id.clone(),
            value: FormFieldValueInput::SelectedOptions(SelectedOptionsInput {
                values: vec!["a".to_string()],
            }),
        };

        let data = FormFieldResponseData::from_input(&input);
        let updated = response.upsert_field_response(data).unwrap();
        assert_eq!(
            updated.value,
            FormFieldValue::SelectedOptions(vec!["a".to_string()])
        );
    }

    #[test]
    fn form_response_multiple_field_responses() {
        let form = FormRow::create().unwrap();
        let field1 = FormFieldRow::create(&form.id).unwrap();
        let field2 = FormFieldRow::create(&form.id).unwrap();
        let mut response = FormResponseRow::create(&form.id).unwrap();

        let input1 = FormFieldResponseInput {
            id: uuid::Uuid::new_v4().to_string(),
            form_field_id: field1.id.clone(),
            value: FormFieldValueInput::Text("hello".to_string()),
        };
        let input2 = FormFieldResponseInput {
            id: uuid::Uuid::new_v4().to_string(),
            form_field_id: field2.id.clone(),
            value: FormFieldValueInput::Number(99.0),
        };

        let data1 = FormFieldResponseData::from_input(&input1);
        let data2 = FormFieldResponseData::from_input(&input2);
        response.upsert_field_response(data1).unwrap();
        response.upsert_field_response(data2).unwrap();

        let reloaded = FormResponseRow::find(&response.id).unwrap();
        assert_eq!(reloaded.field_responses().len(), 2);
    }

    #[test]
    fn upsert_updates_existing_field_response() {
        let form = FormRow::create().unwrap();
        let form_field = FormFieldRow::create(&form.id).unwrap();
        let mut response = FormResponseRow::create(&form.id).unwrap();
        let field_response_id = uuid::Uuid::new_v4().to_string();

        // Insert initial value
        let input = FormFieldResponseInput {
            id: field_response_id.clone(),
            form_field_id: form_field.id.clone(),
            value: FormFieldValueInput::Text("first".to_string()),
        };
        let data = FormFieldResponseData::from_input(&input);
        response.upsert_field_response(data).unwrap();

        // Update same entry
        let input2 = FormFieldResponseInput {
            id: field_response_id.clone(),
            form_field_id: form_field.id.clone(),
            value: FormFieldValueInput::Text("second".to_string()),
        };
        let data2 = FormFieldResponseData::from_input(&input2);
        response.upsert_field_response(data2).unwrap();

        let reloaded = FormResponseRow::find(&response.id).unwrap();
        let entries = reloaded.field_responses();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, FormFieldValue::Text("second".to_string()));
    }
}
