extern crate resp;

pub use resp::{Value, encode_slice, Decoder};
pub use redis::{create_client, Client};
pub use command::COMMANDS;

mod command;
mod connection;
mod redis;
