use crate::{
    helpers::{
        reqwests::{client, get_isahc_client},
        unescape_html_chars::unescape_html_chars,
    },
    Vid,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{error::Error, process::exit, time::SystemTime};

pub fn doodstream(mut url: &str, is_streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    const BASE_URL: &str = "https://dood.to";

    let mut vid = {
        url = url.trim_end_matches('/');
        let mut path = url
            .rsplit_once("/e/")
            .unwrap_or(url.rsplit_once("/d/").unwrap_or_default())
            .1;

        if path.is_empty() {
            const RED: &str = "\u{1b}[31m";
            const RESET: &str = "\u{1b}[0m";

            eprintln!("{RED}Invalid Doodstream url{RESET}");
            exit(1);
        }

        path = path.split_once('/').unwrap_or((path, "")).0;

        Vid {
            referrer: format!("{}/e/{}", BASE_URL, path).into(),
            ..Default::default()
        }
    };

    let client = &client(vid.user_agent, &vid.referrer)?;
    let resp = get_isahc_client(client, &vid.referrer)?;

    vid.title = {
        static RE_TITLE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"<title>(.*?) - DoodStream</title>").unwrap());

        unescape_html_chars(&RE_TITLE.captures(&resp).expect("Failed to get title")[1])
    };

    if is_streaming_link {
        let token: Box<str> = {
            static RE_TOKEN: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"(token=[^&]*)&expiry=").unwrap());

            RE_TOKEN.captures(&resp).expect("Failed to get token")[1].into()
        };

        let link = {
            static RE_MD5: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"get\('(/pass_md5/[^']*)").unwrap());

            format!(
                "{}{}",
                BASE_URL,
                &RE_MD5.captures(&resp).expect("Failed to get pass md5")[1]
            )
            .into_boxed_str()
        };

        drop(resp);
        let resp = get_isahc_client(client, &link)?;

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
        let link = {
            static RE: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"get\('/pass_md5/([^/]*)/([^']*)").unwrap());

            let captures = RE.captures(&resp).expect("Failed to get video link");

            format!("{}/download/{}/n/{}", BASE_URL, &captures[2], &captures[1]).into_boxed_str()
        };

        drop(resp);
        let resp = get_isahc_client(client, &link)?;

        static RE_LINK: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"(https://[^.]*\.video-delivery\.net/[^"']*)"#).unwrap());
        vid.vid_link = RE_LINK
            .captures(&resp)
            .expect("Failed to get download link")[1]
            .into();
    }

    Ok(vid)
}
