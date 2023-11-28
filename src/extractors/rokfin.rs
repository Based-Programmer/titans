use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{from_str, Value};
use std::error::Error;

pub async fn rokfin(url: &str, resolution: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: url.into(),
        ..Default::default()
    };

    let resp = {
        let id = url
            .trim_start_matches("https://rokfin.com/post/")
            .split_once('/')
            .unwrap()
            .0;

        let api = format!("https://prod-api-v2.production.rokfin.com/api/v2/public/post/{id}")
            .into_boxed_str();

        get_isahc(&api, vid.user_agent, &vid.referrer).await?
    };

    let data: Value = from_str(&resp).expect("Failed to serialize json");

    vid.title = data["content"]["contentTitle"]
        .as_str()
        .expect("Failed to get title")
        .into();

    let m3u8: Box<str> = data["content"]["contentUrl"]
        .as_str()
        .expect("Failed to get ")
        .into();

    drop(resp);
    drop(data);

    let resp = get_isahc(&m3u8, vid.user_agent, &vid.referrer).await?;

    if resolution != "best" {
        let re = Regex::new(&format!(
        r"#EXT-X-STREAM-INF:.*?,RESOLUTION=[0-9]*x{resolution}[\s\S]*?(https://.*?\.rokfin\.com/.*/rendition\.m3u8.*)"
    ))
    .unwrap();
        if let Some(vid_link) = re.captures(&resp) {
            vid.vid_link = vid_link[1].into();
        }
    }

    if vid.vid_link.is_empty() {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(https://.*?\.rokfin\.com/.*/rendition\.m3u8.*)").unwrap());
        vid.vid_link = RE.captures(&resp).expect("Failed to get link")[1].into();
    }

    Ok(vid)
}
