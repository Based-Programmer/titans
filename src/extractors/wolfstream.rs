use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub fn wolfstream(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: format!("https://{url}").replacen("embed-", "", 1).into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.*)\[/URL\]").unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].into();

    static RE_VID: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"    sources: \[\{file:"([^"]*)"#).unwrap());
    let m3u8 = &RE_VID.captures(&resp).expect("Failed to get vid link")[1];

    let stream_base = m3u8
        .split_once(',')
        .expect("Failed to ',' for spliting urlset")
        .0;
    let params = m3u8
        .split_once(".urlset/master.m3u8?")
        .expect("Failed to get urlset/master.m3u8 in m3u8 master url")
        .1;

    if m3u8.contains("x,") {
        vid.vid_link = format!("{stream_base}x/index.m3u8?{params}").into();
    } else if m3u8.contains("h,") {
        vid.vid_link = format!("{stream_base}h/index.m3u8?{params}").into();
    }

    Ok(vid)
}
