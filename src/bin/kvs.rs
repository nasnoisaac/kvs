use clap::{AppSettings, Arg, Command};
use std::process::exit;

fn main() {
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
        Some(("set", _sub_m)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        Some(("get", _sub_m)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        Some(("rm", _sub_m)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        _ => unreachable!(),
    }
}
