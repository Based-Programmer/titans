use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub fn vtube(url: &str, is_streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    const BASE_URL: &str = "vtube.network/";

    let mut vid = Vid {
        referrer: format!("https://{}", url)
            .replacen("vtbe.to/", BASE_URL, 1)
            .into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<h3 class="h4 mb-4 text-center">(.*)</h3>"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).unwrap()[1].into();

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r".*\|(.*?)\|(.*?)\|hls\|(.*?)\|").unwrap());

    let captures = RE.captures(&resp).expect("Failed to get video link");

    let mut seg = &captures[1];
    if seg == "urlset" {
        seg = ""
    }

    vid.vid_link = {
        if is_streaming_link {
            format!(
                "https://{}.{}hls/{}{}/index-v1-a1.m3u8",
                &captures[3], BASE_URL, &captures[2], seg
            )
        } else {
            format!(
                "https://{}.{}{}{}/",
                &captures[3], BASE_URL, &captures[2], seg
            )
        }
    }
    .into();

    Ok(vid)
}
