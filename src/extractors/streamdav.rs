use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn streamdav(url: &str) -> Vid {
    let mut vid = Vid {
        referrer: url
            .replace("https://streamdav.com/v/", "https://streamdav.com/e/")
            .into(),
        ..Default::default()
    };

    let resp: &str = &get_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<meta name="og:title" content="(.*)">"#).unwrap());
    vid.title = RE_TITLE.captures(resp).expect("Failed to get title")[1].into();

    static RE_VID: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<source src="(.*?)" res="([0-9]*)""#).unwrap());
    vid.vid_link = RE_VID.captures(resp).expect("Failed to get video link")[1]
        .replace("&amp;", "&")
        .into();
    vid.resolution = Some(RE_VID.captures(resp).expect("Failed to get resolution")[2].into());

    vid
}
