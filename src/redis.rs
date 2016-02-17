use std::io::Result;
use std::net::{ToSocketAddrs};

use super::{Value};
use super::connection::{Connection};

pub fn create_client(hostname: &str, port: u16, password: &str, db: u16) -> Result<Client> {
    let mut client = Client::new((hostname, port));
    try!(client.init(password, db));
    Ok(client)
}

pub struct Client {
    conn: Connection,
}

impl Client {
    pub fn new<A: ToSocketAddrs>(addrs: A) -> Self {
        Client {
            conn: Connection::new(addrs),
        }
    }

    pub fn cmd(&mut self, slice: &[&str]) -> Result<Value> {
        self.conn.request(slice)
    }

    fn init(&mut self, password: &str, db: u16) -> Result<()> {
        if password.len() > 0 {
            try!(self.cmd(&["auth", password]));
        }
        if db > 0 {
            try!(self.cmd(&["select", &db.to_string()]));
        }
        Ok(())
    }
}
