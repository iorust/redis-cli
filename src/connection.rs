use std::vec::Vec;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{BufReader, Result, ErrorKind};

use super::resp::{Value, encode_slice, Decoder};

pub struct Connection {
    tcp: TcpStream,
    de: Decoder,
}

impl Connection {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Self {
        Connection {
            tcp: TcpStream::connect(addr).unwrap(),
            de: Decoder::new(),
        }
    }

    pub fn request(&mut self, slice: &[&str]) -> Result<Value> {
        let buf = encode_slice(slice);
        let mut res = self.cmd(&buf, 1);
        match res {
            Ok(ref mut values) => Ok(values.remove(0)),
            Err(err) => Err(err),
        }
    }

    fn cmd(&mut self, buf: &[u8], expect_count: usize) -> Result<Vec<Value>> {
        try!(self.tcp.write(buf));
        let mut reader = BufReader::new(&mut self.tcp);
        let mut result: Vec<Value> = Vec::with_capacity(expect_count);
        loop {
            let consumed_len = {
                let buffer = match reader.fill_buf() {
                    Ok(buf) => buf,
                    Err(ref err) if err.kind() == ErrorKind::Interrupted => continue,
                    Err(err) => return Err(err),
                };

                if buffer.len() == 0 {
                    continue;
                }
                try!(self.de.feed(&buffer));
                buffer.len()
            };

            reader.consume(consumed_len);
            if let Some(value) = self.de.read() {
                result.push(value);
                if result.len() == expect_count {
                    return Ok(result);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_connection() {
        let mut connection = Connection::new("127.0.0.1:6379");
        let cmd1 = ["set", "rust", "test"];
        let cmd2 = ["get", "rust"];
        println!("{:?}", connection.request(&cmd1));
        println!("{:?}", connection.request(&cmd2));
    }
}
