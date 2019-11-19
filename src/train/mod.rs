use clap::ArgMatches;

mod analyse;
mod data;
mod train;

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
        Some("words") => analyse::print_kv(analyse::word_cmd(&data), first, last),
        Some("groups") => analyse::print_kv(analyse::group_cmd(&data), first, last),
        _ => eprintln!("One of dump|words|groups must be chosen"),
    }
}

pub fn train_cmd(args: &ArgMatches) -> () {
    let path = args.value_of("path").unwrap();
    let db_path = match args.value_of("dbpath") {
        Some(v) => v,
        None => "test",
    };
    let count = match args.value_of("count") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => 1,
    };

    let files = match data::list_dir(path) {
        Ok(fs) => fs,
        Err(e) => {
            eprintln!("couldnt open dir {} the error was: {}", path, e);
            return;
        }
    };

    // build a frequency map from all files
    let mut chain = train::new(db_path).unwrap();
    chain.groups(count, files)


    // build groups based on frequency map

    // We know how many markov chains we want to use (args), this will be the top n most common that
    // we found. Before we can train, we will need to build a lookup table for _word_ -> _group_

    // Now, we can train n markov chains simultaneously, deciding which one to put our words in
    // based on their group. Each group is a separate markov chain trained on the same corpus.
    // NB:
    //   we will need to keep a stack of the last x words, where x == largest group
}
