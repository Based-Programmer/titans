mod extractors;
mod helpers;

use crate::extractors::{
    doodstream::doodstream, mp4upload::mp4upload, odysee::odysee, reddit::reddit, rumble::rumble,
    streamhub::streamhub, streamtape::streamtape, streamvid::streamvid, substack::substack,
    twatter::twatter, youtube::youtube,
};

use std::{
    env, fs,
    process::{exit, Command, Stdio},
};

#[derive(Debug)]
pub struct Vid {
    title: String,
    user_agent: String,
    vid_link: String,
    audio_link: String,
    referrer: String,
}

impl Default for Vid {
    fn default() -> Self {
        Self {
            title: String::new(),
            user_agent: String::from("uwu"),
            vid_link: String::new(),
            audio_link: String::new(),
            referrer: String::new(),
        }
    }
}

/*impl std::fmt::Display for Vid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "title: {}\nuser_agent: {}\nlink: {}\nreferrer: {}",
            self.title, self.user_agent, self.link, self.referrer
        )
    }
}*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vid = Vid::default();
    let mut best_video = String::new();
    let mut best_audio = "";

    let mut todo = "";
    let mut is_streaming_link = true;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                help();
                exit(0);
            }
            "-g" | "--get" => todo = "print link",
            "-p" | "--play" => todo = "play",
            "-d" | "--download" => {
                todo = "download";
                is_streaming_link = false;
            }
            "-D" | "--dl_link" => is_streaming_link = false,
            "-s" | "--stream_link" => is_streaming_link = true,
            a if a.starts_with("https://doodstream.com/")
                || a.starts_with("https://www.doodstream.com/")
                || a.starts_with("https://dood.")
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
                vid = youtube(&arg).await
            }
            a if a.starts_with("https://www.reddit.com/")
                || a.starts_with("https://old.reddit.com/")
                || a.starts_with("https://reddit.com/")
                || a.starts_with("https://redd.it/")
                || a.starts_with("https://libreddit.")
                || a.starts_with("https://teddit.") =>
            {
                (vid, best_video) = reddit(&arg).await;
                best_audio = "DASH_audio.mp4";
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

    if vid.vid_link.is_empty() {
        println!("Failed to find the video link");
        exit(1);
    }

    match todo {
        "print link" => println!("{}", vid.vid_link),
        "play" => {
            println!("Playing {}", vid.title);

            let audio_arg = if vid.audio_link.is_empty() {
                String::new()
            } else {
                format!("--audio-file={}", vid.audio_link)
            };

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
        "download" => {
            println!("Downloading {}", vid.title);

            if vid.audio_link.is_empty() {
                if Command::new("aria2c")
                    .arg(vid.vid_link)
                    .arg("--max-connection-per-server=16")
                    .arg("--max-concurrent-downloads=16")
                    .arg("--split=16")
                    .arg("--min-split-size=1M")
                    .arg("--check-certificate=false")
                    .arg("--summary-interval=0")
                    .arg("--download-result=hide")
                    .arg(format!("--out={}.mp4", vid.title))
                    .arg(format!("--user-agent={}", vid.user_agent))
                    .arg(format!("--referer={}", vid.referrer))
                    .status()
                    .expect("Failed to execute aria2c")
                    .success()
                {
                    println!("Download Completed: {}", vid.title);
                } else {
                    println!("Download Failed: {}", vid.title);
                }
            } else {
                if Command::new("aria2c")
                    .arg(vid.vid_link)
                    .arg(vid.audio_link)
                    .arg("--force-sequential")
                    .arg("--max-connection-per-server=16")
                    .arg("--max-concurrent-downloads=16")
                    .arg("--split=16")
                    .arg("--min-split-size=1M")
                    .arg("--check-certificate=false")
                    .arg("--summary-interval=0")
                    .arg("--download-result=hide")
                    .arg(format!("--user-agent={}", vid.user_agent))
                    .arg(format!("--referer={}", vid.referrer))
                    .status()
                    .expect("Failed to execute aria2c")
                    .success()
                {
                    println!(
                        "\nDownload Completed: {}\n
Merging video & audio files\n",
                        vid.title
                    );

                    if Command::new("ffmpeg")
                        .args(["-i", &best_video])
                        .args(["-i", best_audio])
                        .args(["-c", "copy"])
                        .arg(format!("{}.mp4", vid.title))
                        .status()
                        .expect("Failed to merge")
                        .success()
                    {
                        println!("\nVideo & audio merged successfully");
                    } else {
                        println!("\nVideo & audio merge failed");
                    }
                } else {
                    println!("\nDownload Failed: {}", vid.title);
                }

                println!("\nDeleting video & audio files");

                fs::remove_file("DASH_audio.mp4")
                    .expect("Failed to delete audio file after merging");

                fs::remove_file(best_video).expect("Failed to delete video file after merging");
            }
        }
        _ => println!("{:#?}", vid),
    }

    Ok(())
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
\t-s, --stream_link\t Get streaming link"
    );
}
