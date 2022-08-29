use clap::{AppSettings, Arg, Command};
use std::{env::current_dir, process::exit};

use kvs::{KvStore, KvsError, Result};

fn main() -> Result<()> {
    let matches = Command::new("kvs")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            Command::new("set")
                .about("set a value in the KVS")
                .arg(Arg::new("KEY").help("key to set").required(true))
                .arg(Arg::new("VALUE").help("value to set").required(true)),
        )
        .subcommand(
            Command::new("get")
                .about("get a value from the KVS")
                .arg(Arg::new("KEY").help("key to get").required(true)),
        )
        .subcommand(
            Command::new("rm")
                .about("remove a value from the KVS")
                .arg(Arg::new("KEY").help("key to remove").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("set", sub_m)) => {
            let key = sub_m.value_of("KEY").unwrap();
            let val = sub_m.value_of("VALUE").unwrap();
            let mut kvs = KvStore::open(current_dir()?)?;
            kvs.set(key.to_string(), val.to_string())?;
        }
        Some(("get", sub_m)) => {
            let key = sub_m.value_of("KEY").unwrap();
            let mut kvs = KvStore::open(current_dir()?)?;
            if let Some(res) = kvs.get(key.to_string())? {
                println!("{}", res);
            } else {
                println!("Key not found");
            }
        }
        Some(("rm", sub_m)) => {
            let key = sub_m.value_of("KEY").unwrap();
            let mut kvs = KvStore::open(current_dir()?)?;
            match kvs.remove(key.to_string()) {
                Ok(_) => {}
                Err(KvsError::KeyNotFound(_)) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e),
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
