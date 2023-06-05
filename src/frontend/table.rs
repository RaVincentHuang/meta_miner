
use std::{rc::Rc, fmt::Display, collections::HashSet};
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

    pub fn sub_table(&self, node: &HashSet<usize>) -> Table {
        let table_name = self.table_name.clone();
        let attributes = Rc::clone(&self.attributes);
        let entries: Vec<_> = node.iter().map(|idx| {
            let entry = self.entries.get(*idx).unwrap();
            Entry { attr_ref: Rc::clone(&entry.attr_ref), data: entry.data.clone()}
        }).collect();

        Table { table_name, attributes, entries}
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        res += format!("name: {}\n", self.table_name).as_str();
        for attri in self.attributes.iter() {
            res += format!("{:<15}", attri.value).as_str();
        }
        res += "\n";
        for entry in self.entries.iter() {
            res += format!("{}\n", entry).as_str();
        }
        write!(f, "{}", res)
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
impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        for item in self.data.iter() {
            res += format!("{:<15}", item).as_str();
        }
        write!(f, "{}", res)
    }
}