use std::error::Error;

use crate::{
    helpers::{reqwests::get_isahc, unescape_html_chars::unescape_html_chars},
    Vid,
};
use once_cell::sync::Lazy;
use regex::Regex;

pub fn streamvid(url: &str, streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        // referrer: url.replacen("streamvid.net/", "streamvid.media/", 1).into(),
        referrer: format!("https://{}", url).into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer)?;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<h6 class="card-title">(.*?)</h6>"#).unwrap());
    vid.title = unescape_html_chars(&RE_TITLE.captures(&resp).unwrap()[1]);

    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"\|html\|embed\|if\|?\|?(false\|?)?(on)?\|([^|]*)\|?\|?(ausfile)?\|?\|([^|]*).*urlset\|([^|]*).*?([^|]*)?\|hls",
        )
        .unwrap()
    });

    let captures = RE.captures(&resp).expect("Failed to get video link");
    let mut subdomain = &captures[5];
    let mut tld = &captures[3];

    if subdomain == "vvplay" {
        subdomain = tld;
        tld = "net";
    }

    let domain_name = if let Some(domain_name) = captures.get(4) {
        domain_name.as_str()
    } else {
        "streamvid"
    };

    vid.vid_link = {
        if streaming_link {
            format!(
                "https://{}.{}.{}/hls/{}{}/index-v1-a1.m3u8",
                subdomain, domain_name, tld, &captures[7], &captures[6]
            )
        } else {
            format!(
                "https://{}.{}.{}/{}{}/",
                subdomain, domain_name, tld, &captures[7], &captures[6]
            )
        }
    }
    .into();

    Ok(vid)
}
