use std::io::{Result, Error, ErrorKind};
use std::net::{ToSocketAddrs};
use std::boxed::{Box, FnBox};
use std::vec::Vec;

use super::resp::{Value, encode_slice};
use super::connection::{Connection};
use super::mio::{Poll, Token, EventSet, PollOpt};

pub fn create_client(hostname: &str, port: u16, password: &str, db: u16) -> Result<Client> {
    let mut client = try!(Client::new((hostname, port)));
    try!(client.init(password, db));
    Ok(client)
}

pub struct Client {
    commands: Vec<u8>,
    callbacks: Vec<Box<FnBox(Result<Value>)>>,
    poll: Poll,
    conn: Connection,
}

impl Client {
    pub fn new<T: ToSocketAddrs>(addrs: T) -> Result<Self> {
        let mut addr = try!(addrs.to_socket_addrs());
        Ok(Client {
            commands: Vec::new(),
            callbacks: Vec::new(),
            poll: try!(Poll::new()),
            conn: try!(Connection::new(&addr.next().unwrap())),
        })
    }

    fn register(&mut self) -> Result<()> {
        self.poll.reregister(self.conn.get_tcp_ref(), Token(0), EventSet::all(), PollOpt::edge())
    }

    fn check_sync_io(&mut self, event: EventSet) -> Result<()> {
        loop {
            try!(self.poll.poll(Some(20000)));
            for item in self.poll.events() {
                let event_set = item.kind;
                if event_set.contains(event) {
                    return Ok(());
                } else if event_set.is_error() {
                    return Err(Error::new(ErrorKind::Other, "Some error"))
                }
            }
        }
    }

    fn check_async_io(&mut self) -> Result<()> {
        try!(self.poll.poll(Some(20000)));
        for item in self.poll.events() {
            let event_set = item.kind;
            if self.commands.len() > 0 && event_set.is_writable() {
                try!(self.conn.write_command(&self.commands));
                self.commands.clear();
            } else if self.callbacks.len() > 0 && event_set.is_readable() {
                try!(self.conn.consume_reply());
                if let Some(value) = self.conn.read_value() {
                    let callback = self.callbacks.remove(0);
                    callback(Ok(value));
                }
            } else if event_set.is_error() {
                self.commands.clear();
                while self.callbacks.len() > 0 {
                    let callback = self.callbacks.remove(0);
                    callback(Err(Error::new(ErrorKind::Other, "Some error")));
                }
            }
        }

        if self.commands.len() > 0 || self.callbacks.len() > 0 {
            self.check_async_io()
        } else {
            Ok(())
        }
    }

    pub fn cmd(&mut self, slice: &[&str]) -> Result<Value> {
        try!(self.register());
        let buf = encode_slice(slice);
        let _ = try!(self.check_sync_io(EventSet::writable()));
        try!(self.conn.write_command(&buf));
        self.read_value()
    }

    pub fn cmd_async<F>(&mut self, slice: &[&str], callback: F) ->
    Result<()> where F: FnBox(Result<Value>), F: Send + 'static {
        try!(self.register());
        self.commands.extend_from_slice(&encode_slice(slice));
        self.callbacks.push(Box::new(callback));
        try!(self.check_async_io());
        Ok(())
    }

    pub fn read_value(&mut self) -> Result<Value> {
        if let Some(value) = self.conn.read_value() {
            return Ok(value);
        }
        loop {
            try!(self.check_sync_io(EventSet::readable()));
            try!(self.conn.consume_reply());
            if let Some(value) = self.conn.read_value() {
                return Ok(value);
            }
        }
    }

    fn init(&mut self, password: &str, db: u16) -> Result<()> {
        if password.len() > 0 {
            if let Value::Error(err)  = try!(self.cmd(&["auth", password])) {
                return Err(Error::new(ErrorKind::PermissionDenied, err));
            }
        }
        if db > 0 {
            if let Value::Error(err)  = try!(self.cmd(&["select", &db.to_string()])) {
                return Err(Error::new(ErrorKind::InvalidInput, err));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_connection() {
        let mut client = Client::new("127.0.0.1:6379").unwrap();

        println!("Start");
        let v = client.cmd(&["info"]).unwrap();
        println!("Info result {:?}", v);
        let v = client.cmd(&["set", "rust", "test_redis_cli"]).unwrap();
        println!("Set result {:?}", v);
        let v = client.cmd(&["get", "rust"]).unwrap();
        println!("Get result {:?}", v);

        client.cmd_async(&["info"], |res| {
            println!("Async Info result {:?}", res);
        }).unwrap();
        // assert_eq!(connection.read().unwrap(), Value::String("OK".to_string()));

        // connection.write(&encode_slice(&["get", "rust"])).unwrap();
        // assert_eq!(connection.read().unwrap(), Value::Bulk("test_redis_cli".to_string()));
        //
        // connection.write(&encode_slice(&["set", "rust", "test_redis_cli_2"])).unwrap();
        // connection.write(&encode_slice(&["get", "rust"])).unwrap();
        // assert_eq!(connection.read().unwrap(), Value::String("OK".to_string()));
        // assert_eq!(connection.read().unwrap(), Value::Bulk("test_redis_cli_2".to_string()));
    }
}
