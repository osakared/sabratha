use chrono::{Utc, DateTime};
use std::collections::HashMap;

#[derive(Debug)]
struct TimeSeries {
    values:Vec<(i64, f64)>,
}

impl TimeSeries {
    fn new() -> Self {
        Self { values: Vec::new() }
    }

    fn append(&mut self, time:DateTime<Utc>, value:f64) {
        self.values.push((time.timestamp(), value));
    }
}

#[derive(Debug, Eq, Hash)]
pub struct TimeSeriesLabel {
    channel:String,
    sig_figures:isize,
    // unit,
}

impl TimeSeriesLabel {
    fn new(channel:&str) -> Self {
        Self {
            channel: channel.to_string(),
            sig_figures: 3
        }
    }
}

impl PartialEq for TimeSeriesLabel {
    fn eq(&self, other: &Self) -> bool {
        self.channel == other.channel
    }
}

#[derive(Debug)]
pub struct Database {
    data_sets:HashMap<TimeSeriesLabel, TimeSeries>,
}

impl Database {
    pub fn append(&mut self, channel:&str, time:DateTime<Utc>, value:f64) {
        let label = TimeSeriesLabel::new(channel);
        if let Some(map) = self.data_sets.get_mut(&label) {
            map.append(time, value);
        }
        else {
            let mut data_set = TimeSeries::new();
            data_set.append(time, value);
            self.data_sets.insert(label, data_set);
        }
    }

    pub fn new() -> Self {
        Self {
            data_sets: HashMap::new()
        }
    }
}