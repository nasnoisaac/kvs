use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream, ToSocketAddrs};

use crate::{GetResponse, KvsError, RemoveResponse, Request, Response, Result, SetResponse};

use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};

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
        let req = Request::Set { key, value };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = Response::deserialize(&mut self.reader)?;
        match resp {
            Response::Set(SetResponse::Ok(_)) => Ok(()),
            Response::Set(SetResponse::Err(e)) => Err(KvsError::StringError(e)),
            _ => Err(KvsError::UnexpectedCommandType),
        }
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let req = Request::Get { key };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = Response::deserialize(&mut self.reader)?;
        match resp {
            Response::Get(GetResponse::Ok(value)) => Ok(value),
            Response::Get(GetResponse::Err(e)) => Err(KvsError::StringError(e)),
            _ => Err(KvsError::UnexpectedCommandType),
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        let req = Request::Remove { key };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = Response::deserialize(&mut self.reader)?;
        match resp {
            Response::Remove(RemoveResponse::Ok(_)) => Ok(()),
            Response::Remove(RemoveResponse::Err(e)) => Err(KvsError::StringError(e)),
            _ => Err(KvsError::UnexpectedCommandType),
        }
    }
}
