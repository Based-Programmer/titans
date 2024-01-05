use crate::{helpers::reqwests::get_isahc, Vid};
use std::error::Error;

pub async fn substack(url: &str) -> Result<Vid, Box<dyn Error>> {
    let mut vid = Vid {
        referrer: url.into(),
        ..Default::default()
    };

    let resp = get_isahc(&vid.referrer, vid.user_agent, &vid.referrer).await?;

    vid.title = splitter(&resp, r#"\"title\":\""#, "title").into();

    if let Some(audio_split) = resp.split_once("<audio src=\"") {
        vid.audio_link = Some(audio_split.1.split_once('"').unwrap().0.into());
    } else {
        let video_upload_id = splitter(&resp, r#"\"video_upload_id\":\""#, "video or audio link");

        vid.vid_link = format!(
            "https://corbettreport.substack.com/api/v1/video/upload/{}/src",
            video_upload_id,
        )
        .into();
    }

    Ok(vid)
}

fn splitter<'a>(resp: &'a str, first: &'a str, msg: &'a str) -> &'a str {
    resp.split_once(first)
        .unwrap_or_else(|| panic!("Failed to get {}", msg))
        .1
        .split_once(r#"\""#)
        .unwrap()
        .0
}
