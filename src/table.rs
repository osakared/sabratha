use rerun::{external::arrow::array::ArrayRef, log::{Chunk, RowId}, sink::MemorySinkStorage, ComponentDescriptor, RecordingStream, TimePoint, Timeline};

#[derive(Debug)]
pub enum TripoliError {
    Allocation,
}

/// Represents tabular data
pub trait Table {
    fn add_row(self, path: &str, time: i64, components: impl IntoIterator<Item = (ComponentDescriptor, ArrayRef)>);
    fn create_in_memory() -> Result<Self, TripoliError> where Self: Sized;
}

pub struct RerunTable {
    recording_stream: RecordingStream,
    storage: MemorySinkStorage,
    timeline: Timeline,
}

impl Table for RerunTable {
    fn add_row(self, path: &str, time: i64, components: impl IntoIterator<Item = (ComponentDescriptor, ArrayRef)>) {
        // We're using `send_chunk` because the more standard `log` functions don't let me specify timestamp
        // directly, which we will generally get from the instrument, instead of the computer timing it
        // See https://github.com/rerun-io/rerun/issues/3841

        let row_id = RowId::new();
        let timepoint = TimePoint::from([(self.timeline, time)]);
        match Chunk::builder(path.into()).with_row(row_id, timepoint, components).build() {
            Ok(chunk) => self.recording_stream.send_chunk(chunk),
            Err(_) => (),
        }
    }

    fn create_in_memory() -> Result<Self, TripoliError> {
        let rec = rerun::RecordingStreamBuilder::new("tripoli.app").memory();
        let timeline = Timeline::new("datetime", rerun::time::TimeType::TimestampNs);

        match rec {
            Ok((recording_stream, storage)) => Ok(RerunTable {
                recording_stream,
                storage,
                timeline,
            }),
            Err(_) => Err(TripoliError::Allocation),
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::table::{RerunTable, Table};
    use super::{Table, RerunTable};

    #[test]
    fn log_data() {
        let table = RerunTable::create_in_memory().unwrap();
        table.add_row("/data", 0, []);
        assert!(true);
    }
}
