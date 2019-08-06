#![feature(test)]

extern crate redis_cli;
extern crate test;

use redis_cli::{create_client, Client};
use test::Bencher;

fn prepare_client() -> Client {
    create_client("127.0.0.1", 6379, "", 0).unwrap()
}

#[bench]
fn ping(b: &mut Bencher) {
    let mut client = prepare_client();
    let command = ["ping"];
    b.iter(|| {
        for _ in 0..1000 {
            client.cmd(&command).unwrap();
        }
    });
}
