use crate::dependency::fd::*;
use crate::frontend::table::Attribute;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{BufReader, BufWriter, Read, Write, stdin};
use std::fs::{File, OpenOptions};

#[derive(Deserialize, Serialize)]
struct FDs(Vec<FunctionalDependency>, String);

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
            log::debug!("Save fd: {}", fd);
        }
        let s = serde_json::to_string(&self).unwrap();

        let filepath = self.1.as_str();

        log::debug!("Save: {}", s);

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
    Display,
    Single,
    Current,
    Nothing
}

pub enum Type {
    Mined,
    New
}

pub enum Output {
    Std,
}



pub fn analysis_cli(action: Action, fd_type: Type, n: usize, r: f64) {
    match action {

        Action::Add | Action::Single => {
            let mut fd_str = String::new();
            println!("Please input the fd as format of (name, index)* -> (name, index)");
            stdin().read_line(&mut fd_str).unwrap();

            log::info!("Get str {}", fd_str);

            lazy_static!{
                static ref REGEX: Regex = Regex::new(
                    r##"([0-9A-Za-z_.]+)([[:space:]]*,[[:space:]]*)([[:digit:]]+)"##
                ).unwrap();
            }

            let mut attr_vec = Vec::new();

            for cap in REGEX.captures_iter(fd_str.as_str()) {    
                let name = cap[1].to_string();
                let index = cap[3].to_string().parse::<usize>().unwrap();

                let attr = Attribute { value: name, rank: index };
                attr_vec.push(attr);
            }

            let fd = FunctionalDependency::new_from_vec(attr_vec);

            if let Action::Add = action {
                match fd_type {
                    Type::Mined => {
                        save_mined_fd(fd);
                    }
                    Type::New => {
                        save_new_fd(fd);
                    }
                }
            } else {
                analysis_one_and_output(&fd, n, r);
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
            }
        }
        Action::Display => {
            analysis_new_and_output(n, r);
        }

        Action::Current => {

            let fds = match fd_type {
                Type::Mined => {
                    FDs::mined()
                }
                Type::New => {
                    FDs::new_fd()
                }
            };

            for fd in fds.0.iter() {
                println!("{}", fd);
            }
        }

        _ => {
            println!("Nothing to do");
        }
    }
}

pub fn save_mined_fd(fd: FunctionalDependency)  {
    let mut fds = FDs::mined();
    println!("Create FDs: {}", fds.1);
    fds.0.push(fd);
}

pub fn save_new_fd(fd: FunctionalDependency) {
    let mut fds = FDs::new_fd();
    fds.0.push(fd);
}

pub fn analysis_new_and_output(n: usize, r: f64) {
    let new_fd = FDs::new_fd();
    let mined = FDs::mined();
    
    new_fd.0.iter().for_each(|fd| {
        let N = fd.r_neighborhood_cnt(n, r);
        let NM = mined.0.iter().filter(|fd2| {
            FunctionalDependency::distance_std(fd, fd2, n) < r            
        }).collect::<Vec<_>>().len();

        println!("fd: {} has error: %{}", fd, (NM as f64) / (N as f64) * 100.0);
    });
}

pub fn analysis_one_and_output(fd: &FunctionalDependency, n: usize, r: f64) {
    let mined = FDs::mined();

    let N = fd.r_neighborhood_cnt(n, r);
    let NM = mined.0.iter().filter(|fd2| {
        println!("len of fd:{} to {} is :{}", fd, fd2, FunctionalDependency::distance_std(fd, fd2, n));
        FunctionalDependency::distance_std(fd, fd2, n) < r            
    }).collect::<Vec<_>>().len();
    
    println!("N: {}, MN: {}", N, NM);

    println!("fd: {} has error: %{}", fd, (NM as f64) / (N as f64) * 100.0);

}