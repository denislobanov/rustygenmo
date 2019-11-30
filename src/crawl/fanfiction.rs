extern crate isahc;
extern crate url;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, sleep};
use std::thread;
use std::time::Duration;

use scraper::{Html, Selector};
use url::Url;

use crate::crawl::store;

use self::isahc::{HttpClient, ResponseExt};
use self::url::ParseError;

struct FanFiction {
    client: HttpClient,
    //thread safe
    store: store::Store,

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

struct Worker {
    thread: Option<JoinHandle<()>>,
}

enum Message {
    Crawl(String),
    Terminate,
}

impl Worker {
    pub fn new(queue: Arc<Mutex<VecDeque<Message>>>, processor: Arc<FanFiction>) -> Worker {
        let t: JoinHandle<()> = thread::spawn(move || {
            let mut sleep = false;
            loop {
                if let Ok(ref mut q) = queue.lock() {
                    match q.pop_back() {
                        Some(Message::Crawl(url)) => processor.crawl_book(url),
                        Some(Message::Terminate) => return,

                        //queue is drained
                        None => sleep = true,
                    }
                }

                if sleep {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        });

        return Worker {
            thread: Some(t),
        };
    }
}

// breadth first crawl
pub fn crawl(seed: &str, store: store::Store) -> () {
    let threads: usize = 6;

    // Run multiple crawlers in parallel
    let queue: Arc<Mutex<VecDeque<Message>>> = Arc::new(Mutex::new(VecDeque::new()));
    let processor = Arc::new(FanFiction::new(store));


    // Launch threads
    let mut crawlers = Vec::with_capacity(threads);
    for _ in 0..threads {
        crawlers.push(Worker::new(queue.clone(), processor.clone()));
    }

    // iterate through listings in a genre to build a list of books. Just use 1 crawler for this
    let mut book_urls: Vec<String> = Vec::new();

    let mut previous: String = "".parse().unwrap();
    let mut next: String = seed.to_string();
    while let Some(n) = processor.crawl_genre(&next, &mut book_urls) {
        if n == previous {
            println!("next {:?} previous {:?}", next, previous);
            break;
        }
        previous = next;
        next = n;
        //DEBUG
//            println!("next url to scrap: {} (not continuing)", next);
//            break;
    }
    println!("downloading {} books\n", book_urls.len());

    // iterate through all chapters in each book, saving the content
    book_urls.into_iter()
        .enumerate()
        .for_each(|(i, url)| {
            if i % 100 == 0 {
                print!(".");
                sleep(Duration::from_secs(3));
            }

            queue.lock().unwrap().push_front(Message::Crawl(url));
        });

    // Mark the end of the queue for the crawlers
    for _ in 0..threads+1 {
        queue.lock().unwrap().push_front(Message::Terminate);
    }

    println!("\ncrawling..");
    loop {
        {
            if let Some(msg)= queue.lock().unwrap().pop_back() {
                match msg {
                    Message::Crawl(url) => processor.crawl_book(url),
                    Message::Terminate => break,
                }
            }
        }
    }

    println!("terminating crawlers");
    for i in 0..threads {
        crawlers[i].thread.take().unwrap().join();
    }
}

impl FanFiction {
    pub fn new(store: store::Store) -> FanFiction {
        return FanFiction {
            client: HttpClient::new().unwrap(),
            store,

            content_sel: Selector::parse(r#"#content_parent #content_wrapper #content_wrapper_inner"#).unwrap(),
            books_sel: Selector::parse(r#"div.z-list.zhover.zpointer a.stitle"#).unwrap(),
            link_sel: Selector::parse("center a").unwrap(),

            title_sel: Selector::parse(r#"#profile_top b.xcontrast_txt"#).unwrap(),
            next_sel: Selector::parse(r#"span button.btn"#).unwrap(),
            chapter_sel: Selector::parse(r#"#storytext"#).unwrap(),

        };
    }

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
        let link = content.select(&self.link_sel).into_iter().last()?
            .value().attr("href").unwrap().to_string();

        return Some(base.join(&link).unwrap().into_string());
    }

    fn crawl_book(&self, url: String) {
        let mut previous: String = "".parse().unwrap();
        let mut next: String = url;

        while let Some(n) = self.crawl_chapter(&next) {
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

    fn crawl_chapter(&self, url: &String) -> Option<String> {
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
        let message = store::Message {
            title,
            text,
        };
        self.store.save(message);

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
