use clap::ArgMatches;

mod fanfiction;
mod store;

trait Crawler {
    fn crawl(&self, path: &str) -> ();
}

pub fn crawl_cmd(args: &ArgMatches) -> () {
    let url = args.value_of("seed").unwrap();
    let path = args.value_of("path").unwrap();

    if args.is_present("fanfiction") {
        println!("crawling fanfiction");
        let crawler = fanfiction::new();
        crawler.crawl(url);

    } else if args.is_present("dailymail") {
        panic!("dailymail");
    } else {
        panic!("you must choose one of [fanfiction|dailymail]");
    }
}