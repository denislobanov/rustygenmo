extern crate isahc;
extern crate url;

use std::sync::mpsc::{Receiver, Sender};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::Duration;

use scraper::{Html, Selector};
use url::Url;

use crate::crawl::Crawler;
use crate::crawl::store::Message;

use self::isahc::{HttpClient, ResponseExt};
use self::url::ParseError;

pub struct FanFiction {
    client: HttpClient, //thread safe

    // Selectors
    content_sel: Selector,

    // Genre of books
    books_sel: Selector,
    link_sel: Selector,

    // Book page
    title_sel: Selector,
    next_sel: Selector,
    chapter_sel: Selector,
}

pub fn new() -> FanFiction {
    return FanFiction {
        client: HttpClient::new().unwrap(),

        content_sel: Selector::parse(r#"#content_parent #content_wrapper #content_wrapper_inner"#).unwrap(),
        books_sel: Selector::parse(r#"div.z-list.zhover.zpointer a.stitle"#).unwrap(),
        link_sel: Selector::parse("center a").unwrap(),

        title_sel: Selector::parse(r#"#profile_top b.xcontrast_txt"#).unwrap(),
        next_sel: Selector::parse(r#"span button.btn"#).unwrap(),
        chapter_sel: Selector::parse(r#"#storytext"#).unwrap(),

    };
}

impl FanFiction {
    // breadth first crawl
    pub fn crawl(&self, seed: &str, msg_tx: Sender<Option<Message>>) -> () {
        let mut book_urls: Vec<String> = Vec::new();

        // iterate through listings in a genre to build a list of books
        let mut previous: String = "".parse().unwrap();
        let mut next: String = seed.to_string();
        while let Some(n) = self.crawl_genre(&next, &mut book_urls) {
            if n == previous{
                break;
            }
            previous = next;
            next = n;
            //DEBUG
//            println!("next url to scrap: {} (not continuing)", next);
//            break;
        }

        println!("downloading books");

        // instantiate crawler threads
        let threads = 3;
        let mut hs = Vec::new();
        let mut chans = Vec::new();

        for i in 0..threads {
            let t_msg_tx = std::sync::mpsc::Sender::clone(&msg_tx);
            let (url_tx, url_rx) = std::sync::mpsc::channel();
            chans[i] = url_tx;

            hs.push(spawn(move || self.crawl_thread(url_rx, t_msg_tx)));
        }

        // iterate through all chapters in each book, saving the content
        book_urls.into_iter()
            .enumerate()
            .for_each(|(i, url)| {
                if i % 100 == 0 {
                    println!("sleeping..");
                    sleep(Duration::from_secs(3));
                }

                chans[i%threads].send(Some(url)).unwrap();
            });

        // tell store that we've finished
        msg_tx.send(None).unwrap();

        // stop the threads
        for i in 0..3 {
            chans[i].send(None).unwrap();
            hs[i].join();
        }
    }
}

impl FanFiction {
    // Get all book urls in a genre, return next url to crawl
    fn crawl_genre(&self, url: &String, book_urls: &mut Vec<String>) -> Option<String> {
        let mut result = self.client.get(url).unwrap();
        if !result.status().is_success() {
            eprintln!("request to {} resulted in {}", url, result.status());
            return None;
        }

        let text = result.text().unwrap();
        let doc = Html::parse_document(&text);
        let content = doc.select(&self.content_sel).next().unwrap();

        // descending selectors for books
        let base = base_url(&url).unwrap();

        for book in content.select(&self.books_sel) {
            let book_url = book.value().attr("href").unwrap().to_string();
            book_urls.push(base.join(&book_url).unwrap().into_string());
        }

        // descending selectors for getting next page url's
        let link = content.select(&self.link_sel).next().unwrap()
            .value().attr("href").unwrap().to_string();

        return Some(base.join(&link).unwrap().into_string());
    }

    fn crawl_thread(&self, url_rx: Receiver<Option<String>>, msg_tx: Sender<Option<Message>>) {
        loop {
            match url_rx.recv().unwrap() {
                Some(url) => self.crawl_book(url, &msg_tx),
                _ => return,
            }

        }
    }

    fn crawl_book(&self, url: String, msg_tx: &Sender<Option<Message>>) {
        let mut previous: String = "".parse().unwrap();
        let mut next: String = url;

        while let Some(n) = self.crawl_chapter(&next, msg_tx) {
            if n == previous {
                break;
            }
            previous = next;
            next = n;

            //DEBUG
//            println!("previous={} next={}", previous, next);
//            break
        }
    }

    fn crawl_chapter(&self, url: &String, msg_tx: &Sender<Option<Message>>) -> Option<String> {
//        println!("url={}", url);

        let mut result = self.client.get(url).unwrap();
        if !result.status().is_success() {
            eprintln!("request to {} resulted in {}", url, result.status());
            return None;
        }
        let doc = Html::parse_document(&result.text().unwrap());

        let content = doc.select(&self.content_sel).next().unwrap();
        let title = match content.select(&self.title_sel).next() {
            Some(t) => t.inner_html(),
            None => url.replace("/", ""),
        };
        let text = content.select(&self.chapter_sel).next()?
            .text().into_iter()
            .fold(String::new(), |a, x| a + x);

        // save
        let message = Message {
            title,
            text,
        };
        msg_tx.send(Some(message)).unwrap();

        // build next url
        let next = match content.select(&self.next_sel).next() {
            Some(n) => n.value().attr("onclick").unwrap()
                .replace("self.location=", "")
                .replace("'", "")
                .trim()
                .to_string(),
            None => {
                return None;
            }
        };

//        println!("next={:?}", next);
        let base = base_url(&url).unwrap();
        return Some(base.join(&next).unwrap().into_string());
    }
}

pub fn base_url(url: &str) -> Result<Url, ParseError> {
    let mut base = Url::parse(url)?;

    match base.path_segments_mut() {
        Ok(mut path) => {
            path.clear();
        }
        Err(_) => {
            return Err(ParseError::EmptyHost);
        }
    }

    base.set_query(None);
    return Ok(base);
}
