use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::{KvsEngine, KvsError, Request, Response, Result};

use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};

// key value store client
pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }
    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            let stream = stream?;
            let reader = BufReader::new(stream.try_clone()?);
            let writer = BufWriter::new(stream);
            let mut deserializer = Deserializer::from_reader(reader);
            let request = Request::deserialize(&mut deserializer)?;
            let response = self.handle_request(request)?;
            serde_json::to_writer(writer, &response)?;
        }
        Ok(())
    }

    fn handle_request(&mut self, request: Request) -> Result<Response> {
        match request {
            Request::Get { key } => self.handle_get(key),
            Request::Set { key, value } => self.handle_set(key, value),
            Request::Remove { key } => self.handle_remove(key),
        }
    }
    fn handle_get(&mut self, key: String) -> Result<Response> {
        let value = self.engine.get(key)?;
        Ok(Response::Get { value })
    }
    fn handle_set(&mut self, key: String, value: String) -> Result<Response> {
        self.engine.set(key, value)?;
        Ok(Response::Set {})
    }
    fn handle_remove(&mut self, key: String) -> Result<Response> {
        self.engine.remove(key)?;
        Ok(Response::Remove {})
    }
}
