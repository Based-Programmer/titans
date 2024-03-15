mod extractors;
mod helpers;

use extractors::{
    bitchute::bitchute, doodstream::doodstream, libsyn::libsyn, mp4upload::mp4upload,
    odysee::odysee, reddit::reddit, rokfin::rokfin, rumble::rumble, spotify::spotify,
    streamdav::streamdav, streamhub::streamhub, streamtape::streamtape, streamvid::streamvid,
    substack::substack, twatter::twatter, vtube::vtube, wolfstream::wolfstream, youtube::youtube,
};

use std::{
    env::{args, consts::OS},
    error::Error,
    fs::remove_file,
    process::{exit, Command, Stdio},
};

#[derive(Debug, PartialEq)]
pub struct Vid {
    user_agent: &'static str,
    referrer: Box<str>,
    title: Box<str>,
    vid_link: Box<str>,
    vid_codec: Box<str>,
    resolution: Option<u16>,
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

#[derive(Clone, Copy, PartialEq)]
enum Todo {
    Play,
    Download,
    GetLink,
    Debug,
}

pub const RED: &str = "\u{1b}[31m";
pub const RESET: &str = "\u{1b}[0m";
pub const YELLOW: &str = "\u{1b}[33m";

fn main() -> Result<(), Box<dyn Error>> {
    let mut vid = Vid::default();
    let mut todo = Todo::Debug;
    let mut audio_only = false;
    let mut loop_file = false;
    let mut streaming_link = true;
    let mut is_dash = true;
    let mut no_args = true;
    let mut multiple_links = false;
    let mut resolution: u16 = 0;
    let mut vid_codec = String::from("avc");
    let mut audio_codec = String::from("opus");
    let mut speed: f32 = 0.0;
    let set_play = |todo: &mut Todo, audio_only: bool, is_dash: &mut bool| {
        *todo = Todo::Play;

        if !audio_only {
            *is_dash = false;
        }
    };

    const VTUBE_PREFIXES: [&str; 2] = ["vtbe.to/", "vtube.network/"];
    const SPOTIFY_PREFIXES: [&str; 2] = [
        "open.spotify.com/episode/",
        "open.spotify.com/embed/episode/",
    ];

    const ODYSEE_PREFIXES: [&str; 4] = [
        "odysee.com/",
        // Librarian
        "lbry.",
        "librarian.",
        "odysee.076.ne.jp/",
    ];

    const YT_PREFIXES: [&str; 17] = [
        "youtu.be/",
        // Hyperpipe
        "hyperpipe.",
        "music.",
        "listen.",
        "hp.",
        // Piped
        "piped.",
        "watch.leptons.xyz/",
        "pi.ggtyler.dev",
        // Invidious instances generally start with invidious, inv, etc
        "invidious.",
        "inv.",
        "iv.",
        "yt.",
        "yewtu.be/",
        "vid.puffyan.us/",
        "vid.priv.au/",
        "onion.tube/",
        "anontube.lvkaszus.pl/",
    ];

    const REDDIT_PREFIXES: [&str; 16] = [
        // Reddit
        "old.reddit.com/",
        "redd.it/",
        "reddit.", // bcz some libreddit & teddit instances start with reddit.
        // Libreddit
        "libreddit.",
        "lr.",
        "safereddit.com/",
        "r.walkx.fyi/",
        "l.opnxng.com/",
        "snoo.habedieeh.re/",
        // Teddit
        "teddit.",
        "snoo.ioens.is/",
        "incogsnoo.com/",
        "rdt.trom.tf/",
        "i.opnxng.com/",
        "td.vern.cc/",
        "t.sneed.network/",
    ];

    const TWATTER_PREFIXES: [&str; 11] = [
        "x.com/",
        "mobile.x.com/",
        "twitter.com/",
        "mobile.twitter.com/",
        // Nitter
        "nitter.",
        "nt.",
        "n.",
        "twiiit.com/",
        "tweet.lambda.dance/",
        "bird.habedieeh.re/",
        "t.com.sb/",
    ];

    const DOODSTREAM_PREFIXES: [&str; 7] = [
        "doodstream.com/",
        "d0o0d.com/",
        "d0000d.com/",
        "ds2play.com/",
        "dooood.com/",
        "doods.pro/",
        "dood.",
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
                set_play(&mut todo, audio_only, &mut is_dash);
            }
            arg if starts(&["-sp=", "--speed="], arg) => {
                speed = arg.rsplit_once('=').unwrap().1.parse()?;
                set_play(&mut todo, audio_only, &mut is_dash);
            }
            "-a" | "--audio-only" => audio_only = true,
            "-l" | "--loop" => loop_file = true,
            "-m" | "--music" => {
                audio_only = true;
                loop_file = true;
                speed = 1.0;
                set_play(&mut todo, audio_only, &mut is_dash);
            }
            "-d" | "--download" => {
                todo = Todo::Download;
                streaming_link = false;
            }
            "-D" | "--dl_link" => streaming_link = false,
            "-s" | "--stream_link" => streaming_link = true,
            "-c" | "--combined" => {
                is_dash = false;
                resolution = 720;
            }
            "-b" | "--best" => resolution = 0,
            arg if starts(&["-r=", "--resolution="], arg) => {
                resolution = arg
                    .split_once('=')
                    .unwrap()
                    .1
                    .trim_end_matches('p')
                    .parse()?;
            }
            arg if starts(&["-vc=", "--video-codec="], arg) => {
                vid_codec = arg.split_once('=').unwrap().1.to_string();
            }
            arg if starts(&["-ac=", "--audio-codec="], arg) => {
                audio_codec = arg.split_once('=').unwrap().1.to_string();
            }
            mut arg if starts(&["https://", "http://"], arg) => {
                if vid == Vid::default() {
                    arg = arg
                        .trim_start_matches("https://")
                        .trim_start_matches("http://")
                        .trim_start_matches("www.");

                    if arg.contains(".substack.com/p/") {
                        vid = substack(arg)?;
                    } else if arg.starts_with("streamhub.") {
                        vid = streamhub(arg, streaming_link)?;
                    } else if arg.starts_with("streamvid.") {
                        vid = streamvid(arg, streaming_link)?;
                    } else if arg.starts_with("streamtape.") {
                        vid = streamtape(arg, streaming_link)?;
                    } else if arg.starts_with("streamdav.com/") {
                        vid = streamdav(arg)?;
                    } else if arg.starts_with("wolfstream.tv/") {
                        vid = wolfstream(arg)?;
                    } else if starts(&SPOTIFY_PREFIXES, arg) {
                        vid = spotify(arg)?;
                    } else if arg.starts_with("bitchute.com/") {
                        vid = bitchute(arg)?;
                    } else if arg.starts_with("rumble.com/") {
                        vid = rumble(arg, resolution)?;
                    } else if starts(&ODYSEE_PREFIXES, arg) {
                        vid = odysee(arg)?;
                    } else if arg.contains("youtube.com/") || starts(&YT_PREFIXES, arg) {
                        if todo == Todo::Play && resolution == 0 {
                            resolution = 720;
                        }
                        vid = youtube(arg, resolution, &vid_codec, &audio_codec, is_dash)?;
                    } else if starts(&REDDIT_PREFIXES, arg) {
                        vid = reddit(arg)?;
                    } else if starts(&TWATTER_PREFIXES, arg) || arg.contains("unofficialbird.com/")
                    {
                        vid = twatter(arg, resolution, streaming_link)?;
                    } else if starts(&DOODSTREAM_PREFIXES, arg) {
                        vid = doodstream(arg, streaming_link)?;
                    } else if starts(&VTUBE_PREFIXES, arg) {
                        vid = vtube(arg, streaming_link)?;
                    } else if arg.starts_with("mp4upload.com/") {
                        vid = mp4upload(arg)?;
                    } else if arg.starts_with("rokfin.com/post/") {
                        vid = rokfin(arg, resolution)?;
                    } else if arg.starts_with("html5-player.libsyn.com/") {
                        vid = libsyn(arg)?;
                    } else {
                        eprintln!("{RED}Unsupported link:{YELLOW} https://{arg}{RESET}\n");
                        exit(1);
                    }
                } else if !multiple_links {
                    eprintln!("{RED}Multiple links are not allowed as of now{RESET}\n");
                    multiple_links = true;
                }
            }
            _ => {
                eprintln!("{RED}Invalid arg:{YELLOW} {arg}{RESET}\n");
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

    if vid.vid_link.is_empty() && vid.audio_link.is_none() {
        eprintln!("{RED}No video or audio link found{RESET}");
        exit(1);
    }

    match todo {
        Todo::Debug => println!("{:#?}", vid),
        Todo::GetLink => {
            if let Some(audio_link) = vid.audio_link {
                if !audio_only {
                    println!("{}\n{}", vid.vid_link, audio_link);
                } else {
                    println!("{}", audio_link);
                }
            } else {
                println!("{}", vid.vid_link);
            }
        }
        Todo::Play => {
            println!("{}Playing {}{}", YELLOW, vid.title, RESET);

            let mut audio_arg = String::new();

            if (audio_only && vid.audio_link.is_some()) || vid.vid_link.is_empty() {
                vid.vid_link = vid.audio_link.unwrap();
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

                if speed != 0.0 {
                    mpv_args.push(format!("--speed={}", speed));
                }

                if loop_file {
                    mpv_args.push(String::from("--loop-file"));
                }

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
                    download(&vid, audio_link, " audio", audio_ext, false);
                } else {
                    download(&vid, &vid.vid_link, " video", vid_ext, true);

                    download(&vid, audio_link, " audio", audio_ext, true);

                    let vid_title = format!("{} video.{}", vid.title, vid_ext).into_boxed_str();
                    let audio_title = format!("{} audio.{}", vid.title, audio_ext).into_boxed_str();

                    if Command::new("ffmpeg")
                        .args(["-i", &vid_title])
                        .args(["-i", &audio_title])
                        .args(["-c", "copy"])
                        .arg(format!("{}.mp4", vid.title))
                        .status()
                        .expect("Failed to execute ffmpeg")
                        .success()
                    {
                        println!("\nVideo & audio merged successfully");

                        remove(&vid_title, "Failed to remove downloaded video");
                        remove(&audio_title, "Failed to remove downloaded audio");
                    } else {
                        eprintln!("\n{RED}Video & audio merge failed{RESET}");
                    }
                }
            } else {
                download(&vid, &vid.vid_link, "", vid_ext, false);
            }
        }
    }

    Ok(())
}

fn help() {
    version();

    println!("
Usage: titans <args> <url>

Arguments:
\t-h, --help\t\t Display this help message
\t-v, --version\t\t Print version
\t-g, --get\t\t Get streaming link
\t-p, --play\t\t Play video in mpv
\t-sp=, --speed=\t\t Play video in mpv at --speed=1.5
\t-a, --audio-only\t Play or Download only the audio
\t-l, --loop\t\t Loop file while playing
\t-m, --music\t\t Play music (loop audio at speed 1)
\t-d, --download\t\t Download video with aria2
\t-D, --dl_link\t\t Get download link
\t-s, --stream_link\t Get streaming link
\t-r=, --resolution=720p\t Select resolution
\t-vc=, --video-codec=vp9\t Select video codec (default: avc)
\t-ac=, --audio-codec=mp4a Select audio codec (default: opus)
\t-c, --combined\t\t Combined video & audio
\t-b, --best\t\t best resolution while playing (use it after -p flag)

Supported Extractors: bitchute, doodstream, libsyn, mp4upload, odysee, reddit, rokfin, rumble, spotify, streamdav, streamhub, streamtape, streamvid, substack, twatter, vtube, wolfstream, youtube");
}

fn version() {
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
}

fn download(vid: &Vid, link: &str, mut types: &str, extension: &str, format_title: bool) {
    println!("\nDownloading{}: {}", types, vid.title);

    if !format_title {
        types = "";
    }

    let title_arg = {
        let mut title = if vid.title.len() > 113 {
            let title = &vid.title[..113];
            title.rsplit_once(' ').unwrap_or((title, "")).0
        } else {
            &vid.title
        }
        .replace('\n', " ")
        .trim_end_matches('.')
        .to_owned();

        let title_vec: Vec<&str> = title.split_whitespace().collect(); // multi-space removed
        if !title_vec.is_empty() {
            title = title_vec.join(" ");
        }

        format!("--out={}{}.{}", title, types, extension).into_boxed_str()
    };

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
            &title_arg,
        ])
        .args(["--user-agent", vid.user_agent])
        .args(["--referer", &vid.referrer])
        .status()
        .expect("Failed to execute aria2")
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

fn remove(path: &str, msg: &str) {
    remove_file(path).unwrap_or_else(|_| eprintln!("{RED}{msg}{RESET}"));
}
