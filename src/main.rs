mod extractors;
mod helpers;

use crate::extractors::{
    bitchute::bitchute, doodstream::doodstream, mp4upload::mp4upload, odysee::odysee,
    reddit::reddit, rumble::rumble, streamhub::streamhub, streamtape::streamtape,
    streamvid::streamvid, substack::substack, twatter::twatter, youtube::youtube,
};

use std::{
    env,
    process::{exit, Stdio},
};

use tokio::{fs, process::Command};

#[derive(Debug)]
pub struct Vid {
    user_agent: String,
    referrer: String,
    title: String,
    vid_link: String,
    vid_codec: String,
    resolution: String,
    audio_link: String,
    audio_codec: String,
}

impl Default for Vid {
    fn default() -> Self {
        Self {
            user_agent: String::from("uwu"),
            referrer: String::new(),
            title: String::new(),
            vid_link: String::new(),
            vid_codec: String::new(),
            resolution: String::new(),
            audio_link: String::new(),
            audio_codec: String::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    let mut vid = Vid::default();
    let mut todo = "";
    let mut is_streaming_link = true;
    let mut is_dash = true;
    let mut resolution = String::from("best");
    let mut vid_codec = String::from("avc");
    let mut audio_codec = String::from("opus");

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                help();
                exit(0);
            }
            "-g" | "--get" => todo = "print link",
            "-p" | "--play" => {
                todo = "play";
                resolution = String::from("720");
                is_dash = false;
            }
            "-pa" | "--play-audio" => todo = "play audio",
            "-d" | "--download" => {
                todo = "download";
                is_streaming_link = false;
            }
            "-D" | "--dl_link" => is_streaming_link = false,
            "-s" | "--stream_link" => is_streaming_link = true,
            "-c" | "--combined" => {
                is_dash = false;
                resolution = String::from("720");
            }
            "-b" | "--best" => resolution = String::from("best"),
            a if a.starts_with("-r=") || a.starts_with("--resolution=") => {
                resolution = arg
                    .split_once('=')
                    .unwrap()
                    .1
                    .trim_end_matches('p')
                    .to_string();
            }
            a if a.starts_with("-vc=") || a.starts_with("--vid-codec=") => {
                vid_codec = arg.split_once('=').unwrap().1.to_string();
            }
            a if a.starts_with("-ac=") || a.starts_with("--audio-codec=") => {
                audio_codec = arg.split_once('=').unwrap().1.to_string();
            }
            a if a.starts_with("https://doodstream.com/")
                || a.starts_with("https://www.doodstream.com/")
                || a.starts_with("https://dood.")
                || a.starts_with("https://doods.pro/")
                || a.starts_with("https://dooood.com/") =>
            {
                vid = doodstream(&arg, is_streaming_link).await
            }
            a if a.contains(".substack.com/p/") => vid = substack(&arg).await,
            a if a.starts_with("https://streamhub.") => {
                vid = streamhub(&arg, is_streaming_link).await
            }
            a if a.starts_with("https://youtube.com/")
                || a.starts_with("https://youtu.be/")
                || a.starts_with("https://www.youtube.com/")
                || a.starts_with("https://piped.")
                || a.starts_with("https://invidious.") =>
            {
                vid = youtube(&arg, &resolution, &vid_codec, &audio_codec, is_dash).await
            }
            a if a.starts_with("https://www.reddit.com/")
                || a.starts_with("https://old.reddit.com/")
                || a.starts_with("https://reddit.com/")
                || a.starts_with("https://redd.it/")
                || a.starts_with("https://libreddit.")
                || a.starts_with("https://teddit.") =>
            {
                vid = reddit(&arg).await;
            }
            a if a.starts_with("https://twitter.com/")
                || a.starts_with("https://www.twitter.com/")
                || a.starts_with("https://nitter.") =>
            {
                vid = twatter(&arg).await
            }
            a if a.starts_with("https://odysee.com/")
                || a.starts_with("https://lbry.")
                || a.starts_with("https://librarian.") =>
            {
                vid = odysee(&arg).await
            }
            a if a.starts_with("https://www.bitchute.com/video/")
                || a.starts_with("https://bitchute.com/video/") =>
            {
                let arg = arg.replace("https://bitchute", "https://www.bitchute");
                vid = bitchute(&arg).await
            }
            a if a.starts_with("https://rumble.com/")
                || a.starts_with("https://www.rumble.com/") =>
            {
                vid = rumble(&arg).await
            }
            a if a.starts_with("https://streamvid.net") => {
                vid = streamvid(&arg, is_streaming_link).await
            }
            a if a.starts_with("https://streamtape.") => {
                vid = streamtape(&arg, is_streaming_link).await
            }
            a if a.starts_with("https://mp4upload.com/")
                || a.starts_with("https://www.mp4upload.com/") =>
            {
                vid = mp4upload(&arg).await
            }
            _ => {
                if arg.starts_with("https://") {
                    eprintln!("Unsupported link: {}", arg);
                } else {
                    eprintln!("Invalid arg: {}", arg);
                }
                help();
                exit(1);
            }
        }
    }

    match todo {
        "print link" => {
            if vid.audio_link.is_empty() {
                println!("{}", vid.vid_link);
            } else {
                println!("{}\n{}", vid.vid_link, vid.audio_link);
            }
        }
        "play" => {
            println!("Playing {}", vid.title);

            let mut audio_arg = String::new();

            if vid.vid_link.is_empty() {
                vid.vid_link = vid.audio_link;
            } else if !vid.audio_link.is_empty() {
                audio_arg = format!("--audio-file={}", vid.audio_link)
            }

            if env::consts::OS == "android" {
                Command::new("am")
                    .arg("start")
                    .args(["--user", "0"])
                    .args(["-a", "android.intent.action.VIEW"])
                    .args(["-d", &vid.vid_link])
                    .args(["-n", "is.xyz.mpv/.MPVActivity"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to execute am command");
            } else {
                Command::new("mpv")
                    .arg(vid.vid_link)
                    .arg(audio_arg)
                    .arg("--no-terminal")
                    .arg("--force-window=immediate")
                    .arg(format!("--force-media-title={}", vid.title))
                    .arg(format!("--user-agent={}", vid.user_agent))
                    .arg(format!("--referrer={}", vid.referrer))
                    .spawn()
                    .expect("Failed to execute mpv");
            }
        }
        "play audio" => {
            println!("Playing {}\n", vid.title);

            if vid.audio_link.is_empty() {
                eprintln!("No audio link found");
                exit(1);
            }

            if env::consts::OS == "android" {
                Command::new("am")
                    .arg("start")
                    .args(["--user", "0"])
                    .args(["-a", "android.intent.action.VIEW"])
                    .args(["-d", &vid.audio_link])
                    .args(["-n", "is.xyz.mpv/.MPVActivity"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to execute am command");
            } else if !Command::new("mpv")
                .arg(&vid.audio_link)
                .arg("--speed=1")
                .arg(format!("--force-media-title={}", vid.title))
                .arg(format!("--user-agent={}", vid.user_agent))
                .arg(format!("--referrer={}", vid.referrer))
                .status()
                .await
                .expect("Failed to execute mpv")
                .success()
            {
                eprintln!("Failed to play audio source: {}", vid.audio_link);
            }
        }
        "download" => {
            if !vid.audio_link.is_empty() {
                download(&vid, &vid.vid_link, " video", "mp4").await;

                download(&vid, &vid.audio_link, " audio", "mp3").await;

                let vid_title = format!("{} video.{}", vid.title, "mp4");
                let audio_title = format!("{} audio.{}", vid.title, "mp3");

                if Command::new("ffmpeg")
                    .args(["-i", &vid_title])
                    .args(["-i", &audio_title])
                    .args(["-c", "copy"])
                    .arg(format!("{}.mp4", vid.title))
                    .status()
                    .await
                    .expect("Failed to execute ffmpeg")
                    .success()
                {
                    println!("\nVideo & audio merged successfully");

                    fs::remove_file(vid_title)
                        .await
                        .unwrap_or_else(|_| eprintln!("Failed to remove downloaded video"));

                    fs::remove_file(audio_title)
                        .await
                        .unwrap_or_else(|_| eprintln!("Failed to remove downloaded audio"));
                } else {
                    eprintln!("\nVideo & audio merge failed");
                }
            } else {
                download(&vid, &vid.vid_link, "", "mp4").await;
            }
        }
        _ => println!("{:#?}", vid),
    }
}

fn help() {
    println!(
        "\nUsage: titan <argument> <url>\n
Arguments:                    
\t-h, --help\t\t Display this help message
\t-g, --get\t\t Get streaming link
\t-p, --play\t\t Play video in mpv
\t-d, --download\t\t Download video in aria2 
\t-D, --dl_link\t\t Get download link
\t-s, --stream_link\t Get streaming link
\t-r=, --resolution=720p\t Select resolution
\t-vc=, --video-codec=vp9\t Select video codec (default: avc)
\t-ac=, --audio-codec=mp4a Select audio codec (default: opus)
\t-c, --combined\t\t Combined video & audio        
\t-b, --best\t\t best resolution while playing (use it after -p flag)        

Supported Extractors : bitchute, doodstream, mp4upload, odysee, reddit, rumble, streamhub, streamtape, streamvid, substack, twatter, youtube"
    );
}

async fn download(vid: &Vid, link: &str, types: &str, extension: &str) {
    println!("\nDownloading{}: {}", types, vid.title);

    if Command::new("aria2c")
        .arg(link)
        .arg("--max-connection-per-server=16")
        .arg("--max-concurrent-downloads=16")
        .arg("--split=16")
        .arg("--min-split-size=1M")
        .arg("--check-certificate=false")
        .arg("--summary-interval=0")
        .arg("--download-result=hide")
        .arg(format!("--out={}{}.{}", vid.title, types, extension))
        .arg(format!("--user-agent={}", vid.user_agent))
        .arg(format!("--referer={}", vid.referrer))
        .status()
        .await
        .expect("Failed to execute aria2c")
        .success()
    {
        println!("\nDownloaded successfully");
    } else {
        eprintln!("\nDownload Failed");
    }
}
