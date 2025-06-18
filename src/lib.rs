pub mod store;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct Connection {
}

pub fn connect() -> Option<Connection> {
    None
}

#[cfg(test)]
mod tests {
    use super::connect;

    #[test]
    fn connection_fails() {
        let connection = connect();
        assert_eq!(connection, None);
    }
}
