use async_graphql::*;
use turbosql::{Turbosql, select};
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Enum, Copy, Clone, Eq, PartialEq)]
pub enum FormInputType {
    Text,
    Number,
    CheckboxBoolean,
    CheckboxMultiple,
    Radio,
    Select,
    Signature,
    ImageFile,
}

#[derive(serde::Serialize, serde::Deserialize, SimpleObject, Clone)]
pub struct FormInput {
    pub label: String,
    pub help: String,
    pub input_type: FormInputType,
    // Not using this as part of enum due to limitations of gql
    pub option_values: Vec<String>,
}

#[derive(Turbosql, Default, SimpleObject)]
pub struct Form {
    pub rowid: Option<i64>,
    pub id: Option<String>,
    pub title: Option<String>,
    pub fields: Option<Vec<FormInput>>,
}

impl Form {
    pub fn new() -> Option<Form> {
        let rowid = Form {
            id: Some(Uuid::new_v4().to_string()),
            title: Some(String::new()),
            fields: Some(vec![]),
            ..Default::default()
        }.insert();
        match rowid {
            Ok(id) => select!(Form "WHERE rowid = " id).ok(),
            Err(_) => None
        }
    }

    pub fn query() -> Vec<Form> {
        select!(Vec<Form>).unwrap_or(vec![])
    }
}
