use crate::{
    helpers::{tmp_path::tmp_path, unescape_html_chars::unescape_html_chars},
    Vid, RED, RESET,
};
use fastrand::Rng;
use isahc::{
    config::{Configurable, VersionNegotiation},
    ReadResponseExt, Request, RequestExt,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, to_string, Value};
use std::{error::Error, fs::File, io::Write, process::exit};

pub struct Chapter {
    start: u32,
    title: Box<str>,
}

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
        .split(['?', '&'])
        .next()
        .unwrap();

    let mut vid = Vid {
        user_agent: "com.google.android.apps.youtube.vr.oculus/1.57.29 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip",
        referrer: format!("https://www.youtube.com/watch?v={}", id).into(),
        ..Default::default()
    };

    let data: Value = {
        const CLIENT_VERSION: &str = "1.57.29";

        let json = {
            let rnd = Rng::new();
            let cpn: String = (0..16).map(|_| rnd.alphanumeric()).collect();

            let json_value = json!({
            "context": {
                "client": {
                    "clientName": "ANDROID_VR",
                    "clientVersion": CLIENT_VERSION,
                    "clientScreen": "WATCH",
                    "gl": "IN",
                    "hl": "en",
                    "utcOffsetMinutes": 0,
                    "deviceMake": "Oculus",
                    "deviceModel": "Quest 3",
                    "osName": "Android",
                    "osVersion": "12L",
                    "androidSdkVersion": 32,
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
            "cpn": cpn,
            "racyCheckOk": true,
            "contentCheckOk": true,
            "params": "2AMB"
            });

            to_string(&json_value)?.into_boxed_str()
        };

        Request::post("https://www.youtube.com/youtubei/v1/player?prettyPrint=false")
            .header("user-agent", vid.user_agent)
            .header("referer", &*vid.referrer)
            .header("content-type", "application/json")
            .header("x-youtube-client-name", 28)
            .header("x-youtube-client-version", CLIENT_VERSION)
            .version_negotiation(VersionNegotiation::http2())
            .body(&*json)?
            .send()?
            .json()?
    };
    // println!("{:?}", data);

    vid_codec = match vid_codec {
        "h264" | "libx264" => "avc",
        "av1" => "av01",
        _ => vid_codec,
    };

    if matches!(audio_codec, "aac" | "m4a") {
        audio_codec = "mp4a";
    }

    if !is_dash && vid_codec == "avc" {
        not_dash_link(&data, resolution, &mut vid)?;
    }

    if resolution > 1080 && vid_codec == "avc" {
        vid_codec = "vp9"
    }

    if vid.vid_link.is_empty() {
        if let Some(formats) = data["streamingData"]["adaptiveFormats"].as_array() {
            exit_if_empty(formats);
            let mut bt_audio = 0;
            let mut bt_video = 0;

            let (mut v_codec, mut a_codec, mut v_link, mut a_link, mut res) = Default::default();

            for format in formats {
                let (codec, quality, url, bitrate) =
                    vid_data(format, Some(vid_codec), Some(audio_codec))?;

                if quality > 0 && (bitrate > bt_video || quality == resolution) {
                    v_link = url;
                    v_codec = codec;
                    res = quality;
                    bt_video = bitrate;
                } else if quality == 0 && bitrate > bt_audio {
                    a_link = url;
                    a_codec = codec;
                    bt_audio = bitrate;
                }
            }

            if a_link.is_empty() {
                for format in formats {
                    let (codec, quality, url, bitrate) = vid_data(format, None, Some("mp4a"))?;
                    if quality == 0 && bitrate > bt_audio {
                        a_link = url;
                        a_codec = codec;
                        bt_audio = bitrate;
                    }
                }
            }

            if v_link.is_empty() {
                for format in formats {
                    let (codec, quality, url, bitrate) = vid_data(format, Some("avc"), None)?;

                    if quality > 0 && (bitrate > bt_video || quality == resolution) {
                        v_link = url;
                        v_codec = codec;
                        res = quality;
                        bt_video = bitrate;
                    }
                }
            }

            vid.vid_link = v_link.into();
            vid.audio_codec = Some(a_codec.into());
            vid.audio_link = Some(a_link.into());
            vid.resolution = Some(res);
            vid.vid_codec = Some(v_codec.into());
        }
    }

    vid.title = unescape_html_chars(
        data["videoDetails"]["title"]
            .as_str()
            .expect("Failed to get title"),
    );

    {
        let mut chapters = Vec::new();

        if let Some(description) = data["videoDetails"]["shortDescription"].as_str() {
            static CHAPTER_RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"\n[^\p{L}\p{N}\p{P}]*[[:space:]]*[(|{|\[]?([0-6]?[0-9]:)?([0-6]?[0-9]:[0-6]?[0-9])[)|}|\]]?([[:space:]]*[:|–|-])?[[:space:]]+(.+)")
                    .unwrap()
            });

            for chapter in CHAPTER_RE.captures_iter(description) {
                let hour = if let Some(hour) = chapter.get(1) {
                    hour.as_str()
                } else {
                    ""
                };

                chapters.push(Chapter {
                    start: timestamp_to_ms(&format!("{}{}", hour, &chapter[2]))?,
                    title: chapter[4].into(),
                });
            }
        }

        if !chapters.is_empty() {
            let file_path = format!("{}{}.txt", tmp_path(false)?, vid.title.replace('/', "\\"))
                .into_boxed_str();
            create_chapter_file(&chapters, &file_path)?;

            vid.chapter_file = Some(file_path)
        }
    }

    if vid.vid_link.is_empty() {
        not_dash_link(&data, resolution, &mut vid)?;
    }

    Ok(vid)
}

fn not_dash_link(data: &Value, resolution: u16, vid: &mut Vid) -> Result<(), Box<dyn Error>> {
    if let Some(formats) = data["streamingData"]["formats"].as_array() {
        exit_if_empty(formats);

        let (mut v_codec, mut a_codec, mut v_link, mut res) = Default::default();

        for format in formats {
            let (codec, quality, url, _) = vid_data(format, None, None)?;

            if quality > res {
                let (vid_codec, audio_codec) = codec
                    .split_once(", ")
                    .expect(r#"Failed to find ", " which separates video & audio codec"#);

                v_link = url;
                v_codec = vid_codec;
                a_codec = audio_codec;
                res = quality;

                if quality == resolution {
                    break;
                }
            }
        }

        if !v_link.is_empty() {
            vid.vid_link = v_link.into();
            vid.audio_link = None;
            vid.vid_codec = Some(v_codec.into());
            vid.audio_codec = Some(a_codec.into());
            vid.resolution = Some(res);
        }
    }

    Ok(())
}

fn vid_data<'a>(
    format: &'a Value,
    vid_codec: Option<&'a str>,
    audio_codec: Option<&'a str>,
) -> Result<(&'a str, u16, &'a str, u64), Box<dyn Error>> {
    let codec = format["mimeType"]
        .as_str()
        .expect("Failed to get mimeType")
        .split_once(r#"codecs=""#)
        .expect("Failed to get codec")
        .1
        .trim_end_matches('"');

    let default_audio = codec.starts_with(audio_codec.unwrap_or("false lmao"))
        && format["audioTrack"]
            .as_object()
            .and_then(|audio_track| audio_track["audioIsDefault"].as_bool())
            .unwrap_or(true);

    if default_audio
        || codec.starts_with(vid_codec.unwrap_or("false lmao"))
        || (vid_codec.is_none() && audio_codec.is_none())
    {
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

        // println!("{codec}\t{bitrate}");

        Ok((codec, quality, url, bitrate))
    } else {
        Ok(Default::default())
    }
}

fn exit_if_empty<T>(formats: &[T]) {
    if formats.is_empty() {
        eprintln!("{}No result{}", RED, RESET);
        exit(1);
    }
}

fn timestamp_to_ms(timestamp: &str) -> Result<u32, Box<dyn Error>> {
    let parts: Vec<&str> = timestamp.split(':').collect();

    let mut hours: u32 = 0;
    let minutes: u32;
    let seconds: u32;

    match parts.len() {
        2 => {
            minutes = parts[0].parse()?;
            seconds = parts[1].parse()?;
        }
        3 => {
            hours = parts[0].parse()?;
            minutes = parts[1].parse()?;
            seconds = parts[2].parse()?;
        }
        _ => unreachable!(),
    }

    let milliseconds = (hours * 3600 + minutes * 60 + seconds) * 1000;
    Ok(milliseconds)
}

fn create_chapter_file(chapters: &[Chapter], file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut ffmpeg_chapter = String::from(";FFMETADATA1\n");

    for chapter in chapters {
        ffmpeg_chapter.push_str(&format!(
            "[CHAPTER]
TIMEBASE=1/1000
START={}
END=
title={}\n",
            chapter.start, chapter.title
        ));
    }

    match File::create(file_path) {
        Ok(mut file) => file.write_all(ffmpeg_chapter.as_bytes())?,
        Err(_) => eprintln!("{RED}Failed to create file{RESET}"),
    }

    Ok(())
}
