use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::{KvsEngine, KvsError, Request, Result, SetResponse, GetResponse, RemoveResponse, Response};

use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};
use log::{debug, error, info};

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
            let peer = stream.peer_addr()?;
            let reader = BufReader::new(stream.try_clone()?);
            let writer = BufWriter::new(stream);
            let mut deserializer = Deserializer::from_reader(reader);
            let request = Request::deserialize(&mut deserializer)?;
            debug!("Receive request from {}: {:?}", peer, request);
            let response = self.handle_request(request);
            serde_json::to_writer(writer, &response)?;
        }
        Ok(())
    }

    fn handle_request(&mut self, request: Request) -> Response {
        match request {
            Request::Get { key } => Response::Get(self.handle_get(key)),
            Request::Set { key, value } => Response::Set(self.handle_set(key, value)),
            Request::Remove { key } => Response::Remove(self.handle_remove(key)),
        }
    }
    fn handle_get(&mut self, key: String) -> GetResponse {
        match self.engine.get(key){
            Ok(value) => GetResponse::Ok(Some(value)),
            Err(e) => GetResponse::Err(e.to_string()),
        }
    }
    fn handle_set(&mut self, key: String, value: String) -> SetResponse {
        match self.engine.set(key, value){
            Ok(_) => SetResponse::Ok(()),
            Err(e) => SetResponse::Err(e.to_string()),
        }
    }
    fn handle_remove(&mut self, key: String) -> RemoveResponse {
        match self.engine.remove(key) {
            Ok(_) => RemoveResponse::Ok(()),
            Err(e) => RemoveResponse::Err(e.to_string()),
        }
    }
}
