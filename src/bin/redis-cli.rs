#[macro_use]
extern crate clap;
extern crate redis_cli;

use std::io;
use std::io::prelude::*;
use std::str::FromStr;
use clap::{Arg, App};

use redis_cli::{create_client};

fn main() {
    let matches = App::new("redis-cli")
        .version("0.2.2")
        .author("Qing Yan <admin@zensh.com>")
        .arg(Arg::with_name("hostname")
            .short("h")
            .long("hostname")
            .help("Server hostname (default: 127.0.0.1).")
            .required(false)
            .takes_value(true))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .help("Server port (default: 6379).")
            .required(false)
            .takes_value(true))
        .arg(Arg::with_name("password")
            .short("a")
            .long("password")
            .help("Password to use when connecting to the server.")
            .required(false)
            .takes_value(true))
        .arg(Arg::with_name("db")
            .short("n")
            .long("db")
            .help("Database number.")
            .required(false)
            .takes_value(true))
        .arg(Arg::with_name("command")
            .help("command...")
            .required(false)
            .index(1))
        .get_matches();

    let mut db: u16 = 0;
    let mut port: u16 = 6379;
    let mut password = "";
    let mut hostname = "127.0.0.1";

    if let Some(_db) = matches.value_of("db") {
        db = u16::from_str(_db).expect("Failed to read db");
    }
    if let Some(_port) = matches.value_of("port") {
        port = u16::from_str(_port).expect("Failed to read port");
    }
    if let Some(_password) = matches.value_of("password") {
        password = _password;
    }
    if let Some(_hostname) = matches.value_of("hostname") {
        hostname = _hostname;
    }

    let mut client = create_client(hostname, port, password, db).expect("Failed to connect");
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        let mut input = String::new();
        stdin.read_line(&mut input).expect("Failed to read command");

        let commands: Vec<&str> = input.split_whitespace().collect();
        if commands.len() == 0 {
            continue;
        }
        let command: &str = &commands[0].to_uppercase();
        match client.cmd(&commands) {
            Ok(value) => {
                let mut reply = value.to_beautify_string();
                match command {
                    "INFO" => {
                        // remove first and last '"'
                        reply.remove(0);
                        reply.pop();
                        writeln!(stdout, "{}", reply.trim()).unwrap();
                    }

                    "MONITOR" | "PSUBSCRIBE" | "PUNSUBSCRIBE" | "SUBSCRIBE" | "UNSUBSCRIBE" => {
                        writeln!(stdout, "{}", "Reading messages... (press Ctrl-C to quit)").unwrap();
                        writeln!(stdout, "{}", reply).unwrap();

                        loop {
                            let reply = client.read_more().unwrap().to_beautify_string();
                            writeln!(stdout, "{}", reply).unwrap();
                        }
                    }

                    _ => {
                        writeln!(stdout, "{}", reply).unwrap();
                    }
                }
            }

            Err(err) => {
                writeln!(stderr, "{:?}", err).unwrap();
            }
        };

    }
}
