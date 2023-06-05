use crate::dependency::result::AlgorithmResult;
use crate::frontend::table::Table;

pub trait Algorithm {
    fn execute(&mut self, table: &Table) -> Box<dyn AlgorithmResult>;
}

pub mod tane;
pub mod cluster;