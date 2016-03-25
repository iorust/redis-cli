use std::io::prelude::*;
use std::io::{BufReader, Result, ErrorKind};
use std::net::{SocketAddr, TcpStream as TcpStreamSync};

use super::resp::{Value, Decoder};
use super::mio::tcp::{TcpStream};

pub struct ConnectionSync {
    stream: BufReader<TcpStreamSync>,
    decoder: Decoder,
}

impl ConnectionSync {
    pub fn new(addr: &SocketAddr) -> Result<Self> {
        let stream = try!(TcpStreamSync::connect(addr));
        Ok(ConnectionSync {
            decoder: Decoder::new(),
            stream: BufReader::new(stream),
        })
    }

    pub fn write_cmd(&mut self, buf: &[u8]) -> Result<()> {
        let stream = self.stream.get_mut() as &mut Write;
        stream.write_all(buf)
    }

    pub fn read_value(&mut self) -> Result<Value> {
        if let Some(value) = self.decoder.read() {
            return Ok(value);
        }
        loop {
            let consumed_len = {
                let buffer = match self.stream.fill_buf() {
                    Ok(buf) => buf,
                    Err(ref err) if err.kind() == ErrorKind::Interrupted => continue,
                    Err(err) => return Err(err),
                };

                if buffer.len() == 0 {
                    continue;
                }
                try!(self.decoder.feed(&buffer));
                buffer.len()
            };

            self.stream.consume(consumed_len);
            if let Some(value) = self.decoder.read() {
                return Ok(value);
            }
        }
    }
}

pub struct Connection {
    decoder: Decoder,
    stream: BufReader<TcpStream>,
}

impl Connection {
    pub fn new(addr: &SocketAddr) -> Result<Self> {
        let stream = try!(TcpStream::connect(addr));
        let _ = stream.set_nodelay(false);
        // stream.set_keepalive(120);
        Ok(Connection {
            decoder: Decoder::new(),
            stream: BufReader::new(stream),
        })
    }

    pub fn get_tcp_ref(&self) -> &TcpStream {
        self.stream.get_ref()
    }

    pub fn write_cmd(&mut self, buf: &[u8]) -> Result<()> {
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

    pub fn try_read_value(&mut self) -> Option<Value> {
        self.decoder.read()
    }
}
