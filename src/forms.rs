use async_graphql::*;
use turbosql::{Turbosql, select};
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Enum, Copy, Clone, Eq, PartialEq)]
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

#[derive(serde::Serialize, serde::Deserialize, SimpleObject, InputObject, Clone)]
#[graphql(input_name = "FormFieldInput")]
pub struct FormField {
    pub label: String,
    pub help: String,
    pub input_type: FormFieldType,
    // Not making this algebraic subtype of enum due to limitations of gql
    pub option_values: Vec<String>,
}

#[derive(Turbosql, Default, SimpleObject, InputObject, Clone)]
#[graphql(input_name = "FormInput")]
pub struct Form {
    // Only for local sqlite db so shouldn't be exposed to clients
    #[graphql(skip)]
    pub rowid: Option<i64>,
    // Globally unique id to facilitate decentralization
    pub id: Option<String>,
    pub title: Option<String>,
    pub fields: Option<Vec<FormField>>,
}

impl Form {
    pub fn create() -> Result<Form, turbosql::Error> {
        let form = Form {
            rowid: None,
            id: Some(Uuid::new_v4().to_string()),
            title: Some(String::new()),
            fields: Some(vec![]),
        };
        match form.insert() {
            Ok(id) => select!(Form "WHERE rowid = " id),
            Err(e) => Err(e)
        }
    }

    /// Gets `Vec` of `Form`s
    pub fn query() -> Vec<Form> {
        select!(Vec<Form>).unwrap_or(vec![])
    }

    /// Tries to find a single `Form` with given id
    pub fn find(id:&String) -> Result<Form, turbosql::Error> {
        select!(Form "where id = " id)
    }

    /// Updates from content of other `Form`
    pub fn update_from(&mut self, form:&Form) {
        // Is there a more idiomatic/rustic way of doing this?
        // I'm also concerned more lines need to be added for each non-id, non-rowid field
        // ...possible source of errors!
        if let Some(title) = &form.title {
            self.title = Some(title.clone());
        }
        if let Some(fields) = &form.fields {
            self.fields = Some(fields.clone());
        }
        self.update();
    }
}
