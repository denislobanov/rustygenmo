extern crate clap;

use std::io;
use std::fs::File;
use std::collections::HashMap;
use std::io::Read;
use clap::{App, Arg};

fn read_file(path: &str) -> io::Result<String> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    Ok(s)
}

fn word_frequency(corpus: &str) -> HashMap<&str, u64> {
    let mut map = HashMap::new();
    let words: Vec<&str> = corpus.split_whitespace()
        .map(|w| w.trim().trim_matches(|c| c == '.' || c == ','))
        .collect();

    for word in words.into_iter() {
        if map.contains_key(&word) {
            *map.get_mut(&word).unwrap() += 1;
        } else {
            map.insert(word, 1);
        }
    }

    return map
}

fn main() {
    let matches = App::new("rustygenmo")
        .version("0.0.1")
        .author("Denis Lobanov")
        .about("nanogenmo 2019 entry")
        .arg(Arg::with_name("file")
            .short("f")
            .value_name("FILE")
            .help("The file to parse")
            .takes_value(true))
        .get_matches();

    if let Some(f) = matches.value_of("file") {
        let corpus = read_file(f);
        if let Ok(c) = corpus {
            let freq = word_frequency(c.as_str());

            let mut result: Vec<(&str, u64)> = Vec::new();
            freq.into_iter()
                .for_each(|e| result.push(e));

            // sort
            result.sort_by(|(_,v1), (_, v2)| v1.cmp(v2));
            result.reverse();
            result.iter()
                .for_each(|(k,v)| println!("[{}] {}", k, v));
        }
    } else {
        eprintln!("need to give a file")
    }
}
