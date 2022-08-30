use clap::{AppSettings, Arg, Command, arg_enum};
use log::info;
use std::{env::current_dir, process::exit, net::SocketAddr};
use structopt::StructOpt;

use kvs::*;

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
const DEFAULT_ENGINE: Engine = Engine::kvs;

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-client")]
struct Opt {
    #[structopt(
        long,
        help = "Sets the server address",
        value_name = "ADDRESS_FORMAT",
        default_value = "DEFAULT_LISTENING_ADDRESS",
        parse(try_from_str)
    )]
    addr: SocketAddr,

    #[structopt(
        long,
        help = "Sets the storage engine",
        value_name = "ENGINE-NAME",
        possible_values = &Engine::variants()
    )]
    engine: Option<Engine>,
}
arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum Engine {
        kvs,
        sled
    }
}
fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    info!("kvs-server: {}", env!("CARGO_PKG_VERSION"));
    info!("Listening of address: {}", opt.addr);
    info!("Storage engine: {}", opt.engine.unwrap_or(DEFAULT_ENGINE));
    Ok(())
}
