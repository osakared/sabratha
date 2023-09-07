pub mod time_series;

use chrono::Utc;
use crate::time_series::Database;

fn main() {
    let mut db = Database::new();
    db.append("101", Utc::now(), 0.123);
    db.append("101", Utc::now(), 4.23);
    println!("{:#?}", db);
}
