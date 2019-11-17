use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io;
use std::io::Read;

use clap::ArgMatches;
use std::fmt::Display;

fn read_file(file: &str) -> io::Result<String> {
    let mut s = String::new();
    File::open(file)?.read_to_string(&mut s)?;
    Ok(s)
}

fn create_corpus(corpus: &str) -> Vec<String> {
    return corpus
        // handle no whitespace after punctuation
        .split(|c: char| c.is_whitespace() || c == '.')
        // clean up words
        .map(clean_word)
        // remove empty items
        .filter(|w| w.len() > 1 || *w == "i")
        .collect();
}

fn clean_word(w: &str) -> String {
    return w.to_ascii_lowercase()
        .trim()
        .replace("''", "\"")
        .trim_matches(|c: char| !c.is_alphabetic())
        .to_string();
}

fn word_frequency(data: &str) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    let words: Vec<String> = create_corpus(data);

    for word in words.into_iter() {
        if map.contains_key(&word) {
            *map.get_mut(&word).unwrap() += 1;
        } else {
            map.insert(word.to_string(), 1);
        }
    }

    return map;
}

fn group_frequencies(freq: &HashMap<String, u64>) -> HashMap<u64, u64> {
    let mut map: HashMap<u64, u64> = HashMap::new();
    freq.iter().for_each(|(_, v)| *map.entry(*v).or_insert(0) += 1);

    return map;
}

pub fn analyse_cmd(args: &ArgMatches) -> () {
    let file = args.value_of("file").unwrap();
    let data = match read_file(file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not open file {} for reading, the error was: {}", file, e);
            return
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

    if args.is_present("dump") {
        dump(&data, first, last);
    } else if args.is_present("words") {
        word_cmd(&data, first, last);
    } else if args.is_present("groups") {
        group_cmd(&data, first, last)
    } else {
        eprintln!("you must choose dump|words|groups")
    }
}

fn dump(data: &str, _: usize, _: usize) -> () {
    let words = create_corpus(data);

    let mut set: HashSet<String> = HashSet::new();
    words.into_iter().for_each(|w| {
        set.insert(w);
        return ();
    });

    set.into_iter().for_each(|w| println!("[{}]", w))
}

fn word_cmd(data: &str, first: usize, last: usize) -> () {
    let freq = word_frequency(data);

    // scan map
    let mut result: Vec<(String, u64)> = Vec::new();
    freq.into_iter()
        .for_each(|e| result.push(e));

    result.sort_by(|(_, v1), (_, v2)| v2.cmp(v1));
    print_kv(result, first, last);
}

fn group_cmd(data: &str, first: usize, last: usize) -> () {
    let freq: HashMap<String, u64> = word_frequency(data);
    let groups: HashMap<u64, u64> = group_frequencies(&freq);

    let mut result: Vec<(u64, u64)> = Vec::new();
    groups.into_iter()
        .for_each(|e| result.push(e)); //TODO result::push

    result.sort_by(|(_, v1), (_, v2)| v2.cmp(v1));
    print_kv(result, first, last);
}

fn print_kv<K: Display, V: Display>(mut set: Vec<(K, V)>, first: usize, last: usize) -> () {
    if first == 0 && last == 0 {
        set.iter().for_each(|(k, v)| println!("{} {}", k, v))
    } else {
        if first != 0 {
            set.iter()
                .take(first)
                .for_each(|(k, v)| println!("[{}] {}", k, v));
            println!();
        }
        if last != 0 {
            set.reverse();
            set.iter()
                .take(first)
                .for_each(|(k, v)| println!("[{}] {}", k, v));
            println!();
        }
    }
}
