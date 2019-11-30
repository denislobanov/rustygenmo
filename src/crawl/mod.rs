use clap::ArgMatches;

mod pool;
mod dailymail;
mod fanfiction;
mod store;

pub fn crawl_cmd(args: &ArgMatches) -> () {
    let url = args.value_of("seed").unwrap();
    let path = args.value_of("path").unwrap();
    let store = store::new(path);

    if args.is_present("fanfiction") {
        fanfiction::crawl(url, store);
    } else if args.is_present("dailymail") {
        dailymail::crawl(url, store);
    } else {
        panic!("you must choose one of [fanfiction|dailymail]");
    }
}