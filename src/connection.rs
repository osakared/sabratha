use std::sync::Arc;

use futures_util::lock::Mutex;
use async_graphql::{Enum, Object};
use slab::Slab;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone, Copy, Enum, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connected,
    Idle,
    Connecting,
}

#[derive(Clone)]
pub struct Connection {
    status: ConnectionStatus,
    id: Uuid,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            status: ConnectionStatus::Disconnected,
            id: Uuid::new_v4(),
        }
    }
}

#[Object]
impl Connection {
    async fn status(&self) -> &ConnectionStatus {
        &self.status
    }

    async fn id(&self) -> String {
        self.id.to_string()
    }
}

pub type Storage = Arc<Mutex<Slab<Connection>>>;

#[cfg(test)]
mod tests {
    use super::{Connection, ConnectionStatus};

    #[test]
    fn connection_fails() {
        let connection = Connection::new();
        assert_eq!(connection.status, ConnectionStatus::Disconnected);
    }
}