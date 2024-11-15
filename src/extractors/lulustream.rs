use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;

pub fn lulustream(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = {
        let id = url.rsplit_once('/').unwrap().1;

        Vid {
            referrer: format!("https://cdn1.site/e/{}", id).into(),
            ..Default::default()
        }
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"sources: \[\{file:"([^"]+)"#).unwrap());
    vid.vid_link = RE.captures(&resp).expect("Failed to get link")[1]
        .replacen("/master.m3u8?t=", "/index-v1-a1.m3u8?t=", 1)
        .into();

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"<title>(.+?)( *- Lulustream\.mp4)? *- LuluStream</title>").unwrap()
    });
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get link")[1].into();

    Ok(vid)
}
