use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use url::Url;

pub async fn streamtape(url: &str, streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: format!("https://streamtape.xyz{}", Url::parse(url)?.path()).into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer).await?;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<meta name="og:title" content="([^"]*)"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).unwrap()[1].into();

    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"<div id="(no)?robotlink".*?/(streamtape\.xyz/.*?)&token[\s\S]*?(&token=[^']*)"#,
        )
        .unwrap()
    });

    let captures = RE.captures(&resp).expect("Failed to get video link");
    vid.vid_link = {
        if streaming_link {
            format!("https://{}{}&stream=1", &captures[2], &captures[3])
        } else {
            format!("https://{}{}&dl=1", &captures[2], &captures[3])
        }
    }
    .into();

    Ok(vid)
}
