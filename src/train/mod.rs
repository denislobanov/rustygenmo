use clap::ArgMatches;

mod analyse;
mod data;

pub fn analyse_cmd(args: &ArgMatches) -> () {
    let file = args.value_of("file").unwrap();
    let data = match data::read_file(file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not open file {} for reading, the error was: {}", file, e);
            return;
        }
    };
    let first = match args.value_of("first") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => 0,
    };
    let last = match args.value_of("last") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => 0,
    };

    match args.subcommand_name() {
        Some("dump") => analyse::dump_cmd(&data),
        Some("words") => analyse::word_cmd(&data, first, last),
        Some("groups") => analyse::group_cmd(&data, first, last),
        _ => eprintln!("One of dump|words|groups must be chosen"),
    }
}

