use std::fs::{OpenOptions, File};
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
    pub fn run(&self, msg_rx: Receiver<Option<Message>>, stop_rx: Receiver<bool>) {
        loop {
            match msg_rx.recv().unwrap() {
                Some(m) => self.save(m),
                None => return,
            }
        }
    }

    fn save(&self, msg: Message) {
        let filename = msg.title.trim().replace(" ", "_");
        let p = Path::new(&self.path).join(&filename);

        let mut append = false;
        println!("path is: {:?}", p);
        if !p.is_file() {
            File::create(&p).unwrap();
            append = true;
        }

        let mut file = OpenOptions::new()
            .write(true).append(true)
            .open(&p)
            .unwrap();

        if append {
            let _ = std::writeln!(file, "\n");
        }

        match file.write_all(msg.text.trim().as_bytes()) {
            Err(e) => eprintln!("failed to write file{}: {}", filename, e),
            Ok(_) => return,
        }

        let _ = std::writeln!(file, "\n");
        let _ = file.flush();
    }
}