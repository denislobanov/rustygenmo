use clap::ArgMatches;

mod fanfiction;

trait Crawler {
    fn crawl(self, path: String) -> ();
}

pub fn crawl_cmd(args: &ArgMatches) -> () {
    let url = args.value_of("url").unwrap();
    let seed = args.value_of("seed").unwrap();



    if args.is_present("fanfiction") {

    } else if args.is_present("dailymail") {

    } else {
        eprintln!("you must choose one of [fanfiction|dailymail]");
    }
}