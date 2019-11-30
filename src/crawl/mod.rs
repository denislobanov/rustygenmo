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

    // Start store thread
    let (msg_tx, msg_rx) = channel();
    let (stop_tx, stop_rx) = channel();
    let h = spawn(move || {
        store.run(msg_rx, stop_rx);
    });

    if args.is_present("fanfiction") {
        println!("crawling fanfiction");
        fanfiction::crawl(url, msg_tx);

    } else if args.is_present("dailymail") {
        println!("dailymail");
        let crawler = dailymail::new(msg_tx);
        crawler.crawl(url);
    } else {
        panic!("you must choose one of [fanfiction|dailymail]");
    }

    h.join();
}