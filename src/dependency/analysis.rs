use crate::dependency::fd::*;
use crate::frontend::table::Attribute;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use regex::Regex;
use bit_set::BitSet;
use std::fmt::{Formatter, Display};
use std::io::{BufReader, BufWriter, Read, Write, stdin, BufRead};
use std::fs::{File, OpenOptions};

#[derive(Deserialize, Serialize)]
struct FDs(Vec<FD>, String);

#[derive(Deserialize, Serialize)]
struct AttributesSink(Attributes);

impl AttributesSink {
    fn new() -> AttributesSink {
        let f = if let Ok(f) = File::open("attributes.json") {
            f
        } else {
            File::create("attributes.json").unwrap()
        };

        let mut reader = BufReader::new(f);
        let mut load = String::new();

        if let Ok(_) = reader.read_to_string(&mut load) {
            log::debug!("Already exit {}", load);
        } else {
            log::warn!("Empty file");
        }

        let attributes: Attributes = if let Ok(attributes) = serde_json::from_str(load.as_str()) {
            attributes
        } else {
            log::warn!("Attributes Load Error");
            Attributes(Vec::new())
        };

        AttributesSink(attributes)
    }

    fn build_attributes(&self, index: usize) -> Attribute {
        if let Some(attribute) = self.0.0.get(index) {
            assert_eq!(attribute.rank, index);
            attribute.clone()
        } else {
            log::error!("Can not find the {} in the attributes set", index);
            panic!("attributes error");
        }
    }

    fn clear(&mut self) {
        self.0.0.clear();
    }
}

impl Drop for AttributesSink {
    fn drop(&mut self) {
        let s = serde_json::to_string(&self).unwrap();

        log::trace!("Save: {}", s);

        let f = OpenOptions::new()
            .write(true).truncate(true)
            .open("attributes.json").unwrap();

        let mut wirter = BufWriter::new(f);

        wirter.write(s.as_bytes()).unwrap();

    }
}

#[derive(Deserialize, Serialize)]
struct FD {
    determinant: Attributes,
    dependant: Attributes
}

impl Display for FD {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.determinant, self.dependant)
    }
}

impl FD {
    pub fn new_from_vec(determinant: Vec<Attribute>, dependant: Vec<Attribute>) -> FD {
        
        let mut determinant = determinant;
        let mut dependant = dependant;

        determinant.sort();
        dependant.sort();
        
        let determinant = Attributes(determinant);
        let dependant = Attributes(dependant);

        FD { determinant, dependant }

    }

    pub fn distance(fd1: &Self, fd2: &Self, delta1: f64, delta2: f64, delta3: f64) -> f64 {
        let x1: BitSet = fd1.determinant.0.iter().map(|a| a.rank).collect();
        let x2: BitSet = fd2.determinant.0.iter().map(|a| a.rank).collect();

        let y1: BitSet = fd1.dependant.0.iter().map(|a| a.rank).collect();
        let y2: BitSet = fd2.dependant.0.iter().map(|a| a.rank).collect();

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
        fn dfs(index: usize, n: usize, deter_set: &mut BitSet, x1: &BitSet, y1: &BitSet, r: f64) -> usize {
            let mut cnt = 0;
            if index < n {
                cnt += dfs(index + 1, n, deter_set, x1, y1, r);
                deter_set.insert(index);
                cnt += dfs(index + 1, n, deter_set, x1, y1, r);
                deter_set.remove(index);
            } else {

                let mut determinant_set = deter_set.clone();
                let mut dependant_set = BitSet::new();

                for idx in deter_set.iter() {
                    determinant_set.remove(idx);
                    dependant_set.insert(idx);

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

                    if distance(&x1, &y1, &determinant_set, &dependant_set) < r {
                        cnt += 1;
                    }
                }
            }
    
            cnt
        }
        
        let mut deter_set = BitSet::new();

        let x1: BitSet = self.determinant.0.iter().map(|a| a.rank).collect();
        let y1: BitSet = self.dependant.0.iter().map(|a| a.rank).collect();

        dfs(0, n, &mut deter_set, &x1, &y1, r)
    }
}

impl FDs {
    fn new(filename: &str) -> FDs {
        let f = if let Ok(f) = File::open(filename) {
            f
        } else {
            File::create(filename).unwrap()
        };

        let mut reader = BufReader::new(f);
        let mut load = String::new();

        if let Ok(_) = reader.read_to_string(&mut load) {
            log::debug!("Already exit {}", load);
        } else {
            log::debug!("Empty file");
        }

        let mut fds: FDs = if let Ok(fds) = serde_json::from_str(load.as_str()) {
            // fds.1 = String::from(s.clone());
            fds
        } else {
            FDs(Vec::new(), String::from(filename.clone()))
        };

        fds.1 = String::from(filename);

        fds
    }

    fn mined() -> FDs {
        Self::new("mined.json")
    }

    fn new_fd() -> FDs {
        Self::new("new.json")
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

impl Drop for FDs {
    fn drop(&mut self) {
        for fd in self.0.iter() {
            log::trace!("Save fd: {}", fd);
        }
        let s = serde_json::to_string(&self).unwrap();

        let filepath = self.1.as_str();

        log::trace!("Save: {}", s);

        let f = OpenOptions::new()
            .write(true).truncate(true)
            .open(filepath).unwrap();

        let mut wirter = BufWriter::new(f);

        wirter.write(s.as_bytes()).unwrap();

    }
}

pub enum Action {
    Add,
    Clear,
    Load,
    Display,
    Single,
    Current,
    Attributes,
    Nothing
}

pub enum Type {
    Mined,
    New,
    Attributes
}

pub enum Output {
    Std,
}



pub fn analysis_cli(action: Action, fd_type: Type, n: usize, r: f64) {

    fn parser_fd(fd_str: &String, attribute_sink: &AttributesSink) -> FD {
        lazy_static!{
            static ref REGEX: Regex = Regex::new(
                r##"(?P<l>[\t 0-9,]+)->(?P<r>[\t 0-9,]+)"##
            ).unwrap();

            static ref NUM: Regex = Regex::new(
                r##"([[:digit:]]+)"##
            ).unwrap();
        }

        let (determinant, dependant) = {
            if let Some(cap) = REGEX.captures(fd_str.as_str()) {
                if let (Some(determinant), Some(dependant)) = (cap.name("l"), cap.name("r")) {
                    (determinant.as_str(), dependant.as_str())
                } else {
                    log::error!("Capture lost at {}", fd_str);
                    panic!("Capture lost!");
                }
            } else {
                log::error!("Capture lost at {}", fd_str);
                panic!("Capture lost!");
            }
        };

        let mut determinant_vec = Vec::new();

        let mut dependant_vec = Vec::new();

        for cap in NUM.captures_iter(determinant) {
            let num = cap[0].to_string().parse::<usize>().unwrap();
            let attr = attribute_sink.build_attributes(num);
            determinant_vec.push(attr);
        }

        for cap in NUM.captures_iter(dependant) {
            let num = cap[0].to_string().parse::<usize>().unwrap();
            let attr = attribute_sink.build_attributes(num);
            dependant_vec.push(attr);
        }

        let fd = FD::new_from_vec(determinant_vec, dependant_vec);
        fd
    }
   
    match action {

        Action::Add | Action::Single => {
            let mut fd_str = String::new();
            println!("Please input the fd as format of (index)* -> (index)*");
            stdin().read_line(&mut fd_str).unwrap();

            log::info!("Get str {}", fd_str);

            let attribute_sink = AttributesSink::new();

            let fd = parser_fd(&fd_str, &attribute_sink);

            if let Action::Add = action {
                match fd_type {
                    Type::Mined => {
                        save_mined_fd(fd);
                    }
                    Type::New => {
                        save_new_fd(fd);
                    }
                    Type::Attributes => {
                        println!("Can not add single Attribute");
                    }
                }
            } else {
                analysis_one_and_output(&fd, n, r);
            }

        }
        Action::Load => {
            println!("Enter the load file of the format (index*) -> (index*)");
            let mut file = String::new();
            stdin().read_line(&mut file).unwrap();
            let file = file.trim().to_string();
            let attribute_sink = AttributesSink::new();
            let file = OpenOptions::new().read(true).open(file).unwrap();
            let reader = BufReader::new(file);

            for line in reader.lines() {
                if let Ok(line) = line {
                    log::debug!("{}", line);
                    let fd = parser_fd(&line, &attribute_sink);
                    log::debug!("Load FD: {}", fd);
                    match fd_type {
                        Type::Mined => {
                            save_mined_fd(fd);
                        }
                        Type::New => {
                            save_new_fd(fd);
                        }
                        Type::Attributes => {
                            println!("Can not load single Attribute");
                        }
                    }
                }
            }

        }
        Action::Clear => {
            match fd_type {
                Type::Mined => {
                    let mut fds = FDs::mined();
                    fds.clear();
                }
                Type::New => {
                    let mut fds = FDs::new_fd();
                    fds.clear();
                }
                Type::Attributes => {
                    let mut attr = AttributesSink::new();
                    attr.clear();
                }
            }
        }
        Action::Display => {
            analysis_new_and_output(n, r);
        }

        Action::Current => {

            if let Type::Attributes = fd_type {
                let attribute = AttributesSink::new();
                println!("The current attributes is: {}", attribute.0);
            } else {
                let fds = match fd_type {
                    Type::Mined => {
                        FDs::mined()
                    }
                    Type::New => {
                        FDs::new_fd()
                    }
                    _ => {
                        panic!("Bad trace");
                    }
                };
    
                for fd in fds.0.iter() {
                    println!("{}", fd);
                }
            }

        }

        Action::Attributes => {
            let mut attribute_sink = AttributesSink::new();
            lazy_static!{
                static ref REGEX: Regex = Regex::new(
                    r##"([[:word:]]+)"##
                ).unwrap();
            }

            let mut attr_str = String::new();

            stdin().read_line(&mut attr_str).unwrap();

            let mut attri_vec = Vec::new();
            let mut cnt = 0;
            for cap in REGEX.captures_iter(attr_str.as_str()) {
                let attribute =  Attribute::new(cnt, cap[0].to_string());
                cnt += 1;
                attri_vec.push(attribute);
            }

            attribute_sink.0 = Attributes(attri_vec);
        }

        _ => {
            println!("Nothing to do");
        }
    }
}

fn save_mined_fd(fd: FD)  {
    let mut fds = FDs::mined();
    println!("Create FDs: {}", fds.1);
    fds.0.push(fd);
}

fn save_new_fd(fd: FD) {
    let mut fds = FDs::new_fd();
    fds.0.push(fd);
}

fn analysis_new_and_output(n: usize, r: f64) {
    let new_fd = FDs::new_fd();
    let mined = FDs::mined();
    
    new_fd.0.iter().for_each(|fd| {
        let N = fd.r_neighborhood_cnt(n, r);
        let NM = mined.0.iter().filter(|fd2| {
            log::info!("len of fd:{} to {} is :{}", fd, fd2, FD::distance_std(fd, fd2, n));
            FD::distance_std(fd, fd2, n) < r            
        }).collect::<Vec<_>>().len();

        println!("fd: {} has N: {}, MN: {}, error: %{}", fd, N, NM, (NM as f64) / (N as f64) * 100.0);
    });
}

pub fn analysis_algorithm_one_and_output(fd: &crate::dependency::fd::FunctionalDependency, n: usize) {
    let (determinant, dependant) = fd.disintegrate();
    let new_fd = FD::new_from_vec(determinant, dependant);
    analysis_one_and_output(&new_fd, n, 4.2);
}

fn analysis_one_and_output(fd: &FD, n: usize, r: f64) {
    let mined = FDs::mined();

    let N = fd.r_neighborhood_cnt(n, r);
    let NM = mined.0.iter().filter(|fd2| {
        log::info!("len of fd:{} to {} is :{}", fd, fd2, FD::distance_std(fd, fd2, n));
        FD::distance_std(fd, fd2, n) < r            
    }).collect::<Vec<_>>().len();
    
    println!("FD: {} has N: {}, MN: {}, error: %{}", fd,  N, NM, (NM as f64) / (N as f64) * 100.0);
}