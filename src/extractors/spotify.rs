use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{from_str, Value};

pub fn spotify(url: &str) -> Result<Vid, Box<dyn Error>> {
    static RE_ID: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"open\.spotify\.com/(embed/)?episode/([^?&]*)").unwrap());
    let id = &RE_ID.captures(url).expect("Invalid url: id wasn't found")[2];

    let mut vid = Vid {
        referrer: format!("https://{}", url).into(),
        ..Default::default()
    };

    let json_resp: Value = {
        let resp = get_isahc(
            &format!(
            "https://spclient.wg.spotify.com/soundfinder/v1/unauth/episode/{id}/com.widevine.alpha"
        ),
            vid.user_agent,
            &vid.referrer,
        )?
        .replace("\\u003d", "=");

        from_str(&resp).expect("Failed to derive json")
    };

    if let Some(url) = json_resp["passthroughUrl"].as_str() {
        vid.audio_link = Some(url.into());
    } else if let Some(urls) = json_resp["url"].as_array() {
        vid.audio_link = Some(
            urls.iter()
                .map(|url| url.as_str().unwrap())
                .find(|url| url.contains(".scdn.co/"))
                .expect("Failed to get audio link")
                .into(),
        );
    }

    Ok(vid)
}
