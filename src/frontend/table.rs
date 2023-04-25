
use std::{rc::Rc, fmt::Display};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Table {
    pub table_name: String,
    pub attributes: Rc<Vec<Attribute>>,
    pub entries: Vec<Entry>
}

impl Table {
    pub fn new(table_name: String, metadata: Vec<Attribute>) -> Table {
        Table {
            table_name,
            attributes: Rc::new(metadata),
            entries: Vec::new()
        }
    }

    pub fn add_entry(&mut self, data: Vec<String>) {
        let entry = Entry::new(Rc::clone(&self.attributes), data);
        self.entries.push(entry);
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Attribute {
    pub rank: usize,
    pub value: String
}

impl Attribute {
    pub fn new(rank: usize, value: String) -> Attribute {
        Attribute { rank, value}
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug)]
pub struct Entry {
    pub attr_ref: Rc<Vec<Attribute>>,
    pub data: Vec<String>
}

impl Entry {
    pub fn new(attr_ref: Rc<Vec<Attribute>>, data: Vec<String>) -> Entry {
        Entry { attr_ref, data}
    }

    pub fn get_data_from_metadata(&self, metadata: &String) -> Option<&String> {
        let mut cnt = 0;
        for meta in self.attr_ref.iter() {
            if meta.value == *metadata {
                return Some(&self.data[cnt]);
            }
            cnt = cnt + 1;
        }

        None
    }
}