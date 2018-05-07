use database;
use std::process;

pub fn check_args(args: Vec<String>) {
    match args[1].as_ref() {
        "init" => init(),
        "help" => help(),
        _ => help(),
    }
}

fn init() {
    println!("Initializing database!");
    database::init();
}

fn help() {
    println!("Use the 'init' command to initialize a database!");
    process::exit(1)
}
