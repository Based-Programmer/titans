use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn streamvid(url: &str, streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: url.replace("streamvid.net/", "streamvid.media/").into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer).await?;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<h6 class="card-title">(.*?)</h6>"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).unwrap()[1].into();

    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"adb\|html\|embed\|if(\|?\|?(false\|?)?(on)?\|?)?\|([^|]*)\|?\|([^|]*).*urlset\|([^|]*).*?([^|]*)?\|hls",
        )
        .unwrap()
    });

    let captures = RE.captures(&resp).expect("Failed to get video link");
    let mut subdomain = &captures[5];
    let mut tld = &captures[4];

    if subdomain == "vvplay" {
        subdomain = tld;
        tld = "net";
    }

    vid.vid_link = {
        if streaming_link {
            format!(
                "https://{}.streamvid.{}/hls/{}{}/index-v1-a1.m3u8",
                subdomain, tld, &captures[7], &captures[6]
            )
        } else {
            format!(
                "https://{}.streamvid.{}/{}{}/",
                subdomain, tld, &captures[7], &captures[6]
            )
        }
    }
    .into();

    Ok(vid)
}
