use std::net::{TcpStream, ToSocketAddrs};
use std::io::{BufReader, BufWriter};

use crate::{KvsError, Result};

use serde::Deserialize;
use serde_json::de::{IoRead, Deserializer};

// key value store client
pub struct KvsClient {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}


impl KvsClient {

    // connect to given address
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).unwrap();
        let reader = Deserializer::new(IoRead::new(BufReader::new(stream.try_clone()?)));
        let writer = BufWriter::new(stream);
        let client = KvsClient { reader, writer };
        Ok(client)
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<String> {
        Ok(String::new())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        Ok(())
    }
}


