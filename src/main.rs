extern crate clap;

mod train;

use clap::{App, Arg};

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
            .arg(Arg::with_name("first")
                .short("a")
                .help("return first n items only")
                .takes_value(true))
            .arg(Arg::with_name("last")
                .short("b")
                .help("return last n items only")
                .takes_value(true))
            .subcommand(App::new("dump")
                .about("Dump all unique words considered when parsing corpus"))
            .subcommand(App::new("words")
                .about("Word frequency"))
            .subcommand(App::new("groups")
                .about("Word frequency groups")))
        .get_matches();

    match matches.subcommand() {
        ("analyse", Some(args)) => {
            train::analyse_cmd(args)
        }
        _ => eprintln!("{}", matches.usage())
    }
}
