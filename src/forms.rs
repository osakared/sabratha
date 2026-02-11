use async_graphql::*;
use turbosql::{Turbosql, select};
use uuid::Uuid;

use crate::form_responses::{FormResponse, FormResponseRow};

/// Represents the type of a `FormField` which determines the underlying data type in saved `FormResponse`s
#[derive(serde::Serialize, serde::Deserialize, Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum FormFieldType {
    Text,
    Number,
    CheckboxBoolean,
    CheckboxMultiple,
    Radio,
    Select,
    Signature,
    ImageFile,
}

impl Default for FormFieldType {
    fn default() -> Self {
        Self::Text
    }
}

// ---------------------------------------------------------------------------
// Database-only structs (Turbosql)
// ---------------------------------------------------------------------------

/// Row backing a form field (SQLite storage only)
#[derive(Turbosql, Clone)]
pub struct FormFieldRow {
    pub rowid: Option<i64>,
    pub id: String,
    pub form_id: String,
    pub label: String,
    pub help: String,
    pub input_type: FormFieldType,
    pub option_values: Vec<String>,
}

impl Default for FormFieldRow {
    fn default() -> Self {
        Self {
            rowid: None,
            id: Uuid::new_v4().to_string(),
            form_id: String::default(),
            label: String::default(),
            help: String::default(),
            input_type: FormFieldType::default(),
            option_values: vec![],
        }
    }
}

impl FormFieldRow {
    /// Creates a `FormFieldRow` with given parent form, inserts into db and returns result or error
    pub fn create(form_id: &String) -> Result<Self, turbosql::Error> {
        let mut row = Self::default();
        row.form_id = form_id.clone();
        match row.insert() {
            Ok(id) => select!(FormFieldRow "WHERE rowid = " id),
            Err(e) => Err(e),
        }
    }

    /// Tries to find a single `FormFieldRow` with given id
    pub fn find(id: &String) -> Result<Self, turbosql::Error> {
        select!(FormFieldRow "WHERE id = " id)
    }

    /// Creates or updates a `FormFieldRow` from the given input
    pub fn create_or_update(
        form_id: &String,
        input: &FormFieldInput,
    ) -> Result<Self, turbosql::Error> {
        let mut row = match Self::find(&input.id) {
            Ok(mut r) => {
                r.form_id = form_id.clone();
                r
            }
            Err(_) => Self::create(form_id)?,
        };

        row.label = input.label.clone();
        row.help = input.help.clone();
        row.input_type = input.input_type;
        row.option_values = input.option_values.clone();

        match row.update() {
            Ok(_) => Ok(row),
            Err(e) => Err(e),
        }
    }
}

/// Row backing a form (SQLite storage only)
#[derive(Turbosql, Clone)]
pub struct FormRow {
    pub rowid: Option<i64>,
    pub id: String,
    pub title: String,
}

impl Default for FormRow {
    fn default() -> Self {
        Self {
            rowid: None,
            id: Uuid::new_v4().to_string(),
            title: String::default(),
        }
    }
}

impl FormRow {
    /// Creates a `FormRow`, inserts into db and returns result or error
    pub fn create() -> Result<Self, turbosql::Error> {
        let row = Self::default();
        match row.insert() {
            Ok(id) => select!(FormRow "WHERE rowid = " id),
            Err(e) => Err(e),
        }
    }

    /// Gets child `FormFieldRow`s
    pub fn form_fields(&self) -> Vec<FormFieldRow> {
        select!(Vec<FormFieldRow> "WHERE form_id = ?", self.id).unwrap_or(vec![])
    }

    /// Gets all `FormRow`s
    pub fn query() -> Vec<Self> {
        select!(Vec<FormRow>).unwrap_or(vec![])
    }

    /// Tries to find a single `FormRow` with given id
    pub fn find(id: &String) -> Result<Self, turbosql::Error> {
        select!(FormRow "WHERE id = " id)
    }

    /// Finds a `FormRow` by id or creates a new one, then updates from the given input
    pub fn create_or_update(input: &FormInput) -> Result<Self, turbosql::Error> {
        let mut row = match Self::find(&input.id) {
            Ok(r) => r,
            Err(_) => Self::create()?,
        };

        row.title = input.title.clone();

        for field_input in &input.fields {
            FormFieldRow::create_or_update(&row.id, field_input)?;
        }

        match row.update() {
            Ok(_) => Ok(row),
            Err(e) => Err(e),
        }
    }
}

// ---------------------------------------------------------------------------
// GraphQL input types
// ---------------------------------------------------------------------------

/// Input type for creating/updating a form field
#[derive(InputObject, Clone)]
pub struct FormFieldInput {
    pub id: String,
    pub label: String,
    pub help: String,
    pub input_type: FormFieldType,
    pub option_values: Vec<String>,
}

/// Input type for creating/updating a form
#[derive(InputObject, Clone)]
pub struct FormInput {
    pub id: String,
    pub title: String,
    pub fields: Vec<FormFieldInput>,
}

// ---------------------------------------------------------------------------
// GraphQL output types (resolve fields by fetching from DB)
// ---------------------------------------------------------------------------

/// GraphQL representation of a form field.
/// Holds only the UUID; all fields are resolved from the database on demand.
pub struct FormField {
    pub id: String,
}

impl FormField {
    pub fn from_row(row: &FormFieldRow) -> Self {
        Self { id: row.id.clone() }
    }
}

#[Object]
impl FormField {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn form_id(&self) -> Result<String, async_graphql::Error> {
        let row =
            FormFieldRow::find(&self.id).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row.form_id)
    }

    async fn label(&self) -> Result<String, async_graphql::Error> {
        let row =
            FormFieldRow::find(&self.id).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row.label)
    }

    async fn help(&self) -> Result<String, async_graphql::Error> {
        let row =
            FormFieldRow::find(&self.id).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row.help)
    }

    async fn input_type(&self) -> Result<FormFieldType, async_graphql::Error> {
        let row =
            FormFieldRow::find(&self.id).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row.input_type)
    }

    async fn option_values(&self) -> Result<Vec<String>, async_graphql::Error> {
        let row =
            FormFieldRow::find(&self.id).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row.option_values)
    }
}

/// GraphQL representation of a form.
/// Holds only the UUID; all fields are resolved from the database on demand.
pub struct Form {
    pub id: String,
}

impl Form {
    pub fn from_row(row: &FormRow) -> Self {
        Self { id: row.id.clone() }
    }
}

#[Object]
impl Form {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn title(&self) -> Result<String, async_graphql::Error> {
        let row = FormRow::find(&self.id).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row.title)
    }

    async fn fields(&self) -> Result<Vec<FormField>, async_graphql::Error> {
        let row = FormRow::find(&self.id).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(row.form_fields().iter().map(FormField::from_row).collect())
    }

    /// Gets `FormResponse`s submitted for this form
    async fn responses(&self) -> Vec<FormResponse> {
        FormResponseRow::query_by_form(&self.id)
            .iter()
            .map(FormResponse::from_row)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{FormFieldInput, FormFieldRow, FormFieldType, FormInput, FormRow};

    #[test]
    fn create_form() {
        let form = FormRow::create().unwrap();
        assert!(!form.id.is_empty());
        assert_eq!(form.title, "");
    }

    #[test]
    fn find_form() {
        let form = FormRow::create().unwrap();
        let found = FormRow::find(&form.id).unwrap();
        assert_eq!(found.id, form.id);
    }

    #[test]
    fn query_forms() {
        let before = FormRow::query().len();
        FormRow::create().unwrap();
        FormRow::create().unwrap();
        let after = FormRow::query().len();
        assert_eq!(after - before, 2);
    }

    #[test]
    fn create_or_update_form_creates_new() {
        let input = FormInput {
            id: uuid::Uuid::new_v4().to_string(),
            title: "My Form".to_string(),
            fields: vec![],
        };
        let form = FormRow::create_or_update(&input).unwrap();
        assert_eq!(form.title, "My Form");
    }

    #[test]
    fn create_or_update_form_updates_existing() {
        let form = FormRow::create().unwrap();

        let input = FormInput {
            id: form.id.clone(),
            title: "Updated Title".to_string(),
            fields: vec![],
        };
        let updated = FormRow::create_or_update(&input).unwrap();
        assert_eq!(updated.id, form.id);
        assert_eq!(updated.title, "Updated Title");
    }

    #[test]
    fn create_form_field() {
        let form = FormRow::create().unwrap();
        let field = FormFieldRow::create(&form.id).unwrap();
        assert!(!field.id.is_empty());
        assert_eq!(field.form_id, form.id);
        assert_eq!(field.input_type, FormFieldType::Text);
    }

    #[test]
    fn find_form_field() {
        let form = FormRow::create().unwrap();
        let field = FormFieldRow::create(&form.id).unwrap();
        let found = FormFieldRow::find(&field.id).unwrap();
        assert_eq!(found.id, field.id);
        assert_eq!(found.form_id, form.id);
    }

    #[test]
    fn create_or_update_form_field() {
        let form = FormRow::create().unwrap();
        let field = FormFieldRow::create(&form.id).unwrap();

        let input = FormFieldInput {
            id: field.id.clone(),
            label: "Name".to_string(),
            help: "Enter your name".to_string(),
            input_type: FormFieldType::Text,
            option_values: vec![],
        };
        let updated = FormFieldRow::create_or_update(&form.id, &input).unwrap();
        assert_eq!(updated.id, field.id);
        assert_eq!(updated.label, "Name");
        assert_eq!(updated.help, "Enter your name");
    }

    #[test]
    fn create_or_update_form_field_creates_new() {
        let form = FormRow::create().unwrap();

        let input = FormFieldInput {
            id: uuid::Uuid::new_v4().to_string(),
            label: "Age".to_string(),
            help: "Enter your age".to_string(),
            input_type: FormFieldType::Number,
            option_values: vec![],
        };
        let created = FormFieldRow::create_or_update(&form.id, &input).unwrap();
        assert_eq!(created.label, "Age");
        assert_eq!(created.input_type, FormFieldType::Number);
    }

    #[test]
    fn form_has_fields() {
        let form = FormRow::create().unwrap();
        assert_eq!(form.form_fields().len(), 0);

        FormFieldRow::create(&form.id).unwrap();
        FormFieldRow::create(&form.id).unwrap();
        assert_eq!(form.form_fields().len(), 2);
    }

    #[test]
    fn create_or_update_form_with_fields() {
        let form = FormRow::create().unwrap();

        let field_input = FormFieldInput {
            id: uuid::Uuid::new_v4().to_string(),
            label: "Color".to_string(),
            help: "Pick a color".to_string(),
            input_type: FormFieldType::Select,
            option_values: vec!["red".to_string(), "blue".to_string()],
        };

        let input = FormInput {
            id: form.id.clone(),
            title: "Survey".to_string(),
            fields: vec![field_input.clone()],
        };

        let updated = FormRow::create_or_update(&input).unwrap();
        assert_eq!(updated.title, "Survey");

        let fields = updated.form_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].label, "Color");
        assert_eq!(
            fields[0].option_values,
            vec!["red".to_string(), "blue".to_string()]
        );
    }
}
