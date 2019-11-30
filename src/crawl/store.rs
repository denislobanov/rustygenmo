use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::Receiver;

pub struct Store {
    path: String,
}

pub fn new(path: &str) -> Store {
    return Store {
        path: path.to_string()+"/",
    };
}

#[derive(Debug)]
pub struct Message {
    pub title: String,
    pub text: String,
}

impl Store {
    pub fn run(&self, msg_rx: Receiver<Option<Message>>) {
        loop {
            match msg_rx.recv().unwrap() {
                Some(m) => self.save(m),
                None => return,
            }
        }
    }

    pub fn save(&self, msg: Message) {
        let filename = msg.title.trim()
            .replace(" ", "_")
            .replace("/", "")
            .replace("?", "")
            .replace(".", "")
            .replace(";", "")
            .replace("&", "");
        let p = Path::new(&self.path).join(&filename);

//        println!("path is: {:?}", p);
        if !p.is_file() {
            File::create(&p).unwrap();
        }

        let mut file = OpenOptions::new()
            .write(true).append(true)
            .open(&p)
            .unwrap();

        let mut text = msg.text.trim().to_string();
        while text.starts_with("\n") {
            text = text[1..text.len()].to_string();
        }

        match file.write_all(msg.text.trim().as_bytes()) {
            Err(e) => eprintln!("failed to write file{}: {}", filename, e),
            Ok(_) => return,
        }

        let _ = std::writeln!(file, "\n");
        let _ = file.flush();
    }
}