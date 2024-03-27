# titans

Blazingly fast scraper

# Install

#### Linux/Windows/Android

- Get the binary from the [release page](https://github.com/Based-Programmer/titans/releases)

## Build
#### Any OS

````sh
git clone 'https://github.com/Based-Programmer/titans' && \
cd titans && \
cargo build --release
````

- Then move it to your $PATH

````sh
sudo cp target/release/titans /usr/local/bin/
````

- Or Build it directly with cargo

````sh
cargo install titans
````

- Then move it to your $PATH

````sh
sudo cp "$CARGO_HOME"/bin/titans /usr/local/bin/
````

- Or better add $CARGO_HOME to your $PATH

- In your .zprofile, .bash_profile or .fish_profile ?

````sh
export PATH="$CARGO_HOME/bin:$PATH"
````
## Usage

````
titans <args> <url>
````

#### Example

- Get data

````sh
titans 'https://dooood.com/d/0hdlp0641u82'
````

- Play

````sh
titans -p 'https://www.youtube.com/watch?v=784JWR4oxOI'
````

- Download (frontends are also supported)

````sh
titans -d 'https://nitter.net/stillgray/status/1670812043090497538#m'
````

- More at help

````sh
titans -h
````

## Optimal Dependencies

- mpv or mpv-android (Streaming video)
- aria2 (for downloading)
- ffmpeg (merging downloaded video & audio after download)

## Supported Sites

- [bitchute](https://www.bitchute.com)
- [doodstream](https://doodstream.com)
- [libsyn](https://libsyn.com) only play links (https://html5-player.libsyn.com/episode/id/[0-9]*) as of now
- [mp4upload](https://www.mp4upload.com)
- [odysee](https://odysee.com)
- [reddit](https://www.reddit.com)
- [rokfin](https://rokfin.com) videos only (not bites & live streams)
- [rumble](https://rumble.com)
- [spotify](https://www.spotify.com) only open links like embedded podcasts
- [streamdav](https://streamdav.com)
- [streamhub](https://streamhub.to)
- [streamtape](https://streamtape.xyz)
- [streamvid](https://streamvid.net)
- [substack](https://www.substack.com)
- [twatter](https://twitter.com)
- [vtube](https://vtbe.network)
- [wolfstream](https://wolfstream.tv)
- [youtube](https://www.youtube.com)
