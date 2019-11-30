extern crate sled;

use sled::Db;
use std::path::PathBuf;
use crate::train::data::read_file;
use crate::train::analyse::create_corpus;
use std::convert::TryInto;
use std::collections::{HashMap, HashSet};
use self::sled::{IVec, Tree};

pub struct Persistent {
    db: Db
}

impl Persistent {
    pub fn groups(&mut self, count: usize, files: &Vec<PathBuf>) -> () {
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
    }

    // Now, we can train n markov chains simultaneously, deciding which one to put our words in
    // based on their group. Each group is a separate markov chain trained on the same corpus.
    // NB:
    //   we will need to keep a stack of the last x words, where x == largest group
    pub fn train(self, files: &Vec<PathBuf>) -> () {
        let groups = self.db.open_tree("groups").unwrap();

        // create m * n-grams
        let mut chains: HashMap<u32, Tree> = HashMap::new();
        groups.iter()
            .fold(HashSet::new(), |mut s, v| {
                let (_, g) = v.unwrap();
                s.insert(ivec_to_u32(g));
                return s;
            })
            .into_iter().for_each(|g| {
            chains.insert(g, self.db.open_tree(u32_to_ivec(g)).unwrap());
        });

        for file in files {
            let data = read_file(file).unwrap();
            let words = create_corpus(&data);

            for (i, w) in words.iter().enumerate() {
                //NB: we have parsed this corpus before so |w| should exist, but probably this can
                // be cleaner
                let g: u32 = ivec_to_u32(groups.get(w).unwrap().unwrap());

                if i + g as usize >= words.len() {
                    //todo consider any remaining words - these might be g=1
                    break;
                }

                //finally at the crux of all the above logic: group # is the n in n-gram is the key size
                let key: Vec<String> = words[i..i + g as usize].to_vec();
                let skey = bincode::serialize(&key).unwrap();

                chains.get(&g).unwrap()
                    .update_and_fetch(skey, partial_application::partial!(add_to_chain, words[i+g as usize].to_string(), _))
                    .unwrap();
            }
        }

        chains.iter().for_each(|(k, v)| {
            println!("{}\n------------------", k);
            v.iter().for_each(|r| {
                let (k2, v2) = r.unwrap();
                let key: Vec<String> = bincode::deserialize(&k2).unwrap();
                let value: HashSet<String> = bincode::deserialize(&v2).unwrap();
                println!("{:?} {:?}", key, value)
            });
        })
    }
}

fn add_to_chain(word: String, old: Option<&[u8]>) -> Option<Vec<u8>> {
    let set: HashSet<String> = match old {
        Some(b) => {
            let mut s: HashSet<String> = bincode::deserialize(b.try_into().unwrap()).unwrap();
//            println!("INSERTING TO EXISTING {:?} <- {}", s, word);
            s.insert(word);
            s
        }
        None => {
            let mut s: HashSet<String> = HashSet::new();
            s.insert(word);
            s
        }
    };

    // serialise
    let serialized = bincode::serialize(&set).unwrap();
    return Some(serialized);
}

pub fn new(db_path: &str) -> Result<Persistent, String> {
    return match Db::open(db_path) {
        Ok(d) => Ok(Persistent { db: d }),
        Err(e) => Err(e.to_string()),
    };
}

//fn print_tree(t: sled::Tree) -> () {
//    t.iter().for_each(|r| {
//        if let Ok(e) = r {
//            match e {
//                (k, v) => {
//                    let key = from_utf8(k.borrow()).unwrap();
//                    let a: [u8; 4] = v.iter().as_slice().try_into().unwrap();
//                    let value = u32::from_be_bytes(a);
//
//                    println!("{} {}", key, value)
//                }
//            }
//        }
//    })
//}

fn u32_to_ivec(x: u32) -> IVec {
    IVec::from(x.to_be_bytes().to_vec())
}

fn ivec_to_u32(x: IVec) -> u32 {
    let a: [u8; 4] = x.to_vec().as_slice().try_into().unwrap();
    return u32::from_be_bytes(a);
}
