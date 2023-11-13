use std::error::Error;

use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn rumble(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: url.into(),
        ..Default::default()
    };

    {
        let resp = get_isahc(url, vid.user_agent, url).await?;

        static RE_ID: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"href="https://rumble.com/api/Media/oembed.json\?url=https%3A%2F%2Frumble.com%2Fembed%2F(.*?)%2F""#).unwrap()
        });
        vid.vid_link = format!(
            "https://rumble.com/embedJS/u3/?request=video&ver=2&v={}",
            &RE_ID.captures(&resp).expect("Failed to get id")[1]
        )
        .into();
    }

    let resp = get_isahc(&vid.vid_link, vid.user_agent, &vid.referrer).await?;

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""title":"([^"]*)"#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].into();

    if resp.contains(r#""mp4":{"#) || resp.contains(r#""webm":{"#) {
        static RE_VID_MP4: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"\{"url":"([^"]*)","meta":\{"bitrate":([0-9]*)"#).unwrap());

        vid.vid_link = RE_VID_MP4
            .captures_iter(&resp)
            .max_by_key(|cap| cap[2].parse::<u32>().expect("Failed to parse quality"))
            .map(|cap| cap[1].into())
            .unwrap()
    } else {
        static RE_VID_HLS: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"\{"hls":\{"url":"([^"]*)"#).unwrap());

        let url: Box<str> = RE_VID_HLS
            .captures(&resp)
            .expect("Failed to get the hls link too")[1]
            .into(); // to drop resp

        drop(resp);
        let resp = get_isahc(&url, vid.user_agent, &vid.referrer).await?;

        vid.vid_link = resp.lines().last().unwrap().into();
    }

    Ok(vid)
}
