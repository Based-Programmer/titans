use crate::{
    helpers::{reqwests::*, unescape_html_chars::unescape_html_chars},
    Vid, RED, RESET, YELLOW,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{Map, Value};
use std::error::Error;

pub fn rumble(url: &str, resolution: u16) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        user_agent: "Mozilla/5.0 FurryFox",
        referrer: format!("https://{}", url).into(),
        ..Default::default()
    };

    let client = &client(vid.user_agent, &vid.referrer)?;
    let data: Value = {
        let resp = get_isahc_client(client, &vid.referrer)?;

        static RE_ID: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"href="https://rumble.com/api/Media/oembed.json\?url=https%3A%2F%2Frumble.com%2Fembed%2F(.*?)%2F""#).unwrap()
        });
        let id_link = format!(
            "https://rumble.com/embedJS/u3/?request=video&ver=2&v={}",
            &RE_ID.captures(&resp).expect("Failed to get id")[1]
        )
        .into_boxed_str();
        drop(resp);

        get_isahc_json(client, &id_link)?
    };

    vid.title = unescape_html_chars(data["title"].as_str().expect("Failed to get title"));

    if let Some(qualities) = data["ua"]["mp4"].as_object() {
        (vid.vid_link, vid.resolution) = get_vid_url(&data, qualities, resolution, "mp4");
    } else if let Some(qualities) = data["ua"]["webm"].as_object() {
        (vid.vid_link, vid.resolution) = get_vid_url(&data, qualities, resolution, "webm");
    } else if let Some(url) = data["u"]["hls"]["url"].as_str() {
        let url: Box<str> = url.into();
        drop(data);

        let resp = get_isahc_client(client, &url)?;
        let mut last_line = String::new();

        for line in resp.lines() {
            if line.contains(&format!("_{resolution}p")) && line.ends_with(".m3u8") {
                vid.vid_link = line.into();
                vid.resolution = Some(resolution);
                return Ok(vid);
            }

            line.clone_into(&mut last_line);
        }

        vid.vid_link = last_line.into();
    }

    Ok(vid)
}

fn get_vid_url(
    data: &Value,
    qualities: &Map<String, Value>,
    resolution: u16,
    vid_format: &str,
) -> (Box<str>, Option<u16>) {
    let mut vid_quality: u16 = 0;

    for (quality, _) in qualities {
        match quality.parse() {
            Ok(quality) => {
                if quality == resolution {
                    vid_quality = resolution;
                    break;
                } else if quality > vid_quality {
                    vid_quality = quality;
                }
            }
            Err(_) => eprintln!("{RED}Couldn't parse quality:{YELLOW} {quality}{RESET}"),
        }
    }

    let vid_link = data["ua"][vid_format][vid_quality.to_string()]["url"]
        .as_str()
        .expect("Couldn't get vid url")
        .into();

    (vid_link, Some(vid_quality))
}
