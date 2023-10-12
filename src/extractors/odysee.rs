use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn odysee(url: &str) -> Vid {
    static RE_LINK: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^https://(lbry|librarian)\.[^/]*/").unwrap());

    let mut vid = Vid {
        referrer: RE_LINK.replace(url, "https://odysee.com/").into(),
        ..Default::default()
    };

    let resp: &str = &get_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""contentUrl": "([^"]*)"#).unwrap());
    vid.vid_link = RE.captures(resp).expect("Failed to get link")[1].into();

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<title>(.*?)</title>").unwrap());
    vid.title = RE_TITLE.captures(resp).expect("Failed to get link")[1].into();

    vid
}
