#![feature(iter_intersperse)]
use std::collections::HashSet;

use crate::dependency::analysis::{Action, Type, Output, analysis_cli};
use crate::algorithm::cluster;
use crate::algorithm::tane::Tane;
use crate::frontend::parser;
use crate::frontend::table::Table;
use algorithm::Algorithm;
use clap::{arg, command, value_parser, ArgAction, Command, ArgGroup, builder::ValueParser};
use serde::{Deserialize,Serialize};

mod frontend;
mod algorithm;
mod dependency;

fn main() {
    let matches = command!()
        .subcommand(
            command!("analysis")
                .arg(arg!(-a --add <ADD> "Add the fds")
                        .action(ArgAction::SetTrue)
                        .requires("type"))
                .arg(arg!(-l --load <LOAD> "load from file")
                        .action(ArgAction::SetTrue)
                        .requires("type"))
                .arg(arg!(-d --display <DISPLAY> "Display the analysis of all the new fds")
                        .action(ArgAction::SetTrue)
                        .requires("number")
                        .requires("rate"))
                .arg(arg!(-c --clear <CLEAR> "Clear the current fds")
                        .action(ArgAction::SetTrue)
                        .requires("type"))
                .arg(arg!(-s --single <SINGLE> "Analysis single fd")
                        .requires("number")
                        .requires("rate")
                        .action(ArgAction::SetTrue))
                .arg(arg!(--current <CURRENT> "Current display")
                        .action(ArgAction::SetTrue)
                        .requires("type"))
                .arg(arg!(--attributes <Attributes>)
                        .action(ArgAction::SetTrue))
                .group(ArgGroup::new("actions")
                        .required(true)
                        .multiple(false)
                        .args(["add", "display", "clear", "single", "current", "attributes", "load"]))
                
                .arg(arg!(-t --type [TYPE] "fd type")
                        .value_parser(["mined", "new", "attributes"])
                        .default_value("mined"))
                
                .arg(arg!(-o  --output [OUTPUT])
                        .value_parser(["std", "mdfile"])
                        .default_value("std"))

                .arg(arg!(-n --number [NUM]  "The number of attributes in the table")
                        .value_parser(value_parser!(usize)))
                .arg(arg!(-r --rate [RATE]  "The error rate")
                        .value_parser(value_parser!(f64)))      
        ).subcommand(
            command!("execute")
                .arg(arg!(-i --input <INPUT> "Input files")
                    .value_parser(value_parser!(String))
                    .action(ArgAction::Set))
                    
                
        ).get_matches();
    
    env_logger::init();

    
    match matches.subcommand() {
        Some(("analysis", sub_cmd)) => {
            let action = {
                if sub_cmd.get_flag("add") {
                    Action::Add
                } else if sub_cmd.get_flag("load") {
                    Action::Load
                } else if sub_cmd.get_flag("display") {
                    Action::Display
                } else if sub_cmd.get_flag("clear") {
                    Action::Clear
                } else if sub_cmd.get_flag("single") {
                    Action::Single
                } else if sub_cmd.get_flag("attributes") {
                    Action::Attributes
                } else if sub_cmd.get_flag("current") {
                    Action::Current
                } 
                else {
                    Action::Nothing
                }
            };

            let fd_type = if let Some(matches) = sub_cmd.get_one::<String>("type") {
                    match matches.as_str() {
                        "mined" => Type::Mined,
                        "new" => Type::New,
                        "attributes" => Type::Attributes,
                        _ => unreachable!()
                    }
                } else {
                   unreachable!()
                };

            let n = if let Some(n) = sub_cmd.get_one::<usize>("number") {
                *n
            } else {
                0
            };

            let r = if let Some(r) = sub_cmd.get_one::<f64>("rate") {
                *r
            } else {
                0.0
            };

            analysis_cli(action, fd_type, n, r);

        }
        Some(("execute", sub_cmd)) => {
            if let Some(path) = sub_cmd.get_one::<String>("input") {
                let table = parser::load_from_file(path).unwrap();
                println!("{}", table);
                let nodes = cluster::clustering(&table);

                let mut tane = Tane::new();
                for node in nodes {
                    let sub_table = table.sub_table(&node);
                    let res = tane.execute(&sub_table);
                    println!("sub table of {}", sub_table);
                    res.display();
                }
                // let res = tane.execute(table);
                // res.display();
            }
        }
        _ => {
            log::warn!("Nothing to do")
        }
    }
    
    
}