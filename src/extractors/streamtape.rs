use crate::{
    helpers::{reqwests::get_isahc, unescape_html_chars::unescape_html_chars},
    Vid,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use url::Url;

pub fn streamtape(url: &str, streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: format!(
            "https://streamtape.net{}",
            Url::parse(&format!("https://{}", url))?.path()
        )
        .into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<meta name="og:title" content="([^"]*)"#).unwrap());
    vid.title = unescape_html_chars(&RE_TITLE.captures(&resp).unwrap()[1]);

    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"<div id="(no)?robotlink".*?/(streamtape\.net/.*?)&token[\s\S]*?(&token=[^']*)"#,
        )
        .unwrap()
    });

    let captures = RE.captures(&resp).expect("Failed to get video link");
    vid.vid_link = {
        if streaming_link {
            format!("https://{}{}&stream=1", &captures[2], &captures[3])
        } else {
            format!("https://{}{}&dl=1", &captures[2], &captures[3])
        }
    }
    .into();

    Ok(vid)
}
