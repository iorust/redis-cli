use std::io::{Result, Error, ErrorKind};
use std::net::{ToSocketAddrs};
use std::boxed::{Box};
use std::vec::Vec;

use super::resp::{Value, encode_slice};
use super::connection::{Connection, ConnectionSync};
use super::mio::{Poll, Token, EventSet, PollOpt};

pub fn create_client_sync(hostname: &str, port: u16, password: &str, db: u16) -> Result<ClientSync> {
    let mut client = try!(ClientSync::new((hostname, port)));
    try!(client.init(password, db));
    Ok(client)
}

pub struct ClientSync {
    conn: ConnectionSync,
}

impl ClientSync {
    pub fn new<T: ToSocketAddrs>(addrs: T) -> Result<Self> {
        let mut addr = try!(addrs.to_socket_addrs());
        Ok(ClientSync {
            conn: try!(ConnectionSync::new(&addr.next().unwrap())),
        })
    }

    pub fn cmd(&mut self, slice: &[&str]) -> Result<Value> {
        let buf = encode_slice(slice);
        try!(self.conn.write_cmd(&buf));
        self.conn.read_value()
    }

    pub fn read_value(&mut self) -> Result<Value> {
        self.conn.read_value()
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

pub fn create_client(hostname: &str, port: u16, password: &str, db: u16) -> Result<Client> {
    let mut client = try!(Client::new((hostname, port)));
    // try!(client.init(password, db));
    Ok(client)
}

pub struct Client {
    commands: Vec<u8>,
    callbacks: Vec<Box<Fn(Result<Value>)>>,
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

    fn check_async_io(&mut self) -> Result<()> {
        try!(self.poll.poll(Some(20000)));
        for item in self.poll.events() {
            let event_set = item.kind;
            if self.commands.len() > 0 && event_set.is_writable() {
                try!(self.conn.write_cmd(&self.commands));
                self.commands.clear();
            } else if self.callbacks.len() > 0 && event_set.is_readable() {
                try!(self.conn.consume_reply());
                if let Some(value) = self.conn.try_read_value() {
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

    pub fn cmd<F>(&mut self, slice: &[&str], callback: F) -> Result<()> where F: Fn(Result<Value>), F: Send + 'static {
        try!(self.register());
        self.commands.extend_from_slice(&encode_slice(slice));
        self.callbacks.push(Box::new(callback));
        try!(self.check_async_io());
        Ok(())
    }

    // fn init(&mut self, password: &str, db: u16) -> Result<()> {
    //     if password.len() > 0 {
    //         if let Value::Error(err)  = try!(self.cmd(&["auth", password])) {
    //             return Err(Error::new(ErrorKind::PermissionDenied, err));
    //         }
    //     }
    //     if db > 0 {
    //         if let Value::Error(err)  = try!(self.cmd(&["select", &db.to_string()])) {
    //             return Err(Error::new(ErrorKind::InvalidInput, err));
    //         }
    //     }
    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::resp::{Value};

    #[test]
    fn client_sync() {
        let mut client = ClientSync::new("127.0.0.1:6379").unwrap();

        println!("Start Sync");

        let value = client.cmd(&["set", "rust", "test_redis_cli"]);
        println!("Set result {:?}", value);
        assert_eq!(value.unwrap(), Value::String("OK".to_string()));

        let value = client.cmd(&["get", "rust"]);
        println!("Get result {:?}", value);
        assert_eq!(value.unwrap(), Value::Bulk("test_redis_cli".to_string()));

        let value = client.cmd(&["info"]);
        println!("Info result {:?}", value);
    }

    #[test]
    fn client() {
        let mut client = Client::new("127.0.0.1:6379").unwrap();

        println!("Start Async");

        client.cmd(&["set", "rust", "test_redis_cli"], |res| {
            println!("Async Set result {:?}", res);
            assert_eq!(res.unwrap(), Value::String("OK".to_string()));
        }).unwrap();

        client.cmd(&["get", "rust"], |res| {
            println!("Async Get result {:?}", res);
            assert_eq!(res.unwrap(), Value::Bulk("test_redis_cli".to_string()));
        }).unwrap();

        client.cmd(&["info"], |res| {
            println!("Async Info result {:?}", res);
        }).unwrap();
    }
}
