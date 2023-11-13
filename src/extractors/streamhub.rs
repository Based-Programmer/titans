use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn streamhub(url: &str, streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    const BASE_URL: &str = "streamhub.ink/";

    let mut vid = Vid {
        referrer: url
            .replace("/e/", "/")
            .replace("streamhub.to/", BASE_URL)
            .into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer).await?;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(" *<h4>(.*?)</h4>").unwrap());
    vid.title = RE_TITLE.captures(&resp).unwrap()[1].into();
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\|height\|width\|([^|]*).*?urlset\|([^|]*).*?([^|]*)?\|hls").unwrap()
    });

    let captures = RE.captures(&resp).expect("Failed to get video link");

    vid.vid_link = {
        if streaming_link {
            format!(
                "https://{}.{}hls/{}{}/index-v1-a1.m3u8",
                &captures[1], BASE_URL, &captures[3], &captures[2]
            )
        } else {
            format!(
                "https://{}.{}{}{}/",
                &captures[1], BASE_URL, &captures[3], &captures[2]
            )
        }
    }
    .into();

    Ok(vid)
}
