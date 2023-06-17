use crate::{helpers::reqwests::get_html_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn substack(url: &str) -> Vid {
    let mut vid = Vid {
        referrer: String::from(url),
        ..Default::default()
    };

    let resp = get_html_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"\\"publication_id\\":([0-9]*).*?\\"title\\":\\"([^"]*)\\".*?\\"video_upload_id\\":\\"([^"]*)\\""#).unwrap()
    });
    vid.title = RE.captures(&resp).expect("Failed to get title")[2].to_string();
    vid.link = format!(
        "https://corbettreport.substack.com/api/v1/video/upload/{}/src?override_publication_id={}",
        &RE.captures(&resp).expect("Failed to get video_upload_id")[3],
        &RE.captures(&resp).expect("Failed to get publication_id")[1],
    );

    vid
}
