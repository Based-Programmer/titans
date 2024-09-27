mod extractors;
mod helpers;

use extractors::{
    bitchute::bitchute, doodstream::doodstream, libsyn::libsyn, mp4upload::mp4upload,
    odysee::odysee, reddit::reddit, rokfin::rokfin, rumble::rumble, spotify::spotify,
    streamdav::streamdav, streamhub::streamhub, streamtape::streamtape, streamvid::streamvid,
    substack::substack, twatter::twatter, vtube::vtube, wolfstream::wolfstream, youtube::youtube,
};

use once_cell::sync::Lazy;
use regex::Regex;
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
    vid_codec: Option<Box<str>>,
    resolution: Option<u16>,
    audio_link: Option<Box<str>>,
    audio_codec: Option<Box<str>>,
    chapter_file: Option<Box<str>>,
}

impl Default for Vid {
    fn default() -> Self {
        Self {
            user_agent: "uwu",
            referrer: Box::from(""),
            title: Box::from(""),
            vid_link: Box::from(""),
            vid_codec: None,
            resolution: None,
            audio_link: None,
            audio_codec: None,
            chapter_file: None,
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
    /*
    let set_play = |todo: &mut Todo, audio_only: bool, is_dash: &mut bool| {
        *todo = Todo::Play;

        if !audio_only {
            *is_dash = false;
        }
    };
    */

    const VTUBE_PREFIXES: [&str; 2] = ["vtbe.to/", "vtube.network/"];
    const LIBSYN_PREFIXES: [&str; 2] = ["play.libsyn.com", "html5-player.libsyn.com"];
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

    const TWATTER_PREFIXES: [&str; 12] = [
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
        "xcancel.com/",
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
            "-V" | "--version" => {
                version();
                exit(0);
            }
            "-g" | "--get" => todo = Todo::GetLink,
            "-p" | "--play" => todo = Todo::Play,
            arg if starts(&["-sp=", "--speed="], arg) => {
                speed = arg.rsplit_once('=').unwrap().1.parse()?;
                todo = Todo::Play;
            }
            "-a" | "--audio-only" => audio_only = true,
            "-l" | "--loop" => loop_file = true,
            "-m" | "--music" => {
                audio_only = true;
                loop_file = true;
                speed = 1.0;
                todo = Todo::Play;
            }
            "-d" | "--download" => {
                todo = Todo::Download;
                streaming_link = false;
            }
            "-D" | "--dl_link" => streaming_link = false,
            "-s" | "--stream_link" => streaming_link = true,
            "-c" | "--combined" => is_dash = false,
            "-b" | "--best" => resolution = 0,
            arg if starts(&["-q=", "--quality="], arg) => {
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
                    } else if starts(&LIBSYN_PREFIXES, arg) {
                        vid = libsyn(arg)?;
                    } else if arg.starts_with("mp4upload.com/") {
                        vid = mp4upload(arg)?;
                    } else if arg.starts_with("rokfin.com/post/") {
                        vid = rokfin(arg, resolution)?;
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

            if OS == "android"
                && (!audio_only
                    || !Command::new("sh")
                        .args(["-c", "command -v mpv"])
                        .output()?
                        .status
                        .success())
            {
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

                let args = {
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

                    if let Some(chapters) = vid.chapter_file {
                        mpv_args.push(format!("--chapters-file={}", chapters));
                    }

                    mpv_args.into_boxed_slice()
                };

                let mpv_args = args.iter();

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
            let vid_ext = if some_codec_matches(&vid.vid_codec, "vp9", true) {
                "mkv"
            } else {
                "mp4"
            };

            let chapter = if vid.chapter_file.is_some() {
                " without chapters"
            } else {
                ""
            };

            let no_emoji = remove_emojis(&vid.title);

            if let Some(audio_link) = vid.audio_link.as_deref() {
                let audio_ext = if some_codec_matches(&vid.audio_codec, "opus", false) {
                    "opus"
                } else if some_codec_matches(&vid.audio_codec, "mp4a", true) {
                    "m4a"
                } else {
                    "mp3"
                };

                if audio_only {
                    download(
                        &vid, &no_emoji, audio_link, " audio", audio_ext, false, chapter,
                    );

                    if let Some(chapters) = vid.chapter_file {
                        let audio_title =
                            format!("{} audio{}.{}", no_emoji, chapter, audio_ext).into_boxed_str();

                        drop(no_emoji);

                        if Command::new("ffmpeg")
                            .args(["-i", &audio_title])
                            .args(["-i", &chapters])
                            .args(["-c", "copy"])
                            .args(["-y".to_owned(), format!("{}.{}", vid.title, audio_ext)])
                            .output()
                            .expect("Failed to execute ffmpeg")
                            .status
                            .success()
                        {
                            println!("{YELLOW}\nAudio & Chapters merged successfully{RESET}");
                            remove(&audio_title, "Failed to remove downloaded audio");
                        } else {
                            eprintln!("\n{RED}Audio & Chapters merge failed{RESET}");
                        }
                    }
                } else {
                    download(
                        &vid,
                        &no_emoji,
                        &vid.vid_link,
                        " video",
                        vid_ext,
                        true,
                        chapter,
                    );
                    download(
                        &vid, &no_emoji, audio_link, " audio", audio_ext, true, chapter,
                    );

                    let vid_title =
                        format!("{} video{}.{}", no_emoji, chapter, vid_ext).into_boxed_str();
                    let audio_title =
                        format!("{} audio{}.{}", no_emoji, chapter, audio_ext).into_boxed_str();

                    drop(no_emoji);

                    let mut chapter_name = "";

                    let ffmpeg_args = {
                        let mut args = Vec::new();

                        if let Some(chapters) = vid.chapter_file {
                            args.push("-i".to_owned());
                            args.push(chapters.into_string());

                            chapter_name = " + Chapters";
                        }

                        args.push("-c".to_owned());
                        args.push("copy".to_owned());

                        args.push("-y".to_owned());
                        args.push(format!("{}.{}", vid.title, vid_ext));

                        args.into_boxed_slice()
                    };

                    if Command::new("ffmpeg")
                        .args(["-i", &vid_title])
                        .args(["-i", &audio_title])
                        .args(ffmpeg_args.iter())
                        .status()
                        .expect("Failed to execute ffmpeg")
                        .success()
                    {
                        println!(
                            "{YELLOW}\nVideo + Audio{chapter_name} merged successfully{RESET}"
                        );

                        remove(&vid_title, "Failed to remove downloaded video");
                        remove(&audio_title, "Failed to remove downloaded audio");
                    } else {
                        eprintln!("\n{RED}Video + Audio{chapter_name} merge failed{RESET}");
                    }
                }
            } else {
                download(
                    &vid,
                    &no_emoji,
                    &vid.vid_link,
                    " video",
                    vid_ext,
                    false,
                    chapter,
                );

                if let Some(chapters) = vid.chapter_file {
                    let vid_title =
                        format!("{} video{}.{}", no_emoji, chapter, vid_ext).into_boxed_str();

                    drop(no_emoji);

                    if Command::new("ffmpeg")
                        .args(["-i", &vid_title])
                        .args(["-i", &chapters])
                        .args(["-c", "copy"])
                        .args(["-y".to_owned(), format!("{}.{}", vid.title, vid_ext)])
                        .output()
                        .expect("Failed to execute ffmpeg")
                        .status
                        .success()
                    {
                        println!("{YELLOW}\nVideo & Chapters merged successfully{RESET}");
                        remove(&vid_title, "Failed to remove downloaded video");
                    } else {
                        eprintln!("\n{RED}Video & Chapters merge failed{RESET}");
                    }
                }
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
\t-V, --version\t\t Print version
\t-g, --get\t\t Get streaming link
\t-p, --play\t\t Play video in mpv
\t-sp=, --speed=\t\t Play video in mpv at --speed=1.5
\t-a, --audio-only\t Play or Download only the audio
\t-l, --loop\t\t Loop file while playing
\t-m, --music\t\t Play music (loop audio at speed 1)
\t-d, --download\t\t Download video with aria2
\t-D, --dl_link\t\t Get download link
\t-s, --stream_link\t Get streaming link
\t-q=, --quality=720p\t Select resolution
\t-vc=, --video-codec=vp9\t Select video codec (default: avc)
\t-ac=, --audio-codec=mp4a Select audio codec (default: opus)
\t-c, --combined\t\t Combined video & audio
\t-b, --best\t\t best resolution while playing (use it after -p flag)

Supported Extractors: bitchute, doodstream, libsyn, mp4upload, odysee, reddit, rokfin, rumble, spotify, streamdav, streamhub, streamtape, streamvid, substack, twatter, vtube, wolfstream, youtube");
}

fn version() {
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
}

fn download(
    vid: &Vid,
    vid_title: &str,
    link: &str,
    mut types: &str,
    extension: &str,
    format_title: bool,
    chapter: &str,
) {
    println!(
        "\n{}Downloading{}:{} {}.{}",
        YELLOW, types, RESET, vid.title, extension
    );

    if !format_title && chapter.is_empty() {
        types = "";
    }

    let title_arg = {
        let title = if vid_title.len() > 201 {
            let title = &vid_title[..201];
            title.rsplit_once(' ').unwrap_or((title, "")).0
        } else {
            vid_title
        }
        .replace('\n', " ")
        .replace('/', "|")
        .trim_end_matches('.')
        .to_owned();

        let no_multi_space = remove_multiple_spaces(&title);

        format!("--out={}{}{}.{}", no_multi_space, types, chapter, extension).into_boxed_str()
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
        println!("\n{YELLOW}Downloaded{types} successfully{RESET}");
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

fn remove_emojis(string: &str) -> Box<str> {
    static RE_NO_EMOJI: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"[\p{Emoji_Presentation}\p{Emoji_Modifier_Base}\p{Emoji_Modifier}]").unwrap()
    });
    let no_emojis = RE_NO_EMOJI.replace_all(string, "");

    remove_multiple_spaces(&no_emojis)
}

fn remove_multiple_spaces(string: &str) -> Box<str> {
    let title_vec: Vec<&str> = string.split_whitespace().collect();
    {
        if !title_vec.is_empty() {
            title_vec.join(" ").into()
        } else {
            string.into()
        }
    }
}
fn some_codec_matches(codec: &Option<Box<str>>, matches: &str, starts: bool) -> bool {
    if let Some(codec) = codec {
        if starts {
            codec.starts_with(matches)
        } else {
            **codec == *matches
        }
    } else {
        false
    }
}
