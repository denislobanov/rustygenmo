extern crate isahc;
extern crate url;

use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;

use scraper::{ElementRef, Html, Selector};
use url::Url;

use crate::crawl::Crawler;
use crate::crawl::fanfiction::base_url;
use crate::crawl::store::Message;

use self::isahc::{HttpClient, ResponseExt};
use self::url::ParseError;
use std::error::Error;

pub struct DailyMail {
    client: HttpClient,
    tx: Sender<Option<Message>>,

    // Selectors
    link_sel: Selector,

    // archive page
    year_sel: Selector,
    month_sel: Selector,

    // months
    day_sel: Selector,

    // days
    content_sel: Selector,
    article_sel: Selector,

    // articles
    article_content_sel: Selector,
    title_sel: Selector,
    body_sel: Selector,
    p_sel: Selector,
}

pub fn new(tx: Sender<Option<Message>>) -> DailyMail {
    return DailyMail {
        client: HttpClient::new().unwrap(),
        tx,

        link_sel: Selector::parse("a").unwrap(),
        year_sel: Selector::parse("ul.archive-index.home.link-box li").unwrap(),
        month_sel: Selector::parse("ul.cleared li").unwrap(),

        day_sel: Selector::parse("div.debate.column-split.first-column").unwrap(),

        content_sel: Selector::parse("div.alpha.debate.sitemap").unwrap(),
        article_sel: Selector::parse("ul.archive-articles.debate.link-box").unwrap(),

        article_content_sel: Selector::parse("#js-article-text").unwrap(),
        title_sel: Selector::parse("h1,h2").unwrap(),
        body_sel: Selector::parse(r#"div[itemprop="articleBody""#).unwrap(),
        p_sel: Selector::parse("p").unwrap(),
    };
}

impl Crawler for DailyMail {
    fn crawl(&self, seed: &str) -> () {
        self.crawl_archive(&seed.to_string()).into_iter()
            .flat_map(|m| self.crawl_month(&m)).into_iter()
            .flat_map(|d| self.crawl_day(&d)).into_iter()
            .enumerate()
            .for_each(|(i, url)| {
                if i % 100 == 0 {
                    println!("sleeping");
                    sleep(Duration::from_secs(3));
                }
                self.crawl_article(&url);
            });

        println!("done!");
        self.tx.send(None).unwrap();
    }
}

impl DailyMail {
    // return all monthly links in the archive page
    fn crawl_archive(&self, url: &String) -> Vec<String> {
        let mut result = self.client.get(url).unwrap();
        if !result.status().is_success() {
            eprintln!("request to {} resulted in {}", url, result.status());
            return Vec::new();
        }

        let text = result.text().unwrap();
        let doc = Html::parse_document(&text);

        let base = base_url(&url).unwrap();

        let mut links: Vec<String> = Vec::new();

        let months = doc.select(&self.year_sel).next().unwrap();
        for month in months.select(&self.month_sel) {
            for link in month.select(&self.link_sel) {
                links.push(make_link(&base, link));
            }
        }

        println!("got month links {}", links.len());
        return links;
    }

    fn crawl_month(&self, url: &String) -> Vec<String> {
        let mut result = self.client.get(url).unwrap();
        if !result.status().is_success() {
            eprintln!("request to {} resulted in {}", url, result.status());
            return Vec::new();
        }

        let text = result.text().unwrap();
        let doc = Html::parse_document(&text);

        let base = base_url(&url).unwrap();

        let mut links: Vec<String> = Vec::new();
        doc.select(&self.day_sel).map(|d| d.select(&self.link_sel))
            .flat_map(|x| x.into_iter())
            .map(|l| make_link(&base, l))
            .for_each(|l| links.push(l));


        println!("got day links {}", links.len());
        return links;
    }

    fn crawl_day(&self, url: &String) -> Vec<String> {
        let mut result = self.client.get(url).unwrap();
        if !result.status().is_success() {
            eprintln!("request to crawl days {} resulted in {}", url, result.status());
            return Vec::new();
        }

        let text = result.text().unwrap();
        let doc = Html::parse_document(&text);

        let base = base_url(&url).unwrap();

        let mut links: Vec<String> = Vec::new();
        let content = doc.select(&self.content_sel).next().unwrap();

        content.select(&self.article_sel).map(|a| a.select(&self.link_sel))
            .flat_map(|x| x.into_iter())
            .map(|l| make_link(&base, l))
            .for_each(|l| links.push(l));

        println!("got article links {}", links.len());
        return links;
    }

    fn crawl_article(&self, url: &String) -> Option<()> {
        println!("fetching {}", url);
        let mut result = self.client.get(url).unwrap();
        if !result.status().is_success() {
            eprintln!("request to crawl article {} resulted in {}", url, result.status());
            return None;
        }
        let text = result.text().unwrap();
        let doc = Html::parse_document(&text);

        let base = base_url(&url).unwrap();

        let article = doc.select(&self.article_content_sel).next()?;
        let mut title = match article.select(&self.title_sel).next() {
            Some(t) => t.inner_html(),
            None => url.replace("/", ""),
        };

        let text: String = article.select(&self.body_sel).next()?
            .select(&self.p_sel).into_iter()
            .flat_map(|x| x.text().into_iter())
            .fold(String::new(), |a, x| a + x);


        // reduce title length a little
        if title.len() > 10 {
            title = title.split_whitespace().take(10).collect();
        }

        let message = Message {
            title,
            text,
        };
        Some(self.tx.send(Some(message)).unwrap())
    }
}

fn make_link(base: &Url, link: ElementRef) -> String {
    let url = link.value().attr("href").unwrap().to_string();
    return base.join(&url).unwrap().to_string();
}
