use rusqlite::{Connection, Result};
pub mod model;
use crate::model::Table;
use crate::model::Serializable;

#[derive(Debug)]
struct Measurement {
    force: f32,
    unit: String,
}

impl Serializable for Measurement {
    fn serialize(&self) -> String {
        format!("force: {}{}", self.force, self.unit)
    }
}

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    let measurement = Measurement {
        force: 4320.003,
        unit: "N".to_string(),
    };
    println!("Inserting measurement: {:?}", measurement);
    let table:Table<Measurement> = Table::new(&conn);
    table.init(&conn)?;
    table.add_row(measurement, &conn)?;
    table.show_rows(&conn)?;

    Ok(())
}
