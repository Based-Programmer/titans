mod extractors;
mod helpers;

use extractors::{
    bitchute::bitchute, doodstream::doodstream, mp4upload::mp4upload, odysee::odysee,
    reddit::reddit, rokfin::rokfin, rumble::rumble, streamdav::streamdav, streamhub::streamhub,
    streamtape::streamtape, streamvid::streamvid, substack::substack, twatter::twatter,
    vtube::vtube, wolfstream::wolfstream, youtube::youtube,
};

use std::{
    env::{args, consts::OS},
    error::Error,
    process::{exit, Stdio},
};

use tokio::{fs::remove_file, process::Command};

#[derive(Debug)]
pub struct Vid {
    user_agent: &'static str,
    referrer: Box<str>,
    title: Box<str>,
    vid_link: Box<str>,
    vid_codec: Box<str>,
    resolution: Option<Box<str>>,
    audio_link: Option<Box<str>>,
    audio_codec: Box<str>,
}

impl Default for Vid {
    fn default() -> Self {
        Self {
            user_agent: "uwu",
            referrer: Box::from(""),
            title: Box::from(""),
            vid_link: Box::from(""),
            vid_codec: Box::from(""),
            resolution: None,
            audio_link: None,
            audio_codec: Box::from(""),
        }
    }
}

#[derive(Clone, Copy)]
enum Todo {
    Play,
    Download,
    GetLink,
    Debug,
}

const RED: &str = "\u{1b}[31m";
const RESET: &str = "\u{1b}[0m";
const YELLOW: &str = "\u{1b}[33m";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut vid = Vid::default();
    let mut todo = Todo::Debug;
    let mut audio_only = false;
    let mut streaming_link = true;
    let mut is_dash = true;
    let mut no_args = true;
    let mut resolution = String::from("best");
    let mut vid_codec = String::from("avc");
    let mut audio_codec = String::from("opus");
    let mut speed: Box<str> = Box::from("");
    let set_play =
        |todo: &mut Todo, resolution: &mut String, audio_only: bool, is_dash: &mut bool| {
            *todo = Todo::Play;
            *resolution = String::from("720");

            if !audio_only {
                *is_dash = false;
            }
        };

    const RUMBLE_PREFIXES: [&str; 2] = ["https://rumble.com/", "https://www.rumble.com/"];
    const MP4UPLOAD_PREFIXES: [&str; 2] = ["https://mp4upload.com/", "https://www.mp4upload.com/"];
    const VTUBE_PREFIXES: [&str; 2] = ["https://vtbe.to/", "https://vtube.network/"];

    const BITCHUTE_PREFIXES: [&str; 2] = [
        "https://bitchute.com/video/",
        "https://www.bitchute.com/video/",
    ];

    const ODYSEE_PREFIXES: [&str; 4] = [
        "https://odysee.com/",
        // Librarian
        "https://lbry.",
        "https://librarian.",
        "https://odysee.076.ne.jp/",
    ];

    const YT_PREFIXES: [&str; 13] = [
        "https://youtu.be/",
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
        "https://vid.puffyan.us/",
        "https://vid.priv.au/",
        "https://onion.tube/",
        "https://anontube.lvkaszus.pl/",
    ];

    const REDDIT_PREFIXES: [&str; 17] = [
        // Reddit
        "https://www.reddit.com/",
        "https://old.reddit.com/",
        "https://redd.it/",
        "https://reddit.", // bcz some libreddit & teddit instances start with reddit.
        // Libreddit
        "https://libreddit.",
        "https:://lr.",
        "https://safereddit.com/",
        "https://r.walkx.fyi/",
        "https://l.opnxng.com/",
        "https://snoo.habedieeh.re/",
        // Teddit
        "https://teddit.",
        "https://snoo.ioens.is/",
        "https://incogsnoo.com/",
        "https://rdt.trom.tf/",
        "https://i.opnxng.com/",
        "https://td.vern.cc/",
        "https://t.sneed.network/",
    ];

    const TWATTER_PREFIXES: [&str; 13] = [
        "https://x.com/",
        "https://www.x.com/",
        "https://mobile.x.com/",
        "https://twitter.com/",
        "https://www.twitter.com/",
        "https://mobile.twitter.com/",
        // Nitter
        "https://nitter.",
        "https://nt.",
        "https://n.",
        "https://twiiit.com/",
        "https://tweet.lambda.dance/",
        "https://bird.habedieeh.re/",
        "https://t.com.sb/",
    ];

    const DOODSTREAM_PREFIXES: [&str; 6] = [
        "https://doodstream.com/",
        "https://www.doodstream.com/",
        "https://ds2play.com/",
        "https://dooood.com/",
        "https://doods.pro/",
        "https://dood.",
    ];

    for arg in args().skip(1) {
        no_args = false;

        match arg.as_str() {
            "-h" | "--help" => {
                help_exit(0);
            }
            "-v" | "--version" => {
                version();
                exit(0);
            }
            "-g" | "--get" => todo = Todo::GetLink,
            "-p" | "--play" => {
                set_play(&mut todo, &mut resolution, audio_only, &mut is_dash);
            }
            arg if starts(&["-sp=", "--speed="], arg) => {
                speed = format!("--speed={}", arg.rsplit_once('=').unwrap().1).into();
                set_play(&mut todo, &mut resolution, audio_only, &mut is_dash);
            }
            "-a" | "--audio-only" => audio_only = true,
            "-d" | "--download" => {
                todo = Todo::Download;
                streaming_link = false;
            }
            "-D" | "--dl_link" => streaming_link = false,
            "-s" | "--stream_link" => streaming_link = true,
            "-c" | "--combined" => {
                is_dash = false;
                resolution = String::from("720");
            }
            "-b" | "--best" => resolution = String::from("best"),
            arg if starts(&["-r=", "--resolution="], arg) => {
                resolution = arg
                    .split_once('=')
                    .unwrap()
                    .1
                    .trim_end_matches('p')
                    .to_string();
            }
            arg if starts(&["-vc=", "--video-codec="], arg) => {
                vid_codec = arg.split_once('=').unwrap().1.to_string();
            }
            arg if starts(&["-ac=", "--audio-codec="], arg) => {
                audio_codec = arg.split_once('=').unwrap().1.to_string();
            }
            arg if arg.contains(".substack.com/p/") => vid = substack(arg).await?,
            arg if arg.starts_with("https://streamhub.") => {
                vid = streamhub(arg, streaming_link).await?
            }
            arg if arg.starts_with("https://streamvid.") => {
                vid = streamvid(arg, streaming_link).await?
            }
            arg if arg.starts_with("https://streamtape.") => {
                vid = streamtape(arg, streaming_link).await?
            }
            arg if arg.starts_with("https://streamdav.com/") => vid = streamdav(arg).await?,
            arg if arg.starts_with("https://wolfstream.tv/") => vid = wolfstream(arg).await?,
            arg if starts(&BITCHUTE_PREFIXES, arg) => vid = bitchute(arg).await?,
            arg if starts(&RUMBLE_PREFIXES, arg) => vid = rumble(arg).await?,
            arg if starts(&ODYSEE_PREFIXES, arg) => vid = odysee(arg).await?,
            arg if arg.contains("youtube.com/") || starts(&YT_PREFIXES, arg) => {
                vid = youtube(arg, &resolution, &vid_codec, &audio_codec, is_dash).await?
            }
            arg if starts(&REDDIT_PREFIXES, arg) => vid = reddit(arg).await?,
            arg if arg.contains("unofficialbird.com/") || starts(&TWATTER_PREFIXES, arg) => {
                vid = twatter(arg, &resolution, streaming_link).await?
            }
            arg if starts(&DOODSTREAM_PREFIXES, arg) => {
                vid = doodstream(arg, streaming_link).await?
            }
            arg if starts(&VTUBE_PREFIXES, arg) => vid = vtube(arg, streaming_link).await?,
            arg if starts(&MP4UPLOAD_PREFIXES, arg) => vid = mp4upload(arg).await?,
            arg if arg.starts_with("https://rokfin.com/post/") => {
                vid = rokfin(arg, &resolution).await?
            }
            _ => {
                if arg.starts_with("https://") {
                    eprintln!("{RED}Unsupported link:{YELLOW} {arg}{RESET}\n");
                } else {
                    eprintln!("{RED}Invalid arg:{YELLOW} {arg}{RESET}\n");
                }
                help_exit(1);
            }
        }
    }

    if no_args {
        eprintln!("{RED}No args provided{RESET}\n");
        help_exit(1);
    }

    if vid.vid_codec.is_empty() {
        vid.vid_codec = Box::from("avc");
    }

    if vid.audio_codec.is_empty() {
        vid.audio_codec = Box::from("aac");
    }

    match todo {
        Todo::Debug => println!("{:#?}", vid),
        Todo::GetLink => {
            if let Some(audio_link) = vid.audio_link {
                println!("{}\n{}", vid.vid_link, audio_link);
            } else {
                println!("{}", vid.vid_link);
            }
        }
        Todo::Play => {
            println!("Playing {}", vid.title);

            let mut audio_arg = String::new();

            if audio_only && vid.audio_link.is_some() {
                vid.vid_link = vid.audio_link.unwrap();
            } else if vid.vid_link.is_empty() {
                vid.vid_link = vid.audio_link.expect("No vid or audio link found");
            } else if let Some(audio_link) = vid.audio_link {
                audio_arg = format!("--audio-file={}", audio_link)
            }

            if OS == "android" {
                let am_mpv_args = [
                    "start",
                    "--user",
                    "0",
                    "-a",
                    "android.intent.action.VIEW",
                    "-d",
                    &vid.vid_link,
                    "-n",
                    "is.xyz.mpv/.MPVActivity",
                ];

                Command::new("am")
                    .args(am_mpv_args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to execute am command");
            } else {
                let mpv = {
                    if OS == "windows" {
                        "mpv.exe"
                    } else {
                        "mpv"
                    }
                };

                let mut mpv_args = vec![
                    vid.vid_link.to_string(),
                    format!("--force-media-title={}", vid.title),
                    format!("--user-agent={}", vid.user_agent),
                    format!("--referrer={}", vid.referrer),
                ];

                if !speed.is_empty() {
                    mpv_args.push(speed.to_string());
                }

                drop(speed);

                if !audio_arg.is_empty() {
                    mpv_args.push(audio_arg);
                }

                if !audio_only {
                    Command::new(mpv)
                        .args(mpv_args)
                        .args(["--no-terminal", "--force-window=immediate"])
                        .spawn()
                        .expect("Failed to execute mpv");
                } else if !Command::new(mpv)
                    .args(mpv_args)
                    .arg("--no-video")
                    .status()
                    .await
                    .expect("Failed to execute mpv")
                    .success()
                {
                    eprintln!("{RED}Failed to play audio:{YELLOW} {}{RESET}", vid.vid_link);
                }
            }
        }
        Todo::Download => {
            let vid_ext = if vid.vid_codec.starts_with("vp9") {
                "mkv"
            } else {
                "mp4"
            };

            if let Some(audio_link) = vid.audio_link.as_deref() {
                let audio_ext = if &*vid.audio_codec == "opus" {
                    "opus"
                } else if vid.audio_codec.starts_with("mp4a") {
                    "mp4a"
                } else {
                    "mp3"
                };

                if audio_only {
                    download(&vid, audio_link, " audio", audio_ext, false).await;
                } else {
                    download(&vid, &vid.vid_link, " video", vid_ext, true).await;

                    download(&vid, audio_link, " audio", audio_ext, true).await;

                    let vid_title = format!("{} video.{}", vid.title, vid_ext).into_boxed_str();
                    let audio_title = format!("{} audio.{}", vid.title, audio_ext).into_boxed_str();

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

                        remove(&vid_title, "Failed to remove downloaded video").await;
                        remove(&audio_title, "Failed to remove downloaded audio").await;
                    } else {
                        eprintln!("\n{RED}Video & audio merge failed{RESET}");
                    }
                }
            } else {
                download(&vid, &vid.vid_link, "", vid_ext, false).await;
            }
        }
    }

    Ok(())
}

fn help() {
    version();

    println!("
Usage: titans <argument> <url>

Arguments:
\t-h, --help\t\t Display this help message
\t-v, --version\t\t Print version
\t-g, --get\t\t Get streaming link
\t-p, --play\t\t Play video in mpv
\t-sp=, --speed=\t\t Play video in mpv at --speed=1.5
\t-a, --audio-only\t Play or Download only the audio
\t-d, --download\t\t Download video with aria2
\t-D, --dl_link\t\t Get download link
\t-s, --stream_link\t Get streaming link
\t-r=, --resolution=720p\t Select resolution
\t-vc=, --video-codec=vp9\t Select video codec (default: avc)
\t-ac=, --audio-codec=mp4a Select audio codec (default: opus)
\t-c, --combined\t\t Combined video & audio
\t-b, --best\t\t best resolution while playing (use it after -p flag)

Supported Extractors: bitchute, doodstream, mp4upload, odysee, reddit, rokfin, rumble, streamdav, streamhub, streamtape, streamvid, substack, twatter, vtube, wolfstream, youtube");
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
        .args([
            link,
            "--max-connection-per-server=16",
            "--max-concurrent-downloads=16",
            "--split=16",
            "--min-split-size=1M",
            "--check-certificate=false",
            "--summary-interval=0",
            "--download-result=hide",
            &format!("--out={}{}.{}", vid.title, types, extension),
        ])
        .args(["--user-agent", vid.user_agent])
        .args(["--referer", &vid.referrer])
        .status()
        .await
        .expect("Failed to execute aria2c")
        .success()
    {
        println!("\nDownloaded successfully");
    } else {
        eprintln!("\n{RED}Download Failed{RESET}");
    }
}

fn starts(prefixes: &[&str], arg: &str) -> bool {
    prefixes.iter().any(|&prefix| arg.starts_with(prefix))
}

fn help_exit(exit_code: i32) {
    help();
    exit(exit_code);
}

async fn remove(path: &str, msg: &str) {
    remove_file(path)
        .await
        .unwrap_or_else(|_| eprintln!("{RED}{msg}{RESET}"));
}
