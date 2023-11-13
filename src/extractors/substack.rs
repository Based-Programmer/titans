use crate::{helpers::reqwests::get_isahc, Vid};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{error::Error, process::exit};

pub async fn substack(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: url.into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer).await?;

    if resp.contains(r#"\"type\":\"video\""#) {
        static RE_VIDEO: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"\\"publication_id\\":([0-9]*).*?\\"title\\":\\"([^"]*)\\".*?\\"video_upload_id\\":\\"([^"]*)\\""#).unwrap()
        });

        vid.title = RE_VIDEO.captures(&resp).expect("Failed to get title")[2].into();
        vid.vid_link = format!(
        "https://corbettreport.substack.com/api/v1/video/upload/{}/src?override_publication_id={}",
        &RE_VIDEO.captures(&resp).expect("Failed to get video_upload_id")[3],
        &RE_VIDEO.captures(&resp).expect("Failed to get publication_id")[1],
    )
        .into();
    } else if resp.contains(r#"\"type\":\"podcast\""#) {
        static RE_AUDIO: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<audio src="([^"]*)"#).unwrap());
        vid.audio_link = Some(RE_AUDIO.captures(&resp).expect("Failed to get audio")[1].into());

        static RE_TITLE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"\\"title\\":\\"([^"]*)\\"#).unwrap());
        vid.title = RE_TITLE.captures(&resp).expect("Failed to get title")[1].into();
    } else {
        eprintln!("Failed to get video or audio link\nCheck if its a article only link");
        exit(1);
    }

    Ok(vid)
}
