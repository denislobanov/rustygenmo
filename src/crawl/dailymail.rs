extern crate isahc;
extern crate url;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use scraper::{ElementRef, Html, Selector};
use url::Url;

use crate::crawl::{pool, store};
use crate::crawl::fanfiction::base_url;
use crate::crawl::store::Chapter;

use self::isahc::{HttpClient, ResponseExt};

pub struct DailyMail {
    client: HttpClient,
    store: store::Store,

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

// single threaded
pub fn crawl(seed: &str, store: store::Store) -> () {
    let processor = DailyMail::new(store);
    let articles = processor.crawl_archive(&seed.to_string()).into_iter()
        .flat_map(|m| processor.crawl_month(&m)).into_iter()
        .flat_map(|d| processor.crawl_day(&d)).into_iter()
        .enumerate()
        .for_each(|(i, u)| {
            if i % 100 == 0 {
                thread::sleep(Duration::from_secs(2));
            }
            processor.crawl_article(&u);
        });
}

impl pool::Processor for DailyMail {
    fn crawl(&self, url: String) {
        self.crawl_article(&url);
    }
}

impl DailyMail {
    pub fn new(store: store::Store) -> DailyMail {
        return DailyMail {
            client: HttpClient::new().unwrap(),
            store,

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
        let mut result = match self.client.get(url) {
            Ok(r) => r,
            Err(_) => return None,
        };
        if !result.status().is_success() {
            eprintln!("request to crawl article {} resulted in {}", url, result.status());
            return None;
        }

        let text = match result.text() {
            Ok(t) => t,
            Err(_) => return None,
        };
        let doc = Html::parse_document(&text);

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

        let chapter = Chapter {
            title,
            text,
        };
        Some(self.store.save(chapter))
    }
}

fn make_link(base: &Url, link: ElementRef) -> String {
    let url = link.value().attr("href").unwrap().to_string();
    return base.join(&url).unwrap().to_string();
}
