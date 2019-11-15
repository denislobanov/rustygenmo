extern crate clap;

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Read;

use clap::{App, Arg};

fn read_file(path: &str) -> io::Result<String> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    Ok(s)
}

fn create_corpus(corpus: &str) -> Vec<String> {
    return corpus
        // handle no whitespace after punctuation
        .split(|c: char| c.is_whitespace() || c == '.')
        // clean up words
        .map(clean_word)
        // remove empty items
        .filter(|w| w.len()>1 || *w == "i")
        .collect();
}

fn clean_word(w: &str) -> String {
    return w.to_ascii_lowercase()
        .trim()
        .replace("''", "\"")
        .trim_matches(|c: char| !c.is_alphabetic())
        .to_string()
}

fn word_frequency(corpus: &str) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    let words: Vec<String> = create_corpus(corpus);

    for word in words.into_iter() {
        if map.contains_key(&word) {
            *map.get_mut(&word).unwrap() += 1;
        } else {
            map.insert(word.to_string(), 1);
        }
    }

    return map
}

fn group_frequencies(freq: &HashMap<String, u64>) -> HashMap<u64, u64> {
    let mut map: HashMap<u64, u64> = HashMap::new();
    freq.iter().for_each(|(_, v)| *map.entry(*v).or_insert(0) += 1);

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
            let freq: HashMap<String, u64> = word_frequency(c.as_str());

            let groups: HashMap<u64, u64> = group_frequencies(&freq);

            let mut result: Vec<(String, u64)> = Vec::new();
            freq.into_iter()
                .for_each(|e| result.push(e));

            // sort
            result.sort_by(|(_,v1), (_, v2)| v2.cmp(v1));
            result.iter()
                .for_each(|(k, v)| println!("{} {}", k, v));

            println!("\n5 most common:");
            result.iter()
                .take(5)
                .for_each(|(k,v)| println!("[{}] {}", k, v));

            println!("\n5 least common:");
            result.reverse();
            result.iter()
                .take(5)
                .for_each(|(k,v)| println!("[{}] {}", k, v));

            println!("\ntop 5 groups");
            let mut gr: Vec<(u64, u64)> = Vec::new();
            groups.into_iter().for_each(|e| gr.push(e));
            gr.sort_by(|(_, v1), (_, v2)| v2.cmp(v1));
            gr.iter()
                .take(5)
                .for_each(|(k, v)| println!("{} -> {}", k, v))
        }
    } else {
        eprintln!("need to give a file")
    }
}
