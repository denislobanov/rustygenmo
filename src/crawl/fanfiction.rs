extern crate isahc;
extern crate url;

use std::sync::mpsc::Sender;

use scraper::{Html, Selector};
use url::Url;

use crate::crawl::Crawler;
use crate::crawl::store::Message;

use self::isahc::{HttpClient, ResponseExt};
use self::url::ParseError;

pub struct FanFiction {
    client: HttpClient,
    tx: Sender<Option<Message>>,

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

pub fn new(tx: Sender<Option<Message>>) -> FanFiction {
    return FanFiction {
        client: HttpClient::new().unwrap(),
        tx,

        content_sel: Selector::parse(r#"#content_parent #content_wrapper #content_wrapper_inner"#).unwrap(),
        books_sel: Selector::parse(r#"div.z-list.zhover.zpointer a.stitle"#).unwrap(),
        link_sel: Selector::parse("center a").unwrap(),

        title_sel: Selector::parse(r#"#profile_top b.xcontrast_txt"#).unwrap(),
        next_sel: Selector::parse(r#"span button.btn"#).unwrap(),
        chapter_sel: Selector::parse(r#"#storytext"#).unwrap(),

    };
}

impl Crawler for FanFiction {
    // breadth first crawl
    fn crawl(&self, seed: &str) -> () {
        let mut book_urls: Vec<String> = Vec::new();

        // iterate through listings in a genre to build a list of books
        let mut next: String = seed.to_string();
        while let Some(n) = self.crawl_genre(&next, &mut book_urls) {
            if n == seed {
                break;
            }
            next = n;
            //DEBUG
            println!("next url to scrap: {} (not continuing)", next);
            break;
        }

        println!("downloading books");

        // iterate through all chapters in each book, saving the content
        book_urls.into_iter().take(1).for_each(|url| self.crawl_book(url));

        // tell store that we've finished
        self.tx.send(None).unwrap();
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

        print!(".");
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
            println!("previous={} next={}", previous, next);
            break
        }
    }

    fn crawl_chapter(&self, url: &String) -> Option<String> {
        println!("url={}", url);

        let mut result = self.client.get(url).unwrap();
        if !result.status().is_success() {
            eprintln!("request to {} resulted in {}", url, result.status());
            return None;
        }
        let doc = Html::parse_document(&result.text().unwrap());

        let content = doc.select(&self.content_sel).next().unwrap();
        let title = content.select(&self.title_sel).next().unwrap()
            .inner_html();
        let text = content.select(&self.chapter_sel).next().unwrap()
            .text().into_iter()
            .fold(String::new(), |a, x| a+x);

        // save
        let message = Message {
            title,
            text,
        };
        self.tx.send(Some(message)).unwrap();

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

        println!("next={:?}", next);

        let base = base_url(&url).unwrap();
        return Some(base.join(&next).unwrap().into_string());
    }
}

fn base_url(url: &str) -> Result<Url, ParseError> {
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
    return Ok(base)
}
