use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use std::time::SystemTime;

pub async fn doodstream(url: &str, is_streaming_link: bool) -> Vid {
    const DOOD_LINK: &str = "https://dood.ws";

    let mut vid = {
        let path = url.trim_end_matches('/').rsplit_once('/').unwrap().1;

        Vid {
            referrer: format!("{}/e/{}", DOOD_LINK, path).into(),
            ..Default::default()
        }
    };

    let mut resp = get_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    vid.title = {
        static RE_TITLE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"<title>(.*?) - DoodStream</title>").unwrap());
        RE_TITLE.captures(&resp).expect("Failed to get title")[1].into()
    };

    if is_streaming_link {
        let token: Box<str> = {
            static RE_TOKEN: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"(token=[^&]*)&expiry=").unwrap());

            RE_TOKEN.captures(&resp).expect("Failed to get token")[1].into()
        };

        let link: &str = {
            static RE_MD5: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"get\('(/pass_md5/[^']*)").unwrap());

            &format!(
                "{}{}",
                DOOD_LINK,
                &RE_MD5.captures(&resp).expect("Failed to get pass md5")[1]
            )
        };

        resp = get_isahc(link, &vid.user_agent, &vid.referrer).await;
        vid.vid_link = format!(
            "{}?{}&expiry={}",
            resp,
            token,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        )
        .into();
    } else {
        let link: &str = {
            static RE: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"get\('/pass_md5/([^/]*)/([^']*)").unwrap());

            &format!(
                "{}/download/{}/n/{}",
                DOOD_LINK,
                &RE.captures(&resp).unwrap()[2],
                &RE.captures(&resp).unwrap()[1],
            )
        };
        resp = get_isahc(link, &vid.user_agent, &vid.referrer).await;

        static RE_LINK: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(https://[^.]*\.video-delivery\.net/[^']*)").unwrap());
        vid.vid_link = RE_LINK
            .captures(&resp)
            .expect("Failed to get download link")[1]
            .into();
    }

    vid
}
