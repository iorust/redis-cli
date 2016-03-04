#![feature(test)]

extern crate test;
extern crate redis_cli;

use test::Bencher;
use redis_cli::{create_client, Client, Value};

fn prepare_client() -> Client {
    create_client("127.0.0.1", 6379, "", 0).expect("Failed to connect")
}

#[bench]
fn ping(b: &mut Bencher) {
    let mut client =  prepare_client();
    let command = ["ping"];
    b.iter(|| {
        for _ in 0..1000 {
            client.cmd(&command).unwrap();
        }
    });
}
