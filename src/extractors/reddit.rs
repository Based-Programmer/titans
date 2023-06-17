use crate::{helpers::reqwests::get_html_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn reddit(url: &str) -> Vid {
    static RE_LINK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"https://((libreddit|teddit)\.[^/]*|(www\.|old\.)?reddit\.com|redd\.it)(.*)"#)
            .unwrap()
    });

    let mut vid = Vid {
        referrer: format!(
            "https://www.reddit.com{}.json",
            &RE_LINK.captures(url).expect("Illegal url")[4].trim_end_matches('/')
        ),
        ..Default::default()
    };

    let resp = get_html_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""title": "([^"]*)"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].to_string();

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""hls_url": "([^"]*)"#).unwrap());
    vid.link = RE.captures(&resp).expect("Failed to get link")[1].to_string();

    vid.referrer = vid.referrer.replace(".json", "/");

    vid
}
