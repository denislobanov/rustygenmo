use std::collections::{HashMap, HashSet};
use std::fmt::Display;

fn clean_word(w: &str) -> String {
    return w.to_ascii_lowercase()
        .trim()
        .replace("''", "\"")
        .trim_matches(|c: char| !c.is_alphabetic())
        .to_string();
}

pub fn create_corpus(corpus: &str) -> Vec<String> {
    return corpus
        // handle no whitespace after punctuation
        .split(|c: char| c.is_whitespace() || c == '.')
        // clean up words
        .map(clean_word)
        // remove empty items
        .filter(|w| w.len() > 1 || *w == "i")
        .collect();
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

pub fn print_kv<K: Display, V: Display>(mut set: Vec<(K, V)>, first: usize, last: usize) -> () {
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

pub fn dump_cmd(data: &str) -> () {
    let words = create_corpus(data);

    let mut set: HashSet<String> = HashSet::new();
    words.into_iter().for_each(|w| {
        set.insert(w);
        return ();
    });

    set.into_iter().for_each(|e| println!("[{}]", e))
}

pub fn word_cmd(data: &str) -> Vec<(String, u64)> {
    let freq = word_frequency(data);

    // scan map
    let mut result: Vec<(String, u64)> = Vec::new();
    freq.into_iter()
        .for_each(|e| result.push(e));

    result.sort_by(|(_, v1), (_, v2)| v2.cmp(v1));
    return result;
}

pub fn group_cmd(data: &str) -> Vec<(u64, u64)> {
    let freq: HashMap<String, u64> = word_frequency(data);
    let groups: HashMap<u64, u64> = group_frequencies(&freq);

    let mut result: Vec<(u64, u64)> = Vec::new();
    groups.into_iter()
        .for_each(|e| result.push(e)); //TODO result::push

    result.sort_by(|(_, v1), (_, v2)| v2.cmp(v1));
    return result;
}
