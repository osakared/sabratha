use rusqlite::{Connection, Result};
use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::{Uuid};

// Using this for now until we can properly turn arbitrary POD structs into rows
pub trait Serializable {
    fn serialize(&self) -> String;
}

// Represents a table of data for a connection
pub struct Table<T: Serializable> {
    id: Uuid,
    row_type: PhantomData<T>,
}

// I want to somehow make a macro (don't know what's possible in rust macros yet)
// That, based on the struct type, defines the schema of the table (w/ added time row), and automatically
// inserts, reads accordingly - make it all type-safe but don't assume the db wasn't
// modified under us either
impl<T: Serializable> Table<T> {
    pub fn new(conn:&Connection) -> Self {
        Self {
            id: Uuid::new_v4(),
            row_type: PhantomData,
        }
    }

    pub fn table_name(&self) -> String {
        format!("Row{}", self.id.simple().to_string())
    }

    pub fn init(&self, conn:&Connection) -> Result<()> {
        // What I'm doing here with format!...as_str() is surely not optimal
        conn.execute(
            format!("CREATE TABLE {} (
                id    INTEGER PRIMARY KEY,
                date  INTEGER NOT NULL,
                data  TEXT NOT NULL
            )", self.table_name()).as_str(),
            (),
        )?;
        Ok(())
    }

    pub fn add_row(&self, row:T, conn:&Connection) -> Result<()> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        conn.execute(
            format!("INSERT INTO {} (date, data) VALUES (?1, ?2)", self.table_name()).as_str(),
            (time, row.serialize()),
        )?;
        Ok(())
    }

    pub fn show_rows(&self, conn:&Connection) -> Result<()> {
        let mut stmt = conn.prepare(format!("SELECT id, date, data FROM {}", self.table_name()).as_str())?;
        let row_iter = stmt.query_map([], |row| {
            let col1:i32 = row.get(0)?;
            let col2:i32 = row.get(1)?;
            let col3:String = row.get(2)?;
            Ok(format!("row: {} {} {}", col2, col1, col3))
        })?;
    
        for row in row_iter {
            println!("Found row {:?}", row.unwrap());
        }
        Ok(())
    }
}