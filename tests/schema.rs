use async_graphql::{value};

use sabratha::connection::Connection;
use sabratha::schema::Root;

#[tokio::test]
pub async fn schema_check() {
    let app = Root::new();
    app.storage.lock().await.insert(Connection::new());
    let res = app.schema.execute("{ connections { status } }").await;
    assert_eq!(
        res.data,
        value!({
            "connections": [
                {
                    "status": "DISCONNECTED"
                }
            ]
        }));
}