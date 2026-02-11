use async_graphql::{Context, EmptySubscription, Object, Schema};

use crate::connection::{Connection, Storage};
use crate::form_responses::{
    FormFieldResponseData, FormFieldResponseInput, FormResponse, FormResponseRow,
};
use crate::forms::{Form, FormInput, FormRow};

trait ConvertsToGQLError {
    fn to_gql_error(&self) -> async_graphql::Error;
}

impl ConvertsToGQLError for turbosql::Error {
    fn to_gql_error(&self) -> async_graphql::Error {
        async_graphql::Error::new(self.to_string())
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn connections(&self, ctx: &Context<'_>) -> Vec<Connection> {
        let connections = ctx.data_unchecked::<Storage>().lock().await;
        connections
            .iter()
            .map(|(_, connection)| connection)
            .cloned()
            .collect()
    }

    async fn forms(&self) -> Vec<Form> {
        FormRow::query().iter().map(Form::from_row).collect()
    }

    async fn form_responses(&self) -> Vec<FormResponse> {
        FormResponseRow::query()
            .iter()
            .map(FormResponse::from_row)
            .collect()
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_form(&self) -> Result<String, async_graphql::Error> {
        match FormRow::create() {
            Ok(form) => Ok(form.id),
            Err(err) => Err(err.to_gql_error()),
        }
    }

    async fn update_form(&self, form_input: FormInput) -> Result<String, async_graphql::Error> {
        match FormRow::create_or_update(&form_input) {
            Ok(form) => Ok(form.id),
            Err(e) => Err(e.to_gql_error()),
        }
    }

    async fn create_form_response(&self, form_id: String) -> Result<String, async_graphql::Error> {
        match FormResponseRow::create(&form_id) {
            Ok(response) => Ok(response.id),
            Err(err) => Err(err.to_gql_error()),
        }
    }

    async fn update_form_field_response(
        &self,
        form_response_id: String,
        field_response_input: FormFieldResponseInput,
    ) -> Result<String, async_graphql::Error> {
        let mut row = FormResponseRow::find(&form_response_id).map_err(|e| e.to_gql_error())?;
        let data = FormFieldResponseData::from_input(&field_response_input);
        let result = row
            .upsert_field_response(data)
            .map_err(|e| e.to_gql_error())?;
        Ok(result.id)
    }
}

pub struct Root {
    pub schema: Schema<QueryRoot, MutationRoot, EmptySubscription>,
    pub storage: Storage,
}

impl Root {
    pub fn new() -> Self {
        let storage = Storage::default();
        let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
            .data(storage.clone())
            .finish();
        Self { schema, storage }
    }
}
