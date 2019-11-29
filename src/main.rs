extern crate clap;

#[macro_use]
extern crate partial_application;

mod train;
mod generate;
mod crawl;

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

        // training
        .subcommand(App::new("train")
            .about("markov chain training")
            .arg(Arg::with_name("path")
                .short("p")
                .help("path to corpus")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("dbpath")
                .short("d")
                .help("path to db")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("count")
                .short("c")
                .help("number of groups to use")
                .takes_value(true)))

        // generation
        .subcommand(App::new("generate")
            .about("generate text")
            .arg(Arg::with_name("dbpath")
                .short("d")
                .help("path to db")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("length")
                .short("l")
                .help("how many words to generate")
                .takes_value(true)))

        // getting data
        .subcommand(App::new("crawl")
            .about("web crawler")
            .arg(Arg::with_name("fanfiction")
                .short("f")
                .help("crawl fanfiction")
                .takes_value(false)
                .conflicts_with("dailymail"))
            .arg(Arg::with_name("dailymail")
                .short("d")
                .help("crawl daily mail")
                .takes_value(false)
                .conflicts_with("fanfiction"))
            .arg(Arg::with_name("seed")
                .short("u")
                .help("seed url")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("path")
                .short("p")
                .help("path to store results in")
                .takes_value(true)
                .required(true)))

        .get_matches();

    match matches.subcommand() {
        ("analyse", Some(args)) => train::analyse_cmd(args),
        ("train", Some(args)) => train::train_cmd(args),
        ("generate", Some(args)) => generate::run_cmd(args),
        ("crawl", Some(args)) => crawl::crawl_cmd(args),
        _ => eprintln!("{}", matches.usage())
    }
}
