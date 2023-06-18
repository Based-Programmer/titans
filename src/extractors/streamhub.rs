use crate::{helpers::reqwests::get_html_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn streamhub(url: &str, is_streaming_link: bool) -> Vid {
    const BASE_URL: &str = "streamhub.gg/";

    let mut vid = Vid {
        referrer: url.replace("/e/", "/").replace("streamhub.to/", BASE_URL),
        ..Default::default()
    };

    let resp = get_html_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(" *<h4>(.*?)</h4>").unwrap());
    vid.title = RE_TITLE.captures(&resp).unwrap()[1].to_string();
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"gg\|([^|]*).*?urlset\|([^|]*).*?([^|]*)?\|hls"#).unwrap());

    vid.vid_link = if is_streaming_link {
        format!(
            "https://{}.{}hls/{}{}/index-v1-a1.m3u8",
            &RE.captures(&resp).unwrap()[1],
            BASE_URL,
            &RE.captures(&resp).unwrap()[3],
            &RE.captures(&resp).unwrap()[2]
        )
    } else {
        format!(
            "https://{}.{}{}{}/",
            &RE.captures(&resp).unwrap()[1],
            BASE_URL,
            &RE.captures(&resp).unwrap()[3],
            &RE.captures(&resp).unwrap()[2]
        )
    };

    vid
}
