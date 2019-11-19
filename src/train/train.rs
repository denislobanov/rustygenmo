extern crate sled;

use sled::Db;
use std::path::PathBuf;
use crate::train::data::read_file;
use crate::train::analyse::create_corpus;
use std::convert::TryInto;
use std::str::from_utf8;
use std::borrow::Borrow;

pub struct Persistent {
    db: Db
}

impl Persistent {
    pub fn groups(&mut self, _count: usize, files: Vec<PathBuf>) -> () {
        let t = self.db.open_tree("groups").unwrap(); //todo

        for file in files {
            let data = read_file(file).unwrap(); //todo error handling

            create_corpus(&data).into_iter().for_each(|w| {
                t.update_and_fetch(w, inc).unwrap();
                return;
            });
        }

        // print some data! (for now)
        t.iter().for_each(|e| {
            match e {
                Ok((k, v)) => {
                    let a:[u8; 4] = v.iter().as_slice().try_into().unwrap();
                    let count = u32::from_be_bytes(a);
                    let key = from_utf8(k.borrow()).unwrap();

                    println!("{} {}", key, count);
                }
                Err(e) => eprintln!("shit! {}", e.to_string())
            }
        })
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
        None => 0,
    };

    Some(count.to_be_bytes().to_vec())
}
