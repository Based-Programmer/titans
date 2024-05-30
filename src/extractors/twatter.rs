use crate::{
    helpers::{
        reqwests::get_isahc_client, tmp_path::tmp_path, unescape_html_chars::unescape_html_chars,
    },
    Vid, RED, RESET, YELLOW,
};
use isahc::{
    config::{Configurable, VersionNegotiation},
    HttpClient, ReadResponseExt, Request,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, to_string, Value};
use std::{
    error::Error,
    fs::{read_to_string, File},
    io::prelude::*,
    process::exit,
    time::{SystemTime, UNIX_EPOCH},
};
use url::{form_urlencoded::byte_serialize, Url};

pub fn twatter(url: &str, resolution: u16, streaming_link: bool) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: format!(
            "https://twitter.com{}",
            Url::parse(&format!("http://{}", url))?.path()
        )
        .into(),
        ..Default::default()
    };

    let client = HttpClient::new()?;

    let data: Value = {
        let guest_token = {
            let tmp_path = tmp_path(true)?.into_boxed_str();

            match read_to_string(&*tmp_path) {
                Ok(token) => {
                    let (last_time, gt) = token.split_once(' ').unwrap();

                    if current_time()? - last_time.parse::<u64>()? <= 1770 {
                        gt.parse()?
                    } else {
                        drop(token);
                        fetch_guest_token(&client, &vid, &tmp_path)?
                    }
                }
                Err(_) => fetch_guest_token(&client, &vid, &tmp_path)?,
            }
        };

        let api: Box<str> = {
            let id = vid
                .referrer
                .rsplit_once('/')
                .expect("Invalid url - Failed to get tweetId")
                .1;

            let variables = json!({
                "tweetId": id,
                "withCommunity": false,
                "includePromotedContent": false,
                "withVoice": false
            });

            const FEATURES: &str = r#"{
    "creator_subscriptions_tweet_preview_api_enabled": true,
    "tweetypie_unmention_optimization_enabled": true,
    "responsive_web_edit_tweet_api_enabled": true,
    "graphql_is_translatable_rweb_tweet_is_translatable_enabled": true,
    "view_counts_everywhere_api_enabled": true,
    "longform_notetweets_consumption_enabled": true,
    "responsive_web_twitter_article_tweet_consumption_enabled": false,
    "tweet_awards_web_tipping_enabled": false,
    "freedom_of_speech_not_reach_fetch_enabled": true,
    "standardized_nudges_misinfo": true,
    "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": true,
    "longform_notetweets_rich_text_read_enabled": true,
    "longform_notetweets_inline_media_enabled": true,
    "responsive_web_graphql_exclude_directive_enabled": true,
    "verified_phone_label_enabled": true,
    "responsive_web_media_download_video_enabled": false,
    "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
    "responsive_web_graphql_timeline_navigation_enabled": true,
    "responsive_web_enhance_cards_enabled": false
    }"#;

            const FIELDS: &str = r#"{
    "withArticleRichContentState": false,
    "withAuxiliaryUserLabels": false
    }"#;

            format!(
        "https://twitter.com/i/api/graphql/0hWvDhmW8YQ-S_ib3azIrw/TweetResultByRestId?variables={}&features={}&fieldToggles={}",
        byte_serialize(to_string(&variables)?.as_bytes()).collect::<String>(),
        byte_serialize(FEATURES.as_bytes()).collect::<String>(),
        byte_serialize(FIELDS.as_bytes()).collect::<String>(),
    ).into()
        };

        let req = Request::get(&*api)
        .header("user-agent", vid.user_agent)
        .header("content-type", "application/json")
        .header("authorization", "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .header("x-guest-token", guest_token)
        .version_negotiation(VersionNegotiation::http2())
        .body(())?;

        client.send(req)?.json()?
    };

    {
        let title_data = data["data"]["tweetResult"]["result"]["legacy"]["full_text"]
            .as_str()
            .expect("Failed to get title");

        let title = title_data
            .rsplit_once(" https://t.co/")
            .unwrap_or((title_data, ""))
            .0;

        vid.title = unescape_html_chars(title);
    }

    let variants = data["data"]["tweetResult"]["result"]["legacy"]["extended_entities"]["media"][0]
        ["video_info"]["variants"]
        .as_array()
        .expect("Failed to convert variants to array");

    if !streaming_link {
        let variants = variants.iter().skip(1);

        if resolution != 0 {
            for variant in variants.clone() {
                let url = variant["url"]
                    .as_str()
                    .expect("Failed to get url from json");

                let (res1, res2) = url
                    .rsplit_once("/vid/avc1/")
                    .expect("Failed to get pattern '/vid/avc1/' in url")
                    .1
                    .split_once('/')
                    .expect("Failed to split at '/' in url")
                    .0
                    .split_once('x')
                    .expect("Failed to get mp4 resolution");

                let res1: u16 = res1
                    .parse()
                    .expect("Failed to convert resolution 1 to number");
                let res2: u16 = res2
                    .parse()
                    .expect("Failed to convert resolution 2 to number");

                if resolution == res1 || resolution == res2 {
                    vid.vid_link = url.into();
                }
            }
        }

        if vid.vid_link.is_empty() {
            vid.vid_link = variants
                .max_by_key(|variant| variant["bitrate"].as_u64())
                .map(|variant| {
                    variant["url"]
                        .as_str()
                        .expect("Failed to get url from the json")
                })
                .expect("Failed to get video link")
                .into();
        }
    } else {
        let m3u8: Box<str> = variants[0]["url"]
            .as_str()
            .expect("Failed to get url from the json")
            .into();

        drop(data);

        let resp = get_isahc_client(&client, &m3u8)?;

        if resolution == 0 {
            best_link(&resp, &mut vid)
        }

        if vid.vid_link.is_empty() {
            static RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"#EXT-X-STREAM-INF:.*?RESOLUTION=([0-9]*)x([0-9]*).*\n(.*\.m3u8)")
                    .unwrap()
            });

            for captures in RE.captures_iter(&resp) {
                let res_str = resolution.to_string();

                if res_str == captures[1] || res_str == captures[2] {
                    vid.vid_link = format!("https://video.twimg.com{}", &captures[3]).into();
                    break;
                }
            }

            if vid.vid_link.is_empty() {
                eprintln!("{RED}Failed to get the video link of desired resolution{RESET}");

                if resolution == 0 {
                    exit(1);
                } else {
                    eprintln!("{YELLOW}Trying to get the best video link{RESET}");
                    best_link(&resp, &mut vid)
                }
            } else {
                audio_link(&resp, &mut vid)
            }
        }
    }

    Ok(vid)
}

fn best_link(resp: &str, vid: &mut Vid) {
    vid.vid_link = format!("https://video.twimg.com{}", resp.lines().last().unwrap()).into();
    audio_link(resp, vid)
}

fn audio_link(resp: &str, vid: &mut Vid) {
    let (mut audio_link, mut audio_line) = Default::default();

    for line in resp.lines() {
        if line.starts_with(r#"#EXT-X-MEDIA:NAME="Audio","#) {
            audio_line = line;
        }
    }

    if !audio_line.is_empty() {
        audio_link = audio_line
            .rsplit_once("URI=\"")
            .unwrap()
            .1
            .trim_end_matches('"');
    }

    vid.audio_link = Some(format!("https://video.twimg.com{}", audio_link).into());
}

fn fetch_guest_token(
    client: &HttpClient,
    vid: &Vid,
    tmp_path: &str,
) -> Result<u64, Box<dyn Error>> {
    let guest_token = {
        let req = Request::post("https://api.twitter.com/1.1/guest/activate.json")
        .header("user-agent", vid.user_agent)
        .header("authorization", "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .version_negotiation(VersionNegotiation::http2())
        .body(())?;

        client.send(req)?.json::<Value>()?["guest_token"]
            .as_str()
            .expect("Failed to get guest token")
            .parse()?
    };

    {
        let current_time = current_time()?;

        match File::create(tmp_path) {
            Ok(mut file) => file.write_all(format!("{current_time} {guest_token}").as_bytes())?,
            Err(_) => eprintln!("{RED}Failed to create file{RESET}"),
        }
    }

    Ok(guest_token)
}

fn current_time() -> Result<u64, Box<dyn Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
