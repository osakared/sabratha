use async_graphql::{Context, EmptySubscription, Object, Schema};

use crate::forms::Form;
use crate::connection::{Connection, Storage};

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
    async fn create_form(&self, ctx: &Context<'_>) -> String {
        match Form::new() {
            Some(form) => form.id.unwrap_or(String::new()),
            None => "Failed to create form".to_string()
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
