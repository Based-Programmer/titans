use std::error::Error;

use crate::{
    helpers::{reqwests::get_isahc, unescape_html_chars::unescape_html_chars},
    Vid,
};
use once_cell::sync::Lazy;
use regex::Regex;

pub fn streamdav(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: url
            .replacen("streamdav.com/v/", "streamdav.com/e/", 1)
            .into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<meta name="og:title" content="(.*)">"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].into();

    static RE_VID: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<source src="(.*?)" res="([0-9]*)""#).unwrap());
    vid.vid_link =
        unescape_html_chars(&RE_VID.captures(&resp).expect("Failed to get video link")[1]);
    vid.resolution = Some(RE_VID.captures(&resp).expect("Failed to get resolution")[2].parse()?);

    Ok(vid)
}
