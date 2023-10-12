use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn mp4upload(url: &str) -> Vid {
    static RE_LINK: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"https://(www\.)?mp4upload\.com/(embed-)?([^.]*\.html)").unwrap());
    let mut vid = Vid {
        referrer: format!(
            "https://www.mp4upload.com/embed-{}",
            &RE_LINK.captures(url).expect("Invalid url")[3]
        )
        .into(),
        ..Default::default()
    };

    let resp: &str = &get_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"src: "(https://[^.]*\.mp4upload\.com/files/[^"]*)"#).unwrap());
    vid.vid_link = RE.captures(resp).expect("Failed to get link")[1].into();

    vid
}
