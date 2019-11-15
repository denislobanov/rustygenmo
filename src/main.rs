extern crate clap;

mod analyse;

use clap::{App, Arg};
use crate::analyse::analyse_cmd;

fn main() {
    let matches = App::new("rustygenmo")
        .version(clap::crate_version!())
        .author("Denis Lobanov")
        .about("nanogenmo 2019 entry")

        // corpus analysis tools
        .subcommand(App::new("analyse")
            .about("corpus analysis")
            .arg(Arg::with_name("file")
                .short("f")
                .help("Path to a file to analyse")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("dump")
                .short("d")
                .help("Print all unique words considered when parsing corpus"))
            .arg(Arg::with_name("words")
                .short("w")
                .help("Word frequency"))
            .arg(Arg::with_name("groups")
                .short("g")
                .help("Word frequency groups"))
            .arg(Arg::with_name("first")
                .short("a")
                .help("return first n items only")
                .takes_value(true))
            .arg(Arg::with_name("last")
                .short("b")
                .help("return last n items only")
                .takes_value(true)))
        .get_matches();

    match matches.subcommand() {
        ("analyse", Some(args)) => {
            analyse_cmd(args)
        }
        _ => eprintln!("{}", matches.usage())
    }
}
