use crate::Vid;
use isahc::{AsyncReadResponseExt, Request, RequestExt};
use serde_json::{json, Value};
use std::process::exit;

const RED: &str = "\u{1b}[31m";
const RESET: &str = "\u{1b}[0m";

pub async fn youtube(
    url: &str,
    resolution: &str,
    vid_codec: &str,
    audio_codec: &str,
    is_dash: bool,
) -> Vid {
    let id = url
        .rsplit_once("/watch?v=")
        .unwrap_or(url.rsplit_once('/').expect("Invalid Youtube url"))
        .1
        .rsplit(|delimiter| delimiter == '?' || delimiter == '&')
        .next_back()
        .unwrap_or_default();

    let mut vid = Vid {
        user_agent: String::from("com.google.android.youtube/17.31.35 (Linux; U; Android 11) gzip"),
        referrer: format!("https://www.youtube.com/watch?v={}", id),
        ..Default::default()
    };

    let json = json!({
    "contentCheckOk": true,
    "context": {
        "client": {
            "androidSdkVersion": 30,
            "clientName": "ANDROID",
            "clientVersion": "17.31.35",
            "clientScreen": "WATCH",
            "gl": "US",
            "hl": "en",
            "osName": "Android",
            "osVersion": "11",
            "platform": "MOBILE"
        },
        "user": {
            "lockedSafetyMode": false
        },
        "thirdParty": {
            "embedUrl": "https://www.youtube.com/"
        }
    },
    "videoId": id,
    "playbackContext": {
        "contentPlaybackContext": {
            "signatureTimestamp": 19250
        }
    },
    "racyCheckOk": true,
    "contentCheckOk": true,
    });

    let resp = Request::post("https://www.youtube.com/youtubei/v1/player?key=AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w&prettyPrint=false")
        .header("user-agent", &vid.user_agent)
        .header("referer", &vid.referrer)
        .header("content-type", "application/json")
        .header("x-youtube-client-name", "ANDROID")
        .header("x-youtube-client-version", "17.31.35")
        .body(json.to_string())
        .unwrap()
        .send_async()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let data: Value = serde_json::from_str(&resp).expect("Failed to derive json");

    if !is_dash && vid_codec == "avc" {
        if let Some(formats) = data["streamingData"]["formats"].as_array() {
            exit_if_empty(formats);

            formats.iter().for_each(|format| {
                let (codec, quality, url) = vid_data(format);

                if quality == resolution {
                    vid.vid_link = url;
                    let (vid_codec, audio_codec) = codec
                        .split_once(", ")
                        .expect("Failed to find , which separates video & audio codec");
                    vid.vid_codec = vid_codec.to_string();
                    vid.audio_codec = audio_codec.to_string();
                    vid.resolution = quality;
                }
            })
        }
    }

    if vid.vid_link.is_empty() {
        if let Some(formats) = data["streamingData"]["adaptiveFormats"].as_array() {
            exit_if_empty(formats);

            formats.iter().for_each(|format| {
                let (codec, quality, url) = vid_data(format);

                if codec.starts_with(vid_codec) && {
                    quality == resolution || vid.vid_link.is_empty()
                } {
                    vid.vid_link = url;
                    vid.vid_codec = codec;
                    vid.resolution = quality;
                } else if codec == audio_codec {
                    vid.audio_link = url.to_string();
                    vid.audio_codec = audio_codec.to_string();
                }
            });
        }
    }
    vid.title = data["videoDetails"]["title"]
        .as_str()
        .expect("Failed to get title")
        .to_string();

    vid
}

fn vid_data(format: &Value) -> (String, String, String) {
    let codec = format["mimeType"]
        .as_str()
        .expect("Failed to get mimeType")
        .split_once(r#"codecs=""#)
        .expect("Failed to get codec")
        .1
        .trim_end_matches('"')
        .to_string();

    let quality = format["qualityLabel"]
        .as_str()
        .unwrap_or_default()
        .split_once('p')
        .unwrap_or_default()
        .0
        .to_string();

    let url = format["url"]
        .as_str()
        .expect("Failed to get url")
        .to_string();

    (codec, quality, url)
}

fn exit_if_empty(formats: &Vec<serde_json::Value>) {
    if formats.is_empty() {
        eprintln!("{}No result{}", RED, RESET);
        exit(1);
    }
}
