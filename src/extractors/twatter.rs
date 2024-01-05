use crate::{helpers::unescape_html_chars::unescape_html_chars, Vid};
use isahc::{
    config::{Configurable, VersionNegotiation},
    AsyncReadResponseExt, HttpClient, Request,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{from_str, json, Value};
use std::{
    env::consts::OS,
    error::Error,
    fs::{read_to_string, File},
    io::prelude::*,
    time::{SystemTime, UNIX_EPOCH},
};
use url::{form_urlencoded::byte_serialize, Url};

pub async fn twatter(
    url: &str,
    resolution: &str,
    streaming_link: bool,
) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: format!("https://twitter.com{}", Url::parse(url)?.path()).into(),
        ..Default::default()
    };

    let client = HttpClient::new()?;

    let tmp_path = if OS == "android" {
        "/data/data/com.termux/files/usr/tmp/twatter_guest_token"
    } else {
        "/tmp/twatter_guest_token"
    };

    let guest_token = match read_to_string(tmp_path) {
        Ok(token) => {
            let (last_time, gt) = token.split_once(' ').unwrap();

            if current_time()? - last_time.parse::<u64>().unwrap() <= 3600 {
                gt.into()
            } else {
                fetch_guest_token(&client, &vid, tmp_path).await?
            }
        }
        Err(_) => fetch_guest_token(&client, &vid, tmp_path).await?,
    };

    let data: Value = {
        let api: Box<str> = {
            let id = vid
                .referrer
                .rsplit_once('/')
                .expect("Invalid url - Failed to get tweetId")
                .1;

            let variables = json!({
                "tweetId": id,
                "withCommunity":false,
                "includePromotedContent":false,
                "withVoice":false
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
        byte_serialize(variables.to_string().as_bytes()).collect::<String>(),
        byte_serialize(FEATURES.as_bytes()).collect::<String>(),
        byte_serialize(FIELDS.as_bytes()).collect::<String>(),
    ).into()
        };

        let req = Request::get(&*api)
        .header("user-agent", vid.user_agent)
        .header("content-type", "application/json")
        .header("authorization", "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .header("x-guest-token", &*guest_token)
        .version_negotiation(VersionNegotiation::http2())
        .body(())?;

        let resp = client.send_async(req).await?.text().await?.into_boxed_str();

        drop(guest_token);

        from_str(&resp).expect("Failed to derive json")
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

    if !streaming_link {
        vid.vid_link = data["data"]["tweetResult"]["result"]["legacy"]["extended_entities"]
            ["media"][0]["video_info"]["variants"]
            .as_array()
            .expect("Failed to convert variants to array")
            .iter()
            .max_by_key(|variant| variant["bitrate"].as_u64())
            .map(|variant| {
                variant["url"]
                    .as_str()
                    .expect("Failed to get url from the json")
            })
            .expect("Failed to get video link")
            .into();
    } else {
        let m3u8 = data["data"]["tweetResult"]["result"]["legacy"]["extended_entities"]["media"][0]
            ["video_info"]["variants"]
            .as_array()
            .expect("Failed to convert variants to array")
            .iter()
            .map(|variant| {
                variant["url"]
                    .as_str()
                    .expect("Failed to get url from the json")
            })
            .find(|url| url.contains(".m3u8?tag="))
            .unwrap();

        let req = Request::get(m3u8)
            .header("user-agent", vid.user_agent)
            .version_negotiation(VersionNegotiation::http2())
            .body(())?;

        let resp = client.send_async(req).await?.text().await?.into_boxed_str();

        drop(data);

        if resolution == "best" {
            vid.vid_link = best_link(&resp);
        } else {
            static RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new("#EXT-X-STREAM-INF:.*?RESOLUTION=([0-9]*)x([0-9]*).*\n(.*)").unwrap()
            });

            for captures in RE.captures_iter(&resp) {
                if *resolution == captures[2] || {
                    *resolution == captures[1] && resolution == "480"
                } {
                    vid.vid_link = format!("https://video.twimg.com{}", &captures[3]).into();
                    break;
                }
            }

            if vid.vid_link.is_empty() {
                vid.vid_link = best_link(&resp)
            }
        }
    }

    Ok(vid)
}

fn best_link(resp: &str) -> Box<str> {
    format!(
        "https://video.twimg.com{}",
        resp.lines().last().expect("Failed to get last line")
    )
    .into()
}

async fn fetch_guest_token(
    client: &HttpClient,
    vid: &Vid,
    tmp_path: &str,
) -> Result<Box<str>, Box<dyn Error>> {
    let guest_token = {
        const TWATTER_GUEST_TOKEN_API: &str = "https://api.twitter.com/1.1/guest/activate.json";

        let req = Request::post(TWATTER_GUEST_TOKEN_API)
        .header("user-agent", vid.user_agent)
        .header("authorization", "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .version_negotiation(VersionNegotiation::http2())
        .body(())?;

        let resp = client.send_async(req).await?.text().await?.into_boxed_str();

        let data: Value = from_str(&resp).expect("Failed to serialize guest token json");

        data["guest_token"]
            .as_str()
            .expect("Failed to get guest token")
            .into()
    };

    {
        let current_time = current_time()?;

        match File::create(tmp_path) {
            Ok(mut file) => file
                .write_all(format!("{current_time} {guest_token}").as_bytes())
                .expect("Failed to write file"),
            Err(_) => eprintln!("Failed to create file"),
        }
    }

    Ok(guest_token)
}

fn current_time() -> Result<u64, Box<dyn Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
