use crate::{
    helpers::{reqwests::get_isahc, unescape_html_chars::unescape_html_chars},
    Vid,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;

pub fn bitchute(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = {
        let id = url.rsplit_once('/').unwrap().1;

        Vid {
            referrer: format!("https://www.bitchute.com/api/beta9/embed/{}", id).into(),
            ..Default::default()
        }
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#" +var video_name = "(.+)";[\s\S]+ +var media_url = '([^']+)"#).unwrap()
    });
    let cap = RE.captures(&resp).expect("Failed to get link");

    vid.title = unescape_html_chars(&cap[1]);
    vid.vid_link = cap[2].into();

    Ok(vid)
}
