use crate::Vid;
use isahc::{AsyncReadResponseExt, Request, RequestExt};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn twatter(url: &str) -> Vid {
    static RE_LINK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"https://(nitter\.[^/]*|(www\.|m\.)?twitter\.com)(/[^/]*/status/[0-9]*)"#)
            .unwrap()
    });

    let mut vid = Vid {
        referrer: format!(
            "https://twitter.com{}",
            &RE_LINK.captures(url).expect("Invalid link")[3]
        ),
        ..Default::default()
    };

    let (_, id) = vid.referrer.rsplit_once('/').expect("Invalid url");

    let api = format!(
        "https://api.twitter.com/1.1/statuses/show/{}.json?tweet_mode=extended",
        id
    );

    let resp = Request::get(api)
        .header("user-agent", &vid.user_agent)
        .header("referrer", &vid.referrer)
    .header("authorization", "bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .body(())
        .unwrap()
        .send_async()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    static RE_TITLE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#""full_text":"([^"]*?)((\\n)*| *)https:\\/\\/t\.co\\/"#).unwrap()
    });
    vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].replace("\\n", " ");

    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#""bitrate":([0-9]*).*?"url":"(https:\\/\\/video[^"]*\.mp4\?tag=[0-9]*)""#)
            .unwrap()
    });

    vid.link = RE
        .captures_iter(&resp)
        .max_by_key(|cap| cap[1].parse::<u32>().expect("Failed to parse bitrate"))
        .expect("Failed to get link")[2]
        .replace("\\/", "/");

    vid
}
