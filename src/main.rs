mod extractors;
mod helpers;

use extractors::{
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
    vid_codec: Option<String>,
    resolution: Option<String>,
    audio_link: Option<String>,
    audio_codec: Option<String>,
}

impl Default for Vid {
    fn default() -> Self {
        Self {
            user_agent: String::from("uwu"),
            referrer: String::new(),
            title: String::new(),
            vid_link: String::new(),
            vid_codec: Some(String::from("avc")),
            resolution: None,
            audio_link: None,
            audio_codec: Some(String::from("opus")),
        }
    }
}
const RUMBLE_PREFIXES: [&str; 2] = ["https://rumble.com/", "https://www.rumble.com/"];
const MP4UPLOAD_PREFIXES: [&str; 2] = ["https://mp4upload.com/", "https://www.mp4upload.com/"];
const ODYSEE_PREFIXES: [&str; 3] = ["https://odysee.com/", "https://lbry.", "https://librarian."];

const BITCHUTE_PREFIXES: [&str; 2] = [
    "https://bitchute.com/video/",
    "https://www.bitchute.com/video/",
];

const YT_PREFIXES: [&str; 14] = [
    // YT
    "https://youtube.com/",
    "https://youtu.be/",
    "https://www.youtube.com/",
    "https://m.youtube.com/",
    // Piped
    "https://piped.",
    "https://watch.leptons.xyz/",
    "https://pi.ggtyler.dev",
    // Invidious instances generally start with invidious, inv, etc
    "https://invidious.",
    "https://inv.",
    "https://iv.",
    "https://yt.",
    "https://yewtu.be/",
    "vid.puffyan.us",
    "https://vid.priv.au/",
];

const REDDIT_PREFIXES: [&str; 6] = [
    "https://www.reddit.com/",
    "https://old.reddit.com/",
    "https://reddit.com/",
    "https://redd.it/",
    "https://libreddit.",
    "https://teddit.",
];

const TWATTER_PREFIXES: [&str; 6] = [
    "https://x.com/",
    "https://twitter.com/",
    "https://nitter.",
    "https://nt.",
    "https://mobile.x.com/",
    "https://mobile.twitter.com/",
];

const DOODSTREAM_PREDFIXES: [&str; 5] = [
    "https://doodstream.com/",
    "https://www.doodstream.com/",
    "https://dood.",
    "https://dooood.com/",
    "https://doods.pro/",
];
#[tokio::main]
async fn main() {
    let mut vid = Vid::default();
    let mut todo = "";
    let mut audio_only = false;
    let mut streaming_link = true;
    let mut is_dash = true;
    let mut resolution = String::from("best");
    let mut vid_codec = String::from("avc");
    let mut audio_codec = String::from("opus");
    let mut speed = String::new();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                help();
                exit(0);
            }
            "-v" | "--version" => {
                version();
                exit(0);
            }
            "-g" | "--get" => todo = "print link",
            "-p" | "--play" => {
                todo = "play";
                resolution = String::from("720");
                if !audio_only {
                    is_dash = false;
                }
            }
            arg if starts(&["-sp=", "--speed="], arg).await => {
                speed = format!("--speed={}", arg.split_once('=').unwrap().1);
                todo = "play";
                resolution = String::from("720");
                is_dash = false;
            }
            "-a" | "--audio-only" => audio_only = true,
            "-d" | "--download" => {
                todo = "download";
                streaming_link = false;
            }
            "-D" | "--dl_link" => streaming_link = false,
            "-s" | "--stream_link" => streaming_link = true,
            "-c" | "--combined" => {
                is_dash = false;
                resolution = String::from("720");
            }
            "-b" | "--best" => resolution = String::from("best"),
            arg if starts(&["-r=", "--resolution="], arg).await => {
                resolution = arg
                    .split_once('=')
                    .unwrap()
                    .1
                    .trim_end_matches('p')
                    .to_string();
            }
            arg if starts(&["-vc=", "--video-codec="], arg).await => {
                vid_codec = arg.split_once('=').unwrap().1.to_string();
            }
            arg if starts(&["-ac=", "--audio-codec="], arg).await => {
                audio_codec = arg.split_once('=').unwrap().1.to_string();
            }
            arg if starts(&DOODSTREAM_PREDFIXES, arg).await => {
                vid = doodstream(arg, streaming_link).await
            }
            arg if arg.contains(".substack.com/p/") => vid = substack(arg).await,
            arg if arg.starts_with("https://streamhub.") => {
                vid = streamhub(arg, streaming_link).await
            }
            arg if starts(&YT_PREFIXES, arg).await => {
                vid = youtube(arg, &resolution, &vid_codec, &audio_codec, is_dash).await
            }
            arg if starts(&REDDIT_PREFIXES, arg).await => vid = reddit(arg).await,
            arg if starts(&TWATTER_PREFIXES, arg).await => vid = twatter(arg).await,
            arg if starts(&ODYSEE_PREFIXES, arg).await => vid = odysee(arg).await,
            arg if starts(&BITCHUTE_PREFIXES, arg).await => vid = bitchute(arg).await,
            arg if starts(&RUMBLE_PREFIXES, arg).await => vid = rumble(arg).await,
            arg if arg.starts_with("https://streamvid.net") => {
                vid = streamvid(arg, streaming_link).await
            }
            arg if arg.starts_with("https://streamtape.") => {
                vid = streamtape(arg, streaming_link).await
            }
            arg if starts(&MP4UPLOAD_PREFIXES, arg).await => vid = mp4upload(arg).await,
            _ => {
                if arg.starts_with("https://") {
                    eprintln!("Unsupported link: {}\n", arg);
                } else {
                    eprintln!("Invalid arg: {}\n", arg);
                }
                help();
                exit(1);
            }
        }
    }

    match todo {
        "print link" => {
            if let Some(audio_link) = vid.audio_link {
                println!("{}\n{}", vid.vid_link, audio_link);
            } else {
                println!("{}", vid.vid_link);
            }
        }
        "play" => {
            println!("Playing {}", vid.title);

            let mut audio_arg = String::new();

            if audio_only && vid.audio_link.is_some() {
                vid.vid_link = vid.audio_link.unwrap();
            } else if vid.vid_link.is_empty() {
                vid.vid_link = vid.audio_link.expect("No vid or audio link found");
            } else if let Some(audio_link) = vid.audio_link {
                audio_arg = format!("--audio-file={}", audio_link)
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
            } else if !audio_only {
                Command::new("mpv")
                    .arg(vid.vid_link)
                    .arg(audio_arg)
                    .arg(speed)
                    .arg("--no-terminal")
                    .arg("--force-window=immediate")
                    .arg(format!("--force-media-title={}", vid.title))
                    .arg(format!("--user-agent={}", vid.user_agent))
                    .arg(format!("--referrer={}", vid.referrer))
                    .spawn()
                    .expect("Failed to execute mpv");
            } else if !Command::new("mpv")
                .arg(&vid.vid_link)
                .arg("--speed=1")
                .arg("--no-video")
                .arg(format!("--force-media-title={}", vid.title))
                .arg(format!("--user-agent={}", vid.user_agent))
                .arg(format!("--referrer={}", vid.referrer))
                .status()
                .await
                .expect("Failed to execute mpv")
                .success()
            {
                eprintln!("Failed to play audio: {}", vid.vid_link);
            }
        }
        "download" => {
            let Some(vid_codec) = &vid.vid_codec else {
                todo!()
            };

            let vid_ext = if vid_codec.starts_with("vp9") {
                "mkv"
            } else {
                "mp4"
            };

            if let Some(audio_link) = &vid.audio_link {
                let Some(audio_codec) = &vid.audio_codec else {
                    todo!()
                };

                let audio_ext = if audio_codec.starts_with("opus") {
                    "opus"
                } else {
                    "mp3"
                };

                if audio_only {
                    download(&vid, audio_link, " audio", audio_ext, false).await;
                } else {
                    download(&vid, &vid.vid_link, " video", vid_ext, true).await;

                    download(&vid, audio_link, " audio", audio_ext, true).await;

                    let vid_title = format!("{} video.{}", vid.title, vid_ext);
                    let audio_title = format!("{} audio.{}", vid.title, audio_ext);

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
                }
            } else {
                download(&vid, &vid.vid_link, "", vid_ext, false).await;
            }
        }
        _ => println!("{:#?}", vid),
    }
}

fn help() {
    version();

    println!(
        "\nUsage: titans <argument> <url>\n
Arguments:                    
\t-h, --help\t\t Display this help message
\t-v, --version\t\t Print version
\t-g, --get\t\t Get streaming link
\t-p, --play\t\t Play video in mpv
\t-sp=, --speed=\t\t Play video in mpv at --speed=1.5
\t-d, --download\t\t Download video in aria2 
\t-D, --dl_link\t\t Get download link
\t-s, --stream_link\t Get streaming link
\t-r=, --resolution=720p\t Select resolution
\t-vc=, --video-codec=vp9\t Select video codec (default: avc)
\t-ac=, --audio-codec=mp4a Select audio codec (default: opus)
\t-c, --combined\t\t Combined video & audio        
\t-b, --best\t\t best resolution while playing (use it after -p flag)        

Supported Extractors: bitchute, doodstream, mp4upload, odysee, reddit, rumble, streamhub, streamtape, streamvid, substack, twatter, youtube"
    );
}

fn version() {
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
}

async fn download(vid: &Vid, link: &str, mut types: &str, extension: &str, format_title: bool) {
    println!("\nDownloading{}: {}", types, vid.title);

    if !format_title {
        types = "";
    }

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

async fn starts(prefixes: &[&str], arg: &str) -> bool {
    prefixes.iter().any(|&prefix| arg.starts_with(prefix))
}
