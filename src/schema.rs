use async_graphql::{Context, EmptySubscription, Object, Schema};

use crate::forms::Form;
use crate::connection::{Connection, Storage};

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
        connections.iter().map(|(_, connection)| connection).cloned().collect()
    }

    async fn forms(&self) -> Vec<Form> {
        Form::query()
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_form(&self) -> Result<String, async_graphql::Error> {
        match Form::create() {
            Ok(form) => Ok(form.id.unwrap_or(String::new())),
            Err(err) => Err(err.to_gql_error())
        }
    }

    async fn update_form(&self, form_input:Form) -> Result<String, async_graphql::Error> {
        let form = match &form_input.id {
            Some(id) => Form::find(&id),
            None => Form::create()
        };
        match form {
            Ok(mut f) =>
                match f.update_from(&form_input) {
                    Ok(_) => Ok(f.id.unwrap_or(String::new())),
                    Err(err) => Err(err.to_gql_error())
                },
            Err(err) => Err(err.to_gql_error())
        }
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
        Self { schema, storage, }
    }
}
