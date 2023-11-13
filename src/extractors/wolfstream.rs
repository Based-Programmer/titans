use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn wolfstream(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: url.replace("embed-", "").into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer).await?;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.*)\[/URL\]").unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].into();

    static RE_VID: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"    sources: \[\{file:"([^"]*)"#).unwrap());
    vid.vid_link = RE_VID.captures(&resp).expect("Failed to get link")[1].into();

    Ok(vid)
}
