extern crate isahc;
extern crate soup;

use scraper::{Html, Selector};
use self::isahc::HttpClient;
use crate::crawl::Crawler;

pub struct FanFiction {
    path: String,
    client: HttpClient,
}

impl Crawler for FanFiction {
    // breadth first crawl
    fn crawl(&self, seed: String) -> () {
        let mut book_urls: Vec<String> = Vec::new();

        let mut next = seed;
        while let Some(n) = self.crawl_genre(next, &mut book_urls) {
            if n == seed {
                break;
            }
            next = n;
        }

        // build a list of links to continue crawl

        // build a list of link for content download
    }
}

impl FanFiction {
    // Get all book urls in a genre, return next url to crawl
    fn crawl_genre(&self, url: String, book_urls: &mut Vec<String>) -> Some<String> {
        let result = self.client.get(url).unwrap();
        let doc = Html::parse_document(&result.body().text().unwrap());

        // content wrapper contains both titles & links to next pages,
        let cont_sel = Selector::parse("div id=\"content_wrapper_inner\"").unwrap();

        // descending selectors for books
        let books_sel = Selector::parse("div class=\"z-list zhover zpointer \"").unwrap();
        let book_sel = Selector::parse("a class=\"stitle\"").unwrap();
        let books = doc.select(&cont_sel).next().unwrap()
            .select(&books_sel).next().unwrap();

        for book in books.select(&book_sel) {
            book_urls.push(book.value().name().to_string());
        }

        // descending selectors for getting next page url's
        let links_sel = Selector::parse("center").unwrap();
        let link_sel = Selector::parse("a").unwrap();
        let link = doc.select(&cont_sel).next().unwrap()
            .select(&links_sel).next().unwrap()
            .select(&link_sel).next().unwrap();

        return Some(link.value().name().to_string());
    }

    fn crawl_book(&self, url: String) -> Some<String> {
        let result = self.client.get(url).unwrap();
        let doc = Html::parse_document(&result.body().text().unwrap());

        let cont_sel = Selector::parse("div id=\"content_wrapper_inner\"").unwrap();

        let prof_sel = Selector::parse("div id=\"profile_top\"").unwrap();
        let title_sel = Selector::parse("b class=\"xcontrast_txt\"").unwrap();
        let next_sel = Selector::parse("button class=\"btn\" type=\"BUTTON\"").unwrap();
        let chapter_sel = Selector::parse("div id=\"storytext\" class=\"storytext xcontrast_txt nocopy\"").unwrap();




    }
}
