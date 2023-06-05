use std::collections::HashSet;

use crate::frontend::table::{*, self};
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PySet, PyList, PyString, PyTuple};
use serde::{Deserialize,Serialize};
#[derive(Deserialize, Serialize, Debug)]
struct Nodes(Vec<HashSet<usize>>);

pub fn clustering(table: &Table) -> Vec<HashSet<usize>> {

    let mut instance = Vec::new();
    for item in table.entries.iter() {
        instance.push(item.data.clone());
    }

    let py_cluster = include_str!("../../python/cluster.py");
    let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
        
        let cluster: Py<PyAny> = PyModule::from_code(py, py_cluster, "", "")?
            .getattr("clustering")?.into();

        let table: Vec<_> = instance.into_iter().map(|entry: Vec<String>| {
            let entry: Vec<_> = entry.into_iter().map(move |s| {
                PyString::new(py, s.as_str())
            }).collect();
            PyList::new(py, entry)
        }).collect();

        let table = PyList::new(py, table);
        // println!("{}", table);

        let args = PyTuple::new(py, vec![table]);
        cluster.call1(py, args)
    });

    let s = format!("{}", from_python.unwrap());
    let s = s.replace("{", "[");
    let s = s.replace("}", "]");
    let nodes: Nodes = serde_json::from_str(s.as_str()).unwrap();
    nodes.0
}