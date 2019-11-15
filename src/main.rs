use std::io;
use std::fs::File;
use std::collections::HashMap;

fn readFile(path: String) -> std::result::Result<String, std::io::Error> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    Ok(s);
}

fn wordFrequency(corpus: &str) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    corpus.split



    //corpus.split_whitespace().map(|x: Some(String)| -> x.box
}

fn main() {
    println!("aaeusntaohu");
    assert!(!'\n'.is_whitespace());
}
