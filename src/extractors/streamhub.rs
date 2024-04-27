use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub fn streamhub(url: &str, _streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    const BASE_URL: &str = "streamhub.to";
    const STREAM_URL: &str = "streamhub.top";

    let mut vid = {
        let path = url.trim_end_matches('/').rsplit_once('/').unwrap().1;

        Vid {
            referrer: format!("https://{}/{}", BASE_URL, path).into(),
            ..Default::default()
        }
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(" *<h4>(.*?)</h4>").unwrap());
    vid.title = RE_TITLE.captures(&resp).unwrap()[1].into();

    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\|vjsplayer\|data\|(.*?)\|(.*?)\|.*?\|chromecast\|(.*?)\|(.*?)\|(.*?)\|.*?\|sp\|(.*?)\|(.*?)\|m3u8\|master\|(.*?)\|(.*?)\|").unwrap()
    });
    let captures = RE.captures(&resp).expect("Failed to get video link");

    let t = match &captures[7].split_once('|') {
        Some(val) => format!("{}-{}", val.1, val.0),
        None => captures[7].to_owned(),
    };

    vid.vid_link = format!(
        "https://{}.{}/{}/{}/{}/{}/index-v1-a1.m3u8?t={}&s={}&e={}&f={}&i=0.0&sp=0",
        &captures[5],
        STREAM_URL,
        &captures[9],
        &captures[4],
        &captures[3],
        &captures[8],
        t,
        &captures[1],
        &captures[6],
        &captures[2]
    )
    .into();

    Ok(vid)
}
