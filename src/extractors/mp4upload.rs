use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub fn mp4upload(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = {
        let id = url.rsplit_once('/').unwrap().1.trim_start_matches("embed-");

        Vid {
            referrer: format!("https://www.mp4upload.com/embed-{}", id).into(),
            ..Default::default()
        }
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"src: "(https://[^.]*\.mp4upload\.com/files/[^"]*)"#).unwrap());
    vid.vid_link = RE.captures(&resp).expect("Failed to get link")[1].into();

    Ok(vid)
}
