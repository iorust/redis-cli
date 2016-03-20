use std::io::prelude::*;
use std::io::{BufReader, Result};
use std::net::SocketAddr;

use super::resp::{Value, Decoder};
use super::mio::tcp::{TcpStream};

pub struct Connection {
    decoder: Decoder,
    stream: BufReader<TcpStream>,
}

impl Connection {
    pub fn new(addr: &SocketAddr) -> Result<Self> {
        let tcp = try!(TcpStream::connect(addr));
        let _ = tcp.set_nodelay(false);
        // tcp.set_keepalive(120);
        Ok(Connection {
            decoder: Decoder::new(),
            stream: BufReader::new(tcp),
        })
    }

    pub fn get_tcp_ref(&self) -> &TcpStream {
        self.stream.get_ref()
    }

    pub fn write_command(&mut self, buf: &[u8]) -> Result<()> {
        let stream = self.stream.get_mut() as &mut Write;
        stream.write_all(buf)
    }

    pub fn consume_reply(&mut self) -> Result<()> {
        let consumed_len = {
            let buffer = match self.stream.fill_buf() {
                Ok(buf) => buf,
                Err(err) => return Err(err),
            };

            if buffer.len() > 0 {
                try!(self.decoder.feed(&buffer));
            }
            buffer.len()
        };
        self.stream.consume(consumed_len);
        Ok(())
    }

    pub fn read_value(&mut self) -> Option<Value> {
        self.decoder.read()
    }
}
