mod extractors;
mod helpers;

use crate::extractors::{
    doodstream::doodstream, reddit::reddit, streamhub::streamhub, streamtape::streamtape,
    streamvid::streamvid, substack::substack, twatter::twatter, youtube::youtube,
};

use std::{
    env,
    process::{exit, Command},
};

#[derive(Debug)]
pub struct Vid {
    title: String,
    user_agent: String,
    link: String,
    referrer: String,
}

impl Default for Vid {
    fn default() -> Self {
        Self {
            title: String::new(),
            user_agent: String::from("uwu"),
            link: String::new(),
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
    let mut vid = Vid {
        title: String::new(),
        user_agent: String::new(),
        link: String::new(),
        referrer: String::new(),
    };

    let mut todo = "";
    let mut is_streaming_link = true;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
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
                || a.starts_with("https://libreddit.")
                || a.starts_with("https://teddit.") =>
            {
                vid = reddit(&arg).await
            }
            a if a.starts_with("https://twitter.com/")
                || a.starts_with("https://www.twitter.com/")
                || a.starts_with("https://nitter.") =>
            {
                vid = twatter(&arg).await
            }
            a if a.starts_with("https://streamvid.net") => {
                vid = streamvid(&arg, is_streaming_link).await
            }
            a if a.starts_with("https://streamtape.") => {
                vid = streamtape(&arg, is_streaming_link).await
            }
            _ => {
                eprintln!("Invalid: {}", arg);
                exit(1);
            }
        }
    }

    match todo {
        "print link" => println!("{}", vid.link),
        "play" => {
            println!("Playing {}", vid.title);

            Command::new("mpv")
                .arg(vid.link)
                .arg("--no-terminal")
                .arg("--force-window=immediate")
                .arg(format!("--force-media-title={}", vid.title))
                .arg(format!("--user-agent={}", vid.user_agent))
                .arg(format!("--referrer={}", vid.referrer))
                .spawn()
                .expect("Failed to execute mpv");
        }
        "download" => {
            println!("Downloading {}", vid.title);

            if Command::new("aria2c")
                .arg(vid.link)
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
        }
        _ => println!("{:#?}", vid),
    }

    Ok(())
}
