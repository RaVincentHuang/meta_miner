#![feature(iter_intersperse)]
use crate::dependency::analysis::{Action, Type, Output, analysis_cli};
use clap::{arg, command, value_parser, ArgAction, Command, ArgGroup, builder::ValueParser};

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
                .group(ArgGroup::new("actions")
                        .required(true)
                        .multiple(false)
                        .args(["add", "display", "clear", "single", "current"]))
                
                .arg(arg!(-t --type [TYPE] "fd type")
                        .value_parser(["mined", "new"])
                        .default_value("mined"))
                
                .arg(arg!(-o  --output [OUTPUT])
                        .value_parser(["std", "mdfile"])
                        .default_value("std"))

                .arg(arg!(-n --number [NUM]  "The number of attributes in the table")
                        .value_parser(value_parser!(usize)))
                .arg(arg!(-r --rate [RATE]  "The error rate")
                        .value_parser(value_parser!(f64)))


                
        ).get_matches();
    
    env_logger::init();

    
    match matches.subcommand() {
        Some(("analysis", sub_cmd)) => {
            let action = {
                if sub_cmd.get_flag("add") {
                    Action::Add
                } else if sub_cmd.get_flag("display") {
                    Action::Display
                } else if sub_cmd.get_flag("clear") {
                    Action::Clear
                } else if sub_cmd.get_flag("single") {
                    Action::Single
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
        _ => {
            log::warn!("Nothing to do")
        }
    }
    
    
}
