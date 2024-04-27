use crate::{
    helpers::reqwests::{client, get_isahc_client},
    Vid,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use url::Url;

pub fn reddit(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: format!(
            "https://www.reddit.com{}",
            Url::parse(&format!("https://{}", url))?.path()
        )
        .into(),
        ..Default::default()
    };

    let client = &client(vid.user_agent, &vid.referrer)?;
    let resp = {
        let json_url = format!("{}.json", vid.referrer);
        get_isahc_client(client, &json_url)?
    };

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""title": "(.*?)", ""#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1]
        .replace(r#"\""#, "")
        .into();

    static DASH_LINK: Lazy<Regex> = Lazy::new(|| Regex::new(r#""dash_url": "([^"]*)"#).unwrap());
    let dash_link: Box<str> = DASH_LINK.captures(&resp).expect("Failed to get link")[1].into();

    static VID_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r#""fallback_url": "([^"]*)"#).unwrap());

    vid.vid_link = if let Some(link) = VID_URL.captures(&resp) {
        link[1].into()
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

        dash_link.replacen("DASHPlaylist.mpd", best_video, 1).into()
    };

    drop(resp);
    let resp = get_isahc_client(client, &dash_link)?;

    vid.audio_link = if resp.contains("<BaseURL>DASH_audio.mp4</BaseURL>") {
        Some(
            dash_link
                .replacen("DASHPlaylist.mpd", "DASH_audio.mp4", 1)
                .into(),
        )
    } else if resp.contains("<BaseURL>audio</BaseURL>") {
        Some(dash_link.replacen("DASHPlaylist.mpd", "audio", 1).into())
    } else {
        static RE_DASH_AUDIO: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"<BaseURL>(DASH_AUDIO_[0-9]*(\.mp4)?)</BaseURL>").unwrap());

        RE_DASH_AUDIO
            .captures_iter(&resp)
            .max_by_key(|resolution| {
                resolution[1]
                    .trim_start_matches("DASH_AUDIO_")
                    .trim_end_matches(".mp4")
                    .parse::<u16>()
                    .expect("Dash audio bitrate not a number")
            })
            .map(|audio_link| {
                dash_link
                    .replacen("DASHPlaylist.mpd", &audio_link[1], 1)
                    .into()
            })
    };

    Ok(vid)
}
