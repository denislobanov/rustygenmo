use std::sync::mpsc::channel;
use std::thread::spawn;

use clap::ArgMatches;

mod dailymail;
mod fanfiction;
mod store;

trait Crawler {
    fn crawl(&self, path: &str) -> ();
}

pub fn crawl_cmd(args: &ArgMatches) -> () {
    let url = args.value_of("seed").unwrap();
    let path = args.value_of("path").unwrap();
    let store = store::new(path);

    if args.is_present("fanfiction") {
        println!("crawling fanfiction");
        fanfiction::crawl(url, store);
    } else if args.is_present("dailymail") {
        println!("dailymail");

        // Start store thread
        let (msg_tx, msg_rx) = channel();
        let h = spawn(move || {
            store.run(msg_rx);
        });

        let crawler = dailymail::new(msg_tx);
        crawler.crawl(url);

        h.join();
    } else {
        panic!("you must choose one of [fanfiction|dailymail]");
    }
}