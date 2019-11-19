extern crate sled;

use sled::Db;
use std::path::PathBuf;
use crate::train::data::read_file;
use crate::train::analyse::create_corpus;
use std::convert::TryInto;
use std::str::from_utf8;
use std::borrow::Borrow;
use std::collections::HashMap;

pub struct Persistent {
    db: Db
}

impl Persistent {
    pub fn groups(&mut self, _count: usize, files: Vec<PathBuf>) -> () {
        let t = self.db.open_tree("groups").unwrap(); //todo

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

        t.iter().into_iter().for_each(|x| {
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

        // print some data! (for now)
        map.iter().for_each(|(k, v)| println!("{} {}", *k, *v))
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
