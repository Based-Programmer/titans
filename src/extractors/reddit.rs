use crate::{helpers::reqwests::get_html_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn reddit(url: &str) -> Vid {
    static RE_LINK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"https://((libreddit|teddit)\.[^/]*|(www\.|old\.)?reddit\.com|redd\.it)(.*)")
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

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""title": "(.*?)", ""#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].replace(r#"\""#, "");

    static DASH_LINK: Lazy<Regex> = Lazy::new(|| Regex::new(r#""dash_url": "([^"]*)"#).unwrap());
    let dash_link = DASH_LINK.captures(&resp).expect("Failed to get link")[1].to_string();

    static VID_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r#""fallback_url": "([^"]*)"#).unwrap());

    vid.vid_link = if let Some(link) = VID_URL.captures(&resp) {
        link[1].to_string()
    } else {
        static DASH_VID: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"<BaseURL>(DASH_[0-9]*(\.mp4)?)</BaseURL>").unwrap());

        let best_video = &DASH_VID
            .captures_iter(&resp)
            .max_by_key(|resolution| {
                resolution[1]
                    .trim_start_matches("DASH_")
                    .trim_end_matches(".mp4")
                    .parse::<u16>()
                    .expect("Dash video quality not a number")
            })
            .expect("Failed to get dash video")[1];

        dash_link.replace("DASHPlaylist.mpd", best_video)
    };

    resp = get_html_isahc(&dash_link, &vid.user_agent, &vid.referrer).await;

    vid.audio_link = if resp.contains("<BaseURL>DASH_audio.mp4</BaseURL>") {
        Some(dash_link.replace("DASHPlaylist.mpd", "DASH_audio.mp4"))
    } else if resp.contains("<BaseURL>audio</BaseURL>") {
        Some(dash_link.replace("DASHPlaylist.mpd", "audio"))
    } else {
        static RE_DASH_AUDIO: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"<BaseURL>(DASH_AUDIO_[0-9]*(\.mp4)?)</BaseURL>").unwrap());

        if let Some(audio_link) = RE_DASH_AUDIO.captures_iter(&resp).max_by_key(|resolution| {
            resolution[1]
                .trim_start_matches("DASH_AUDIO_")
                .trim_end_matches(".mp4")
                .parse::<u16>()
                .expect("Dash audio bitrate not a number")
        }) {
            Some(dash_link.replace("DASHPlaylist.mpd", audio_link.get(1).unwrap().as_str()))
        } else {
            Some(String::new())
        }
    };

    vid.referrer = vid.referrer.trim_end_matches(".json").to_string();

    vid
}
