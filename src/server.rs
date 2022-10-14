use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::{
    GetResponse, KvsEngine, RemoveResponse, Request, Response, Result, SetResponse, ThreadPool,
};

use log::{debug, error};
use serde::Deserialize;
use serde_json::de::Deserializer;

// key value store client
pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    engine: E,
    pool: P,
}

impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    pub fn new(engine: E, pool: P) -> Self {
        KvsServer { engine, pool }
    }
    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            let engine = self.engine.clone();
            self.pool.spawn(move || match stream {
                Ok(stream) => {
                    if let Err(e) = serve(engine, stream) {
                        error!("error serving client: {}", e);
                    }
                }
                Err(e) => {
                    error!("connection failed: {}", e);
                }
            });
        }
        Ok(())
    }
}

fn serve<E: KvsEngine>(engine: E, stream: TcpStream) -> Result<()> {
    let peer = stream.peer_addr()?;
    let reader = BufReader::new(stream.try_clone()?);
    let writer = BufWriter::new(stream);
    let mut deserializer = Deserializer::from_reader(reader);
    let request = Request::deserialize(&mut deserializer)?;
    debug!("Receive request from {}: {:?}", peer, request);
    let response = handle_request(engine, request);
    debug!("Send response back to {}: {:?}", peer, response);
    serde_json::to_writer(writer, &response)?;
    Ok(())
}

fn handle_request<E: KvsEngine>(engine: E, request: Request) -> Response {
    match request {
        Request::Get { key } => Response::Get(handle_get(engine, key)),
        Request::Set { key, value } => Response::Set(handle_set(engine, key, value)),
        Request::Remove { key } => Response::Remove(handle_remove(engine, key)),
    }
}
fn handle_get<E: KvsEngine>(engine: E, key: String) -> GetResponse {
    match engine.get(key) {
        Ok(value) => GetResponse::Ok(value),
        Err(e) => GetResponse::Err(e.to_string()),
    }
}
fn handle_set<E: KvsEngine>(engine: E, key: String, value: String) -> SetResponse {
    match engine.set(key, value) {
        Ok(_) => SetResponse::Ok(()),
        Err(e) => SetResponse::Err(e.to_string()),
    }
}
fn handle_remove<E: KvsEngine>(engine: E, key: String) -> RemoveResponse {
    match engine.remove(key) {
        Ok(_) => RemoveResponse::Ok(()),
        Err(e) => RemoveResponse::Err(e.to_string()),
    }
}
