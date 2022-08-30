use clap::{AppSettings, Arg, Command};
use std::{env::current_dir, process::exit, net::SocketAddr};
use structopt::StructOpt;

use kvs::{KvStore, KvsError, Result};

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-client")]
struct Opt {
    #[structopt(subcommand)]
    command: Subcommands,
}

#[derive(Debug, StructOpt)]
enum Subcommands {
    #[structopt(name = "get", about = "Get the string value of a given string key")]
    Get {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
        #[structopt(
            long,
            help = "Sets the server address",
            value_name = "ADDRESS_FORMAT",
            default_value = "DEFAULT_LISTENING_ADDRESS",
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
    #[structopt(name = "set", about = "Set the value of a given key")]
    Set {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
        #[structopt(name = "VALUE", help = "The string value of the key")]
        value: String,
        #[structopt(
            long,
            help = "Sets the server address",
            value_name = "ADDRESS_FORMAT",
            default_value = "DEFAULT_LISTENING_ADDRESS",
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
    #[structopt(name = "rm", about = "Remove the string value of a given string key")]
    Rm {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
        #[structopt(
            long,
            help = "Sets the server address",
            value_name = "ADDRESS_FORMAT",
            default_value = "DEFAULT_LISTENING_ADDRESS",
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    match opt.command {
        Subcommands::Get { key, addr } => {
            let mut kvs = KvStore::open(current_dir()?)?;
            if let Some(res) = kvs.get(key)? {
                println!("{}", res);
            } else {
                println!("Key not found");
            }
        }
        Subcommands::Set { key, value, addr } => {
            let mut kvs = KvStore::open(current_dir()?)?;
            kvs.set(key, value)?;
        }
        Subcommands::Rm { key, addr } => {
            let mut kvs = KvStore::open(current_dir()?)?;
            match kvs.remove(key) {
                Ok(_) => {}
                Err(KvsError::KeyNotFound(_)) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e),
            }
        }

    }
    Ok(())
}
