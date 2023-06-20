use crate::{helpers::reqwests::get_html_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn rumble(url: &str) -> Vid {
    let mut vid = Vid {
        referrer: url.to_string(),
        ..Default::default()
    };

    let mut resp = get_html_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE_ID: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"href="https://rumble.com/api/Media/oembed.json\?url=https%3A%2F%2Frumble.com%2Fembed%2F(.*?)%2F""#).unwrap()
    });
    vid.vid_link = format!(
        "https://rumble.com/embedJS/u3/?request=video&ver=2&v={}",
        &RE_ID.captures(&resp).expect("Failed to get id")[1]
    );

    resp = get_html_isahc(&vid.vid_link, &vid.user_agent, &vid.referrer).await;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""title":"([^"]*)"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].to_string();

    if resp.contains(r#""mp4":{"#) {
        static RE_VID_MP4: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#""([0-9]*)":\{"url":"([^"]*)"#).unwrap());

        if let Some(cap) = RE_VID_MP4
            .captures_iter(&resp)
            .max_by_key(|cap| cap[1].parse::<u32>().expect("Failed to parse quality"))
            .and_then(|cap| cap.get(2))
        {
            vid.vid_link = cap.as_str().to_string();
        } else {
            vid.vid_link = get_hls(resp, &vid).await;
        }
    } else {
        vid.vid_link = get_hls(resp, &vid).await;
    }

    vid
}

async fn get_hls(mut resp: String, vid: &Vid) -> String {
    static RE_VID_HLS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"\{"hls":\{"url":"([^"]*)"#).unwrap());

    let url = &RE_VID_HLS
        .captures(&resp)
        .expect("Failed to get the hls link too")[1];

    resp = get_html_isahc(url, &vid.user_agent, &vid.referrer).await;

    resp.lines().last().unwrap().to_string()
}
