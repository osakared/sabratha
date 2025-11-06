use std::error::Error;

use async_graphql::{http::GraphiQLSource};
use async_graphql_poem::*;
use poem::{listener::TcpListener, web::Html, *};

use sabratha::schema::Root;

#[handler]
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create the schema
    let app = Root::new();

    // start the http server
    let app = Route::new().at("/", get(graphiql).post(GraphQL::new(app.schema)));
    println!("GraphiQL: http://localhost:8000");
    Server::new(TcpListener::bind("0.0.0.0:8000"))
        .run(app)
        .await?;
    Ok(())
}