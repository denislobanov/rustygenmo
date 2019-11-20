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
        let mut freq: HashMap<String, u32> = HashMap::new();

        for file in files {
            let data = read_file(file).unwrap(); //todo proper error handling

            // build map of word frequency
            create_corpus(&data).into_iter()
                .for_each(|w| *freq.entry(w).or_insert(0) += 1);
        }

        // calculate groups
        let mut gmap: HashMap<u32, (u32, HashSet<String>)> = HashMap::new();
        freq.into_iter().for_each(|(k, v)| {
            match gmap.get_mut(&v) {
                Some((g, s)) => {
                    *g += 1;
                    s.insert(k);
                }
                None => {
                    let mut s: HashSet<String> = HashSet::new();
                    s.insert(k);

                    gmap.insert(v, (1, s));
                }
            }
        });

        // calculate top /count/ groups
        let mut gvec: Vec<(u32, (u32, HashSet<String>))> = Vec::new();
        gmap.into_iter().for_each(|e| gvec.push(e));
        gvec.sort_by(|(_, (c1, _)), (_, (c2, _))| c2.cmp(c1));

        let mut last: u32 = 0;

        // persist word -> group map for top _count_ groups
        let groups = self.db.open_tree("groups").unwrap();
        for (i, (g, (_, s))) in gvec.into_iter().enumerate() {
            if i < count {
                s.into_iter().for_each(|w| {
                    //todo proper error handling
                    groups.insert(w, u32_to_ivec(g)).unwrap();
                    return;
                });
                last = g
            } else {
                s.into_iter().for_each(|w| {
                    groups.insert(w, u32_to_ivec(last)).unwrap();
                })
            }
        }

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

//fn inc(v: Option<&[u8]>) -> Option<Vec<u8>> {
//    let count = match v {
//        Some(b) => {
//            let a:[u8; 4] = b.try_into().unwrap();
//            let n = u32::from_be_bytes(a);
//            n + 1
//        }
//        None => 1,
//    };
//
//    Some(count.to_be_bytes().to_vec())
//}

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
