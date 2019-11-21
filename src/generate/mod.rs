extern crate sled;
extern crate rand;

use clap::ArgMatches;
use std::collections::{HashMap, HashSet, VecDeque};
use sled::{IVec, Tree};
use self::rand::seq::IteratorRandom;
use std::convert::TryInto;
use self::rand::prelude::ThreadRng;

pub fn run_cmd(args: &ArgMatches) -> () {
    let db_path = match args.value_of("dbpath") {
        Some(v) => v,
        None => "test",
    };
    let count = match args.value_of("count") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => 10,
    };

    let db = sled::Db::open(db_path).unwrap();
    let groups = db.open_tree("groups").unwrap();

    // create m * n-grams
    let chain_set: HashSet<u32> = groups.into_iter().fold(HashSet::new(), |mut s, v| {
        let (_, g) = v.unwrap();
        s.insert(ivec_to_u32(g));
        return s;
    });
    let mut largest_n: u32 = 0;

    let mut chains: HashMap<u32, Tree> = HashMap::new();
    chain_set.iter().for_each(|&g| {
        chains.insert(g, db.open_tree(u32_to_ivec(g)).unwrap());
        if g > largest_n {
            largest_n = g
        }
    });

    println!("chains: {}\n largest n: {}", chains.len(), largest_n);

    let mut rng = rand::thread_rng();

    let mut sentence = String::new();

    // use a circular buffer as our 'stack' of last seen words. we will then try to find a bunch of
    // matches across all chains (stack should be as deep as the largest n-gram size), and select
    // a random one from those that provide an answer.
    let mut stack: VecDeque<String> = VecDeque::with_capacity(largest_n as usize);
    seed_stack(&chains, &mut rng, &mut stack);

    println!("initial stack: {:?}", stack);

    // now run the main loop until we hit our target length.
    let mut len = 0;

    while len < count {
        if stack.len() > largest_n as usize {
            for _ in 0..stack.len() - largest_n as usize {
                sentence.push_str(&stack.pop_back().unwrap());
                sentence.push_str(" ");
                len += 1
            }
        } else if stack.len() <= 0 {
            sentence.remove(sentence.len()-1);
            sentence.push_str(". ");
            seed_stack(&chains, &mut rng, &mut stack);
        }

        let k = stack.len() as u32;
        match chains.get(&k) {
            Some(t) => {
                match t.get(bincode::serialize(&stack).unwrap()).unwrap() {
                    Some(vec) => {
                        let words: HashSet<String> = bincode::deserialize(&vec).unwrap();
                        words.iter().for_each(|w| stack.push_front(w.to_string()));
                    }
                    None => {
                        sentence.push_str(&stack.pop_back().unwrap());
                        sentence.push_str(" ");
                        len += 1
                    }
                }
            }
            None => {
                // todo reduce key and try again. for now just bail
                sentence.push_str(&stack.pop_back().unwrap());
                sentence.push_str(" ");
                len += 1;
                eprintln!("no chain found for key {} stack is now {}", k, stack.len())
            }
        }
    }

    println!("output:\n\n{}", sentence);
}


fn seed_stack(chains: &HashMap<u32, Tree>, rng: &mut ThreadRng, stack: &mut VecDeque<String>) {
    // choose a random chain
    let start = match chains.iter().choose(rng) {
        Some((_, v)) => v,
        None => {
            panic!("couldnt choose a random starting chain")
        }
    };

    // choose a random kv pair from said chain
    let (k, v) = start.iter().choose(rng).unwrap().unwrap();

    let x: Vec<String> = bincode::deserialize(&k).unwrap();
    x.into_iter().for_each(|w| stack.push_front(w));

    let y: HashSet<String> = bincode::deserialize(&v).unwrap();
    y.into_iter().for_each(|w| stack.push_front(w));
}

fn u32_to_ivec(x: u32) -> IVec {
    IVec::from(x.to_be_bytes().to_vec())
}

fn ivec_to_u32(x: IVec) -> u32 {
    let a: [u8; 4] = x.to_vec().as_slice().try_into().unwrap();
    return u32::from_be_bytes(a);
}
