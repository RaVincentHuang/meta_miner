
use crate::algorithm::Algorithm;
use crate::frontend::partition::{Partitions, StrippedPartition};
use crate::frontend::table::Table;
use crate::dependency::result::AlgorithmResult;
use crate::dependency::fd::FDs;

use bit_set::BitSet;
use std::collections::HashMap;

use std::cell::RefCell;
struct Tane;

struct Combiantion {
    pub rhs: BitSet,
    pub partition: StrippedPartition,
    valid: bool
}

impl Combiantion {
    fn kill(&mut self) {
        self.valid = false;
    }

    fn is_valid(&self) -> bool {
        self.valid
    }
}

impl Algorithm for Tane {
    fn execute(&mut self, table: Table) -> Box<dyn AlgorithmResult> {
        let mut res = FDs::new(&table);

        let partitions = Partitions::new(&table);

        let attri_num = table.attributes.len();

        let mut level0 = HashMap::<BitSet, Combiantion>::new();
        let mut level1 = HashMap::<BitSet, Combiantion>::new();

        let level0_bitset: BitSet = (1..=attri_num).collect();

        let level0_sp = StrippedPartition::new(attri_num);
        level0.insert(BitSet::new(), Combiantion {
            rhs: level0_bitset,
            partition: level0_sp,
            valid: true
        });

        for (index, attri) in table.attributes.iter().enumerate() {
            let mut level1_key = BitSet::new();
            level1_key.insert(index + 1);

            let level1_bitset: BitSet = (1..=attri_num).collect();

            let level1_sp = partitions.0.get(attri).unwrap();

            level1.insert(level1_key, Combiantion {
                rhs: level1_bitset,
                partition: level1_sp.clone(),
                valid: true
            });

            let mut l = 1;
            while !level1.is_empty() && l <= attri_num {
                compute_dependencies(&mut level0, &mut level1, attri_num, &mut res);
                level1 = prune(level1, attri_num, &mut res);
                (level0, level1) = generate_next_level(level1);
                l += 1;
            }
        }

        Box::new(res)
    }
}

fn initial_c_plus_for_level(level0: &mut HashMap::<BitSet, Combiantion>, level1: &mut HashMap::<BitSet, Combiantion>, attri_num: usize) {
    for (X, ch) in level1.iter_mut() {
        let mut Cx_without_A_list = Vec::new();

        let mut X_clone = X.clone();
        for A in X {
            X_clone.remove(A);
            let ref Cx_without_a = level0.get(&X_clone).unwrap().rhs;

            Cx_without_A_list.push(Cx_without_a);
        }

        let mut C_for_X = BitSet::new();

        if !Cx_without_A_list.is_empty() {

            (1..=attri_num).for_each(|i| { C_for_X.insert(i); });
            for Cx_without_A in Cx_without_A_list {
                C_for_X.intersect_with(Cx_without_A);
            }
        }

        ch.rhs = C_for_X;
    }
}

fn compute_dependencies(level0: &mut HashMap::<BitSet, Combiantion>, level1: &mut HashMap::<BitSet, Combiantion>, attri_num: usize, res: &mut FDs) {
    initial_c_plus_for_level(level0, level1, attri_num);

    for (X, ch) in level1.iter_mut() {
        if ch.is_valid() {
            let ref mut c_plus = ch.rhs;
            let mut intersection = X.clone();

            intersection.intersect_with(c_plus);

            let mut X_clone = X.clone();

            for A in intersection.iter() {
                X_clone.remove(A);

                let ref spX_without_A = level0.get(&X_clone).unwrap().partition;
                let ref spX = ch.partition;

                if spX.get_error() == spX_without_A.get_error() {
                    let X_without_A = X_clone.clone();
                    res.add_from_index(&X_without_A.iter().map(|x| x - 1).collect(), A - 1);

                    ch.rhs.remove(A);

                    let mut R_without_X: BitSet = (1..=attri_num).collect();

                    R_without_X.difference_with(X);

                    R_without_X.iter().for_each(|i| {ch.rhs.remove(i);});
                }

                X_clone.insert(A);
            }
        }
    }
}

fn prune(level1: HashMap::<BitSet, Combiantion>, attri_num: usize, res: &mut FDs) -> HashMap::<BitSet, Combiantion> {
    let level1 = RefCell::new(level1);
    
    let mut element_to_remove = Vec::new();

    for (x, ch) in level1.borrow_mut().iter_mut() {
        if ch.rhs.is_empty() {
            element_to_remove.push(x.clone());
            continue;
        }

        if ch.is_valid() && ch.partition.get_error() == 0 {
            let mut rhs_without_x = ch.rhs.clone();

            rhs_without_x.difference_with(x);

            for a in rhs_without_x.iter() {
                let mut intersect : BitSet = (1..=attri_num).collect();

                let mut x_union_a_without_b = x.clone();

                x_union_a_without_b.insert(a);

                for b in x.iter() {
                    x_union_a_without_b.remove(b);
                    if let Some(c) = level1.borrow().get(&x_union_a_without_b) {
                        intersect.intersect_with(&c.rhs);
                    } else {
                        intersect = BitSet::new();
                        break;
                    }

                    x_union_a_without_b.insert(b);
                }
                if intersect.contains(a) {
                    let lhs = x.clone();
                    res.add_from_index(&lhs.iter().map(|x| x - 1).collect(), a - 1);
                    ch.rhs.remove(a);
                    ch.kill();
                }
            }
        }
    }

    for x in element_to_remove {
        level1.borrow_mut().remove(&x);
    }

    level1.into_inner()
}

fn generate_next_level(level1: HashMap::<BitSet, Combiantion>) -> (HashMap::<BitSet, Combiantion>, HashMap::<BitSet, Combiantion>) {
    let mut prefix_blocks: HashMap::<BitSet, Vec<BitSet>> = HashMap::new();

    level1.keys().for_each(|level_iter| {
        let mut prefix = level_iter.clone();
        prefix.remove(level_iter.iter().max().unwrap());
        
        if prefix_blocks.contains_key(&prefix) {
            prefix_blocks.get_mut(&prefix).unwrap().push(level_iter.clone());
        } else {
            let mut v = Vec::new();
            v.push(level_iter.clone());
            prefix_blocks.insert(prefix, v);
        }
    });

    let mut new_level = HashMap::new();

    for prefix_block_list in prefix_blocks.values() {
        if prefix_block_list.len() < 2 {
            continue;
        }

        let combinations = {
            let mut combinations = Vec::new();
            for i in 0..prefix_block_list.len() {
                for j in (i + 1)..prefix_block_list.len() {
                    combinations.push((prefix_block_list[i].clone(), prefix_block_list[j].clone()));
                }
            }
            combinations
        };

        for (a, b) in combinations {
            let mut X = a.clone();
            X.union_with(&b);
            let check = {
                let mut res = true;
                let mut X_clone = X.clone();
                for l in X.iter() {
                    X_clone.remove(l);
                    if !level1.contains_key(&X_clone) {
                        res = false;
                        break;
                    }
                    X_clone.insert(l);
                }
                res
            };

            if check {
                if level1.get(&a).unwrap().is_valid() && level1.get(&b).unwrap().is_valid() {
                    let st = &level1.get(&a).unwrap().partition * &level1.get(&b).unwrap().partition;
                    let rhs = BitSet::new();
                    new_level.insert(X, Combiantion { 
                        rhs: rhs, partition: st, valid: true 
                    });
                } else {
                    new_level.insert(X, Combiantion {
                        rhs: BitSet::new(), partition: StrippedPartition::new(0), valid: false
                    });
                }
            }
        }
    }

    (level1, new_level)
}