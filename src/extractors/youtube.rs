use crate::Vid;
use isahc::{AsyncReadResponseExt, Request, RequestExt};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn youtube(url: &str) -> Vid {
    let (_, mut id) = url
        .rsplit_once("v=")
        .unwrap_or(url.rsplit_once('/').expect("Invalid Youtube url"));

    id = id.split('&').next().unwrap_or_default();

    let mut vid = Vid {
        user_agent: String::from("com.google.android.youtube/17.31.35 (Linux; U; Android 11) gzip"),
        referrer: format!("https://www.youtube.com/watch?v={}", id),
        ..Default::default()
    };

    let json = format!(
        r#"{{
  "context": {{
    "client": {{
      "clientName": "ANDROID",
      "clientVersion": "17.31.35",
      "userAgent": "{}",
    }}
  }},
  "videoId": "{}",
  "params": "8AEB",
}}"#,
        vid.user_agent, id
    );

    let resp = Request::post("https://www.youtube.com/youtubei/v1/player")
        .header("user-agent", &vid.user_agent)
        .header("referrer", &vid.referrer)
        .header("content-type", "application/json")
        .body(json)
        .unwrap()
        .send_async()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    //println!("{}", resp);

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""itag": 22,\n.*"url": "(.*?)","#).unwrap());
    vid.vid_link = RE.captures(&resp).expect("Failed to get the video link")[1].to_string();

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""title": "(.*?)","#).unwrap());
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get the title")[1].to_string();
    vid
}
