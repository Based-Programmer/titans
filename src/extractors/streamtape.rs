use crate::{helpers::reqwests::get_html_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn streamtape(url: &str, is_streaming_link: bool) -> Vid {
    const BASE_URL: &str = "https://streamtape.to/";

    let mut vid = Vid {
        referrer: url.replace("https://streamtape.com/", BASE_URL),
        ..Default::default()
    };

    let resp = get_html_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE_TITLE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<meta name="og:title" content="([^"]*)"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).unwrap()[1].to_string();
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"<div id="(no)?robotlink".*?(/streamtape\.to/.*?)&token[\s\S]*?(&token=[^']*)"#,
        )
        .unwrap()
    });

    vid.vid_link = if is_streaming_link {
        format!(
            "http:/{}{}&stream=1",
            &RE.captures(&resp).expect("Failed to get the link")[2],
            &RE.captures(&resp).expect("Failed to get the link")[3]
        )
    } else {
        format!(
            "https:/{}{}&dl=1",
            &RE.captures(&resp).expect("Failed to get the link")[2],
            &RE.captures(&resp).expect("Failed to get the link")[3]
        )
    };

    vid
}
