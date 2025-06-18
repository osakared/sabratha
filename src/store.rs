use std::collections::HashMap;
use std::vec::Vec;

use rerun::{Scalar, TimeCell};
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum TripoliError {
    Allocation,
}

pub type Unit = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct ScalarValue<T> {
    val: T,
    unit: Unit,
    significant_digits: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CellType {
    Text(String),
    Float(ScalarValue<f64>),
    Int(ScalarValue<i64>),
    Time(TimeCell),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cell {
    col_name: String,
    cell_type: CellType,
}

impl Cell {
    pub fn of_text(col_name: String, text: String) -> Self {
        Self {
            col_name,
            cell_type: CellType::Text(text),
        }
    }

    pub fn of_float(col_name: String, float: f64, unit: Unit, significant_digits: Option<i32>) -> Self {
        Self {
            col_name,
            cell_type: CellType::Float(ScalarValue{val: float, unit, significant_digits}),
        }
    }

    pub fn of_int(col_name: String, int: i64, unit: Unit, significant_digits: Option<i32>) -> Self {
        Self {
            col_name,
            cell_type: CellType::Int(ScalarValue{val: int, unit, significant_digits}),
        }
    }
}

pub trait Store {
    fn add_row(&mut self, path: &str, time: TimeCell, data: Vec<Cell>);
    fn create_in_memory() -> Result<Self, TripoliError> where Self: Sized;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleTable {
    rows: Vec<Vec<Cell>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleStore {
    tables: HashMap<String, SimpleTable>,
}

impl SimpleTable {
    fn new() -> Self {
        Self {
            rows: vec![]
        }
    }
}

impl SimpleStore {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new()
        }
    }
}

impl Store for SimpleStore {
    fn add_row(&mut self, path: &str, time: TimeCell, mut data: Vec<Cell>) {
        data.push(Cell {col_name: "time".to_string(), cell_type: CellType::Time(time)});
        self.tables.entry(path.to_string()).or_insert(SimpleTable::new()).rows.push(data);
    }

    fn create_in_memory() -> Result<Self, TripoliError> where Self: Sized {
        Ok(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use rerun::TimeCell;

    use crate::store::Cell;

    use super::{SimpleStore, Store};

    #[test]
    fn adding_rows_works() {
        let mut store = SimpleStore::new();
        store.add_row("/data", TimeCell::from_duration_nanos(0), vec![Cell::of_text("success".to_string(), "PASS".to_string())]);
        store.add_row("/data", TimeCell::from_duration_nanos(0), vec![Cell::of_text("success".to_string(), "FAIL".to_string())]);
        assert_eq!(store.tables.get("/data").unwrap().rows.len(), 2);
    }
}
