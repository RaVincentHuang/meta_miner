use crate::frontend::table::{Attribute, Table};
use crate::dependency::result::AlgorithmResult;
use bit_set::BitSet;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Attributes(Vec<Attribute>);

#[derive(Deserialize, Serialize)]
pub struct FunctionalDependency {
    determinant: Attributes,
    dependant: Attribute
}

struct Foo;

trait MyTrait {
    fn foo() -> usize {
        return 0;
    }
}

impl MyTrait for Foo {

}

impl FunctionalDependency {
    pub fn new_from_vec(attr_vec: Vec<Attribute>) -> FunctionalDependency {
        let mut dependant = Attribute {value: "".to_string(), rank: 0};
        
        let mut group = Vec::<Attribute>::new();
        
        let len = attr_vec.len();

        for (index, attr) in attr_vec.into_iter().enumerate() {
            if index != len - 1 {
                group.push(attr);
            } else {
                dependant = attr;
            }
        }

        group.sort();

        let determinant = Attributes(group);

        FunctionalDependency { determinant, dependant }

    }

    pub fn distance(fd1: &Self, fd2: &Self, delta1: f64, delta2: f64, delta3: f64) -> f64 {
        let x1: BitSet = fd1.determinant.0.iter().map(|a| a.rank).collect();
        let x2: BitSet = fd2.determinant.0.iter().map(|a| a.rank).collect();

        let mut y1 = BitSet::new();
        y1.insert(fd1.dependant.rank);

        let mut y2 = BitSet::new();
        y2.insert(fd2.dependant.rank);

        let s1: BitSet = x1.union(&y1).collect();
        let s2: BitSet = x2.union(&y2).collect();

        let cnt1 = s1.symmetric_difference(&s2).collect::<BitSet>().len() as f64;
        let cnt2 = x1.symmetric_difference(&x2).collect::<BitSet>().len() as f64;
        let cnt3 = y1.symmetric_difference(&y2).collect::<BitSet>().len() as f64;

        delta1 * cnt1 + delta2 * cnt2 + delta3 * cnt3
    }

    pub fn distance_std(fd1: &Self, fd2: &Self, n: usize) -> f64 {
        let n = n as f64;
        Self::distance(fd1, fd2, 1.0, (n - 1.0) / (n * n), 1.0 / (n * n))
    }

    pub fn r_neighborhood_cnt(&self, n: usize, r: f64) -> usize {
        fn dfs(index: usize, n: usize, deter_set: &mut BitSet, fd: &FunctionalDependency, r: f64) -> usize {
            let mut cnt = 0;
            if index < n {
                cnt += dfs(index + 1, n, deter_set, fd, r);
                deter_set.insert(index);
                cnt += dfs(index + 1, n, deter_set, fd, r);
                deter_set.remove(index);
            } else if deter_set.len() != n {
                let mut least: BitSet = (0..n).collect();
                least.difference_with(deter_set);
                for idx in least.iter() {
                    let distance = |x1: &BitSet, y1: &BitSet, x2: &BitSet, y2: &BitSet| {
                        let s1: BitSet = x1.union(&y1).collect();
                        let s2: BitSet = x2.union(&y2).collect();

                        let cnt1 = s1.symmetric_difference(&s2).collect::<BitSet>().len() as f64;
                        let cnt2 = x1.symmetric_difference(&x2).collect::<BitSet>().len() as f64;
                        let cnt3 = y1.symmetric_difference(&y2).collect::<BitSet>().len() as f64;

                        let n = n as f64;

                        let delta1 = 1.0;
                        let delta2 = (n - 1.0) / (n * n);
                        let delta3 = 1.0 / (n * n);

                        delta1 * cnt1 + delta2 * cnt2 + delta3 * cnt3
                    };

                    let x1: BitSet = fd.determinant.0.iter().map(|a| a.rank).collect();
                    let mut y1 = BitSet::new();
                    y1.insert(fd.dependant.rank);

                    let mut y2 = BitSet::new();
                    y2.insert(idx);

                    if distance(&x1, &y1, &deter_set, &y2) < r {
                        cnt += 1;
                    }
                }
            }
    
            cnt
        }
        
        let mut deter_set = BitSet::new();

        dfs(0, n, &mut deter_set, self, r)
    }
}


impl Display for Attributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: String = self.0.iter().map(|a| a.value.clone()).intersperse(", ".to_string()).collect();
        write!(f, "{{{}}}", s)
    }
}

impl Display for FunctionalDependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.determinant, self.dependant)
    }
}

pub struct FDs {
    fds: Vec<FunctionalDependency>,
    table_name: String,
    attributes: Rc<Vec<Attribute>>,
}

impl FDs {
    pub fn new(table: &Table) -> FDs {
        FDs {fds: Vec::new(), table_name: table.table_name.clone(), attributes: Rc::clone(&table.attributes)}
    }

    pub fn add(&mut self, fd: FunctionalDependency) {
        self.fds.push(fd);
    }

    pub fn add_from_index(&mut self, X: &BitSet, a: usize) {
        let mut determinant: Vec<_> = X.iter().map(|index| {
            self.attributes.get(index).unwrap().clone()
        }).collect();

        determinant.sort();

        let determinant = Attributes(determinant);

        let dependant = self.attributes.get(a).unwrap().clone();

        let fd = FunctionalDependency {determinant, dependant};

        self.add(fd);
    }

    pub fn r_neighborhood(&self, fd: &FunctionalDependency, r: f64) -> usize {
        let n = self.attributes.len();
        fd.r_neighborhood_cnt(n, r)
    }


    pub fn r_neighborhood_with_M(&self, fd: &FunctionalDependency, r: f64) -> usize {
        let mut cnt = 0;
        let n = self.attributes.len();
        self.fds.iter().for_each(|f| {
            if FunctionalDependency::distance_std(f, fd, n) < r {
                cnt += 1;
            }
        });

        cnt
    }
}

impl AlgorithmResult for FDs {
    fn dispaly(&self) {
        println!("We have functional dependency set of the table {}:", self.table_name);
        for fd in self.fds.iter() {
            print!("{};\t", fd);
        }
    }

    fn save_as_file(&self) -> Result<(), std::io::Error> {

        Ok(())
    }
}