use crate::frontend::table::{Attribute, Table};
use std::ops::Mul;
use std::collections::{HashMap, HashSet};
#[derive(Clone)]
pub struct StrippedPartition(Vec<HashSet<usize>>);

impl StrippedPartition {
    pub fn new(num: usize) -> StrippedPartition {
        let mut cnt = 0;
        let set: HashSet<usize> = std::iter::from_fn(move || {
            cnt += 1;
            if cnt - 1 < num {
                Some(cnt - 1)
            } else {
                None
            }
        }).collect();
        
        StrippedPartition(vec![set])
    }

    pub fn get_error(&self) -> i64 {
        let element_cnt: usize = self.0.iter().map(|x| x.len()).sum();

        element_cnt as i64 - self.0.len() as i64
    }
}

impl<'a> Mul for &'a StrippedPartition {
    type Output = StrippedPartition;

    fn mul(self, rhs: &'a StrippedPartition) -> Self::Output { 
        let mut temp_partition = StrippedPartition(Vec::new());
        let mut idx_check = HashMap::<usize, usize>::new();

        for (index, eq_class) in self.0.iter().enumerate() {
            temp_partition.0.push(HashSet::new());
            for entry in eq_class {
                idx_check.insert(*entry, index);
            }
        }

        for eq_class in rhs.0.iter() {
            for entry in eq_class {
                if let Some(index) = idx_check.get(entry) {
                    temp_partition.0.get_mut(*index).unwrap().insert(*entry);
                }
            }
        }

        let res: Vec<_> = temp_partition.0.into_iter().filter(|h| h.len() > 1).collect();
        StrippedPartition(res)
    }
}

pub struct Partitions<'a>(pub HashMap<&'a Attribute, StrippedPartition>);

impl<'a> Partitions<'a> {
    pub fn new(table: &Table) -> Partitions {
        let mut partition = HashMap::<&Attribute, StrippedPartition>::new();

        let mut cnt = 0;
        for meta in table.attributes.iter() {
            let mut entry_check = HashMap::<String, HashSet<usize>>::new();

            for (index, entry) in table.entries.iter().enumerate() {                
                let val = entry.data.get(cnt).unwrap();

                if let Some(indexs) = entry_check.get_mut(val) {
                    indexs.insert(index);
                } else {
                    let mut indexs = HashSet::new();
                    indexs.insert(index);
                    entry_check.insert(val.clone(), indexs);
                }
            }
            partition.insert(meta, StrippedPartition(entry_check.into_iter().map(|tulple|  tulple.1).collect()));

            cnt = cnt + 1;
        }

        Partitions(partition)
    }
}
