use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};

use crate::connection::{Connection, Storage};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn connections(&self, ctx: &Context<'_>) -> Vec<Connection> {
        let connections = ctx.data_unchecked::<Storage>().lock().await;
        connections.iter().map(|(_, connection)| connection).cloned().collect()
    }
}

pub struct Root {
    pub schema: Schema<QueryRoot, EmptyMutation, EmptySubscription>,
    pub storage: Storage,
}

impl Root {
    pub fn new() -> Self {
        let storage = Storage::default();
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(storage.clone())
            .finish();
        Self { schema, storage, }
    }
}
