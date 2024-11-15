use crate::{
    helpers::{reqwests::get_isahc, unescape_html_chars::unescape_html_chars},
    Vid,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;

pub fn odysee(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = {
        let path = url
            .split_once('/')
            .unwrap()
            .1
            .trim_start_matches("$/embed/");

        Vid {
            referrer: format!("https://odysee.com/{}", path).into(),
            ..Default::default()
        }
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""contentUrl": "([^"]*)"#).unwrap());
    vid.vid_link = RE.captures(&resp).expect("Failed to get link")[1].into();

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<title>(.*?)</title>").unwrap());
    vid.title = unescape_html_chars(&RE_TITLE.captures(&resp).expect("Failed to get link")[1]);

    Ok(vid)
}
