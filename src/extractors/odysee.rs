use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use url::Url;

pub fn odysee(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: format!(
            "https://odysee.com{}",
            Url::parse(&format!("https://{}", url))?.path()
        )
        .into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""contentUrl": "([^"]*)"#).unwrap());
    vid.vid_link = RE.captures(&resp).expect("Failed to get link")[1].into();

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<title>(.*?)</title>").unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get link")[1].into();

    Ok(vid)
}
