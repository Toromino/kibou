use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::hash_map::{Entry, HashMap};

pub struct Cache {
    pub context: Box<HashMap<i64, i64>>,
    pub storage: Box<HashMap<i64, Vec<u8>>>,
}

impl Cache {
    pub fn get(&mut self, key: i64) -> Option<Vec<u8>> {
        self.maintain();
        match self.storage.entry(key) {
            Entry::Occupied(entry) => Some(entry.get().to_owned()),
            Entry::Vacant(entry) => None,
        }
    }

    pub fn maintain(&mut self) {
        for (id, timestamp) in self.context.iter() {
            if Utc::now()
                .signed_duration_since(DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(*timestamp, 0),
                    Utc,
                ))
                .num_minutes()
                > 5
            {
                self.context.to_owned().remove(id);
                self.storage.remove(id);
            }
        }
    }

    pub fn store(&mut self, key: i64, value: Vec<u8>) {
        self.maintain();
        self.context.insert(key, Utc::now().timestamp());
        self.storage.insert(key, value);
    }
}
