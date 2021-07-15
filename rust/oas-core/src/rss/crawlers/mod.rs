use std::pin::Pin;

use super::Crawler;

pub fn default_crawlers() -> Vec<Pin<Box<dyn Crawler>>> {
    vec![Box::pin(frn::FrnCrawler {}), Box::pin(cba::CbaCrawler {})]
}

pub mod util {
    use std::collections::HashMap;
    use url::Url;
    pub fn query_map(url: &Url) -> HashMap<String, String> {
        url.query_pairs().into_owned().collect()
    }

    pub fn set_query_map(url: &mut Url, map: &HashMap<String, String>) {
        url.query_pairs_mut().clear().extend_pairs(map.iter());
    }

    pub fn insert_or_add(map: &mut HashMap<String, String>, key: &str, default: usize, add: usize) {
        if let Some(value) = map.get_mut(key) {
            let num: Result<usize, _> = value.parse();
            let num = num.map_or(default, |num| num + add);
            *value = num.to_string();
        } else {
            map.insert(key.into(), default.to_string());
        }
    }
}

pub mod frn {
    use super::util::{insert_or_add, query_map, set_query_map};
    use crate::rss::{Crawler, FetchedFeedPage, Next};

    pub struct FrnCrawler {}

    #[async_trait::async_trait]
    impl Crawler for FrnCrawler {
        async fn next(&self, feed_page: FetchedFeedPage) -> anyhow::Result<Next> {
            match feed_page.items.len() {
                0 => Ok(Next::Finished),
                _ => {
                    let len = feed_page.items.len();
                    let mut url = feed_page.url;
                    let mut params = query_map(&url);
                    insert_or_add(&mut params, "start", len, len);
                    set_query_map(&mut url, &params);
                    Ok(Next::NextPage(url))
                }
            }
        }

        fn domains(&self) -> Vec<String> {
            vec![
                "freie-radios.net".to_string(),
                "www.freie-radios.net".to_string(),
            ]
        }
    }
}

pub mod cba {
    use super::util::{insert_or_add, query_map, set_query_map};
    use crate::rss::{Crawler, FetchedFeedPage, Next};

    pub struct CbaCrawler {}

    #[async_trait::async_trait]
    impl Crawler for CbaCrawler {
        async fn next(&self, feed_page: FetchedFeedPage) -> anyhow::Result<Next> {
            let len = feed_page.items.len();
            match len {
                0 => Ok(Next::Finished),
                _ => {
                    let mut url = feed_page.url;
                    let mut params = query_map(&url);
                    insert_or_add(&mut params, "offset", len, len);
                    set_query_map(&mut url, &params);
                    Ok(Next::NextPage(url))
                }
            }
        }

        fn domains(&self) -> Vec<String> {
            vec!["cba.media".to_string(), "cba.fro.at".to_string()]
        }
    }
}
