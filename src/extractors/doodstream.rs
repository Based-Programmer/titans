use crate::{helpers::reqwests::*, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use std::time::SystemTime;

pub async fn doodstream(url: &str, is_streaming_link: bool) -> Vid {
    const DOOD_LINK: &str = "https://dood.ws";

    static RE_DOOD: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"https?://doo[^/]*/[e|d]/([^/?&]*)"#).unwrap());

    let mut vid = Vid {
        referrer: format!(
            "{}/e/{}",
            DOOD_LINK,
            &RE_DOOD.captures(url).expect("Illegal url")[1]
        ),
        ..Default::default()
    };

    let mut resp = get_html_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<title>(.*) - DoodStream</title>"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].to_string();

    let link;

    if is_streaming_link {
        static RE_TOKEN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"(token=[^&]*)&expiry="#).unwrap());
        let token = &RE_TOKEN.captures(&resp).expect("Failed to get token")[1];

        static RE_MD5: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"get\('(/pass_md5/[^']*)"#).unwrap());
        link = format!(
            "{}{}",
            DOOD_LINK,
            &RE_MD5.captures(&resp).expect("Failed to get pass md5")[1]
        );

        // isahc is blocked by cf
        let resp = get_html_curl(&link, &vid.user_agent, &vid.referrer).await;
        vid.vid_link = format!(
            "{}?{}&expiry={}",
            resp,
            token,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
    } else {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"get\('/pass_md5/([^/]*)/([^']*)"#).unwrap());
        link = format!(
            "{}/download/{}/n/{}",
            DOOD_LINK,
            &RE.captures(&resp).unwrap()[2],
            &RE.captures(&resp).unwrap()[1],
        );

        resp = get_html_isahc(&link, &vid.user_agent, &vid.referrer).await;
        static RE_LINK: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"(https://[^\.]*\.dood\.video/[^']*)"#).unwrap());
        vid.vid_link = RE_LINK
            .captures(&resp)
            .expect("Failed to get streaming link")[1]
            .to_string();
    }

    vid
}
