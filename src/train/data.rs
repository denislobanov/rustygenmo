use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{PathBuf, Path};

pub fn read_file<P: AsRef<Path>>(file: P) -> io::Result<String> {
    let mut s = String::new();
    File::open(file)?.read_to_string(&mut s)?;
    Ok(s)
}

pub fn list_dir(path: &str) -> io::Result<Vec<PathBuf>> {
    return Ok(fs::read_dir(path)?.into_iter()
        .map(|p| p.unwrap().path())
        .collect());
}

// TODO: func(String) -> Vec<Lazy<String>>
// load files in a directory and return a list of lazy evaluation futures that would read the file
// only at the point of execution.
