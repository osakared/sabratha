use rerun::TimeCell;

use sabratha::connect;
use sabratha::store::{Cell, SimpleStore, Store};

fn main() {
    let connection = connect();
    println!("{:#?}", connection);

    let mut store = SimpleStore::new();
    store.add_row("/data", TimeCell::from_duration_nanos(0), vec![Cell::of_float("temp".to_string(), 17.5, "C".to_string(), Some(3))]);
    let serialized = serde_json::to_string(&store).unwrap();
    println!("serialized = {}", serialized);
}
