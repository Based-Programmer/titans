use crate::{helpers::unescape_html_chars::unescape_html_chars, Vid};
use isahc::{AsyncReadResponseExt, Request, RequestExt};
use serde_json::{from_str, json, Value};
use std::{
    error::Error,
    fs::{read_to_string, File},
    io::prelude::*,
    time::{SystemTime, UNIX_EPOCH},
};
use url::{form_urlencoded::byte_serialize, Url};

pub async fn twatter(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: format!("https://twitter.com{}", Url::parse(url)?.path()).into(),
        ..Default::default()
    };

    let guest_token = match read_to_string("/tmp/twatter_guest_token") {
        Ok(token) => {
            let (last_time, gt) = token.split_once(' ').unwrap();

            if current_time()? - last_time.parse::<u64>().unwrap() <= 3600 {
                gt.into()
            } else {
                fetch_guest_token(&vid).await?
            }
        }
        Err(_) => fetch_guest_token(&vid).await?,
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

        let resp = Request::get(&*api)
        .header("user-agent", vid.user_agent)
        .header("referer", &*vid.referrer)
        .header("content-type", "application/json")
        .header("authorization", "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .header("x-guest-token", &*guest_token)
        .body(()).unwrap()
        .send_async().await.unwrap()
        .text().await.unwrap()
        .into_boxed_str();

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

    vid.vid_link = data["data"]["tweetResult"]["result"]["legacy"]["extended_entities"]["media"][0]
        ["video_info"]["variants"]
        .as_array()
        .expect("Failed to convert variants to array")
        .iter()
        .max_by_key(|variant| {
            variant["bitrate"]
                .to_string()
                .parse::<u32>()
                .unwrap_or_default()
        })
        .map(|variant| {
            variant["url"]
                .as_str()
                .expect("Failed to get url from the json")
        })
        .expect("Failed to get video link")
        .into();

    Ok(vid)
}

async fn fetch_guest_token(vid: &Vid) -> Result<Box<str>, Box<dyn Error>> {
    let guest_token = {
        const TWATTER_GUEST_TOKEN_API: &str = "https://api.twitter.com/1.1/guest/activate.json";

        let resp = Request::post(TWATTER_GUEST_TOKEN_API)
        .header("user-agent", vid.user_agent)
        .header("authorization", "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .body(())?
        .send_async()
        .await?
        .text()
        .await?
        .into_boxed_str();

        let data: Value = from_str(&resp).expect("Failed to serialize guest token json");

        data["guest_token"]
            .as_str()
            .expect("Failed to get guest token")
            .into()
    };

    {
        let current_time = current_time()?;

        match File::create("/tmp/twatter_guest_token") {
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
