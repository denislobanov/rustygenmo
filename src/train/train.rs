extern crate sled;

use sled::Db;
use std::path::PathBuf;
use crate::train::data::read_file;
use crate::train::analyse::create_corpus;
use std::convert::TryInto;
use std::str::from_utf8;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use self::sled::IVec;

pub struct Persistent {
    db: Db
}

impl Persistent {
    pub fn groups(&mut self, count: usize, files: Vec<PathBuf>) -> () {
        //TODO: calculating word frequency across corpus here, what if I do it per document?
        //  * grouping would still have to be done globally
        let t = self.db.open_tree("words").unwrap(); //todo - proper error handling

        for file in files {
            let data = read_file(file).unwrap(); //todo error handling

            // build map of word frequency
            create_corpus(&data).into_iter().for_each(|w| {
                t.update_and_fetch(w, inc).unwrap();
                return;
            });
        }

        // calculate groups
        let mut map: HashMap<u32, u32> = HashMap::new();

        t.iter().for_each(|x| {
            if x.is_err() {
                panic!("wtf {}", x.err().unwrap())
            }

            match x.ok() {
                Some((_, v)) => {
                    let a:[u8; 4] = v.iter().as_slice().try_into().unwrap();
                    let count = u32::from_be_bytes(a);
                    *map.entry(count).or_insert(0) += 1;
                },
                None => panic!("empty!"),
            }
        });

        // calculate top /count/ groups
        let mut gs: Vec<(u32, u32)> = Vec::new();
        map.into_iter().for_each(|e| gs.push(e));
        gs.sort_by(|(_, v1), (_, v2)| v2.cmp(v1));

        let mut tops: HashSet<u32> = HashSet::new();
        let mut last: u32 = 0;
        for (g, _) in gs.into_iter().take(count) {
            tops.insert(g);
            last = g;
        }

        // persist word -> group map for top _count_ groups
        //TODO: refactor to store words with group counts to avoid O(n2)
        let groups = self.db.open_tree("groups").unwrap();

        t.iter().for_each(|x| {
            // already checked for errors on previous loop :'(

            match x.ok() {
                Some((k, v)) => {
                    let key = from_utf8(k.borrow()).unwrap();
                    let a: [u8; 4] = v.iter().as_slice().try_into().unwrap();
                    let count = u32::from_be_bytes(a);

                    if tops.contains(&count) {
                        groups.insert(key, v).unwrap(); //todo err handling
                    } else {
                        groups.insert(key, u32_to_ivec(last)).unwrap();
                    }

                    return;
                },
                None => panic!("again with the empty!"),
            }
        });

        // we dont need the word frequency tree anymore
        self.db.drop_tree(b"words").unwrap();

        // print some data! (for now)
        print_tree(groups)
    }
}

pub fn new(db_path: &str) -> Result<Persistent, String> {
    return match Db::open(db_path) {
        Ok(d) => Ok(Persistent { db: d }),
        Err(e) => Err(e.to_string()),
    };
}

fn inc(v: Option<&[u8]>) -> Option<Vec<u8>> {
    let count = match v {
        Some(b) => {
            let a:[u8; 4] = b.try_into().unwrap();
            let n = u32::from_be_bytes(a);
            n + 1
        }
        None => 1,
    };

    Some(count.to_be_bytes().to_vec())
}

fn print_tree(t: sled::Tree) -> () {
    t.iter().for_each(|r| {
        if let Ok(e) = r {
            match e {
                (k, v) => {
                    let key = from_utf8(k.borrow()).unwrap();
                    let a: [u8; 4] = v.iter().as_slice().try_into().unwrap();
                    let value = u32::from_be_bytes(a);

                    println!("{} {}", key, value)
                }
            }
        }
    })
}

fn u32_to_ivec(x: u32) -> IVec {
    IVec::from(x.to_be_bytes().to_vec())
}
