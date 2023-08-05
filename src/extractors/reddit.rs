use crate::{helpers::reqwests::get_html_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn reddit(url: &str) -> Vid {
    static RE_LINK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"https://((libreddit|teddit)\.[^/]*|(www\.|old\.)?reddit\.com|redd\.it)(.*)"#)
            .unwrap()
    });

    let mut vid = Vid {
        referrer: format!(
            "https://www.reddit.com{}.json",
            &RE_LINK.captures(url).expect("Illegal url")[4].trim_end_matches('/')
        ),
        ..Default::default()
    };

    let mut resp = get_html_isahc(&vid.referrer, &vid.user_agent, &vid.referrer).await;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""title": "([^"]*)"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].to_string();

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""dash_url": "([^"]*)"#).unwrap());
    vid.vid_link = RE.captures(&resp).expect("Failed to get link")[1].to_string();

    static RE_DASH_VID: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"<BaseURL>(DASH_[0-9]*\.mp4)</BaseURL>"#).unwrap());

    resp = get_html_isahc(&vid.vid_link, &vid.user_agent, &vid.referrer).await;

    if resp.contains("<BaseURL>DASH_audio.mp4</BaseURL>") {
        vid.audio_link = vid.vid_link.replace("DASHPlaylist.mpd", "DASH_audio.mp4")
    }

    let best_video = &RE_DASH_VID
        .captures_iter(&resp)
        .last()
        .expect("Failed to get dash video")[1];

    vid.vid_link = vid.vid_link.replace("DASHPlaylist.mpd", best_video);

    vid.referrer = vid.referrer.replace(".json", "/");

    vid
}
