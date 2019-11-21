use clap::ArgMatches;

mod crawler;

pub fn crawl_cmd(args: &ArgMatches) -> () {
    let url = args.value_of("url").unwrap();


}