use crate::Vid;
use isahc::{
    config::{Configurable, VersionNegotiation},
    ReadResponseExt, Request, RequestExt,
};
use serde_json::{from_str, json, to_string, Value};
use std::{error::Error, process::exit};

const RED: &str = "\u{1b}[31m";
const RESET: &str = "\u{1b}[0m";

pub fn youtube(
    url: &str,
    resolution: u16,
    mut vid_codec: &str,
    mut audio_codec: &str,
    is_dash: bool,
) -> Result<Vid, Box<dyn Error>> {
    let id = url
        .rsplit_once("v=")
        .unwrap_or(url.rsplit_once('/').expect("Invalid Youtube url"))
        .1
        .rsplit(|delimiter| delimiter == '?' || delimiter == '&')
        .next_back()
        .unwrap_or_default();

    let mut vid = Vid {
        user_agent: "com.google.android.youtube/17.31.35 (Linux; U; Android 11) gzip",
        referrer: format!("https://www.youtube.com/watch?v={}", id).into(),
        ..Default::default()
    };

    let data: Value = {
        let resp = {
            const CLIENT_VERSION: &str = "18.48.37";

            let json = {
                let json_value = json!({
                "context": {
                    "client": {
                        "androidSdkVersion": 34,
                        "clientName": "ANDROID",
                        "clientVersion": CLIENT_VERSION,
                        "clientScreen": "WATCH",
                        "gl": "IN",
                        "hl": "en",
                        "utcOffsetMinutes": 0,
                        "osName": "Android",
                        "osVersion": "14",
                        "platform": "MOBILE"
                    },
                    "request": {
                        "internalExperimentFlags": [],
                        "useSsl": true
                    },
                    "user": {
                        "lockedSafetyMode": false
                    },
                    "thirdParty": {
                        "embedUrl": "https://www.youtube.com/"
                    }
                },
                "videoId": id,
                "cpn": "XkQAv5c26hv4qzip",
                "racyCheckOk": true,
                "contentCheckOk": true,
                "params": "CgIQBg"
                });

                to_string(&json_value)?.into_boxed_str()
            };

            Request::post("https://www.youtube.com/youtubei/v1/player?key=AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w&prettyPrint=false")
            .header("user-agent", vid.user_agent)
            .header("referer", &*vid.referrer)
            .header("content-type", "application/json")
            .header("x-youtube-client-name", "ANDROID")
            .header("x-youtube-client-version", CLIENT_VERSION)
            .version_negotiation(VersionNegotiation::http2())
            .body(&*json)?
            .send()?
            .text()?
        };

        from_str(&resp).expect("Failed to derive json")
    };

    vid_codec = match vid_codec {
        "h264" | "libx264" => "avc",
        "av1" => "av01",
        _ => vid_codec,
    };

    if audio_codec == "aac" {
        audio_codec = "mp4a";
    }

    if !is_dash && vid_codec == "avc" {
        if let Some(formats) = data["streamingData"]["formats"].as_array() {
            exit_if_empty(formats);

            for format in formats {
                let (codec, quality, url, _) = vid_data(format)?;

                if quality == resolution {
                    let (vid_codec, audio_codec) = codec
                        .split_once(", ")
                        .expect(r#"Failed to find ", " which separates video & audio codec"#);

                    vid.vid_link = url.into();
                    vid.vid_codec = vid_codec.into();
                    vid.audio_codec = audio_codec.into();
                    vid.resolution = Some(quality);

                    break;
                }
            }
        }
    }

    if vid.vid_link.is_empty() {
        if let Some(formats) = data["streamingData"]["adaptiveFormats"].as_array() {
            exit_if_empty(formats);
            let mut bt_audio = 0;
            let mut bt_video = 0;

            let (mut v_codec, mut a_codec, mut v_link, mut a_link, mut res) = Default::default();

            for format in formats {
                let (codec, quality, url, bitrate) = vid_data(format)?;

                if codec.starts_with(vid_codec) && (bitrate > bt_video || quality == resolution) {
                    v_link = url;
                    v_codec = codec;
                    res = quality;
                    bt_video = bitrate;
                } else if codec.starts_with(audio_codec) && bitrate > bt_audio {
                    a_link = url;
                    a_codec = codec;
                    bt_audio = bitrate;
                }
            }

            vid.vid_link = v_link.into();
            vid.audio_codec = a_codec.into();
            vid.audio_link = Some(a_link.into());
            vid.resolution = Some(res);
            vid.vid_codec = v_codec.into();
        }
    }
    vid.title = data["videoDetails"]["title"]
        .as_str()
        .expect("Failed to get title")
        .into();

    Ok(vid)
}

fn vid_data(format: &Value) -> Result<(&str, u16, &str, u64), Box<dyn Error>> {
    let codec = format["mimeType"]
        .as_str()
        .expect("Failed to get mimeType")
        .split_once(r#"codecs=""#)
        .expect("Failed to get codec")
        .1
        .trim_end_matches('"');

    let mut default_audio = true; // for videos & audios which doesn't have audioTrack

    if codec.starts_with("mp4a") || codec == "opus" {
        if let Some(audio_track) = format["audioTrack"].as_object() {
            default_audio = audio_track["audioIsDefault"]
                .as_bool()
                .expect("Failed to get audioisDefault");
        }
    }

    let (quality, url, bitrate) = if default_audio {
        let quality = format["qualityLabel"]
            .as_str()
            .unwrap_or_default()
            .split_once('p')
            .unwrap_or_default()
            .0
            .parse()
            .unwrap_or_default();

        let url = format["url"].as_str().expect("Failed to get url");

        let bitrate = format["bitrate"]
            .as_u64()
            .expect("Failed to convert bitrate into a number");

        (quality, url, bitrate)
    } else {
        (0, "", 0)
    };

    Ok((codec, quality, url, bitrate))
}

fn exit_if_empty<T>(formats: &[T]) {
    if formats.is_empty() {
        eprintln!("{}No result{}", RED, RESET);
        exit(1);
    }
}
