use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;

pub fn libsyn(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = {
        let mut id = url
            .split_once("/episode/id/")
            .expect("Invalid Libsyn url")
            .1
            .trim_end_matches('/');
        id = id.split_once('/').unwrap_or((id, "")).0;

        Vid {
            referrer: format!("https://html5-player.libsyn.com/embed/episode/id/{}", id).into(),
            ..Default::default()
        }
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""media_url":"([^"]*)"#).unwrap());
    vid.vid_link = RE.captures(&resp).expect("Failed to get link")[1]
        .replace("\\/", "/")
        .into();

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""item_title":"([^"]*)"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get link")[1].into();

    Ok(vid)
}
