# titans

Blazingly fast scraper

# Install

#### Linux/Mac

- Get the binary from the [release page](https://github.com/Based-Programmer/titans/releases)

- Build

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
titans <argument> <url>
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

- mpv (Streaming video)
- aria2 (for downloading)
- ffmpeg (merging downloaded video & audio)

## Supported Sites

- [bitchute](https://www.bitchute.com)
- [doodstream](https://doodstream.com)
- [mp4upload](https://www.mp4upload.com)
- [odysee](https://odysee.com)
- [reddit](https://www.reddit.com)
- [rokfin](https://rokfin.com)
- [rumble](https://rumble.com)
- [streamdav](https://streamdav.com)
- [streamhub](https://streamhub.to)
- [streamtape](https://streamtape.xyz)
- [streamvid](https://streamvid.net)
- [substack](https://www.substack.com)
- [twatter](https://twitter.com)
- [vtube](https://vtbe.network)
- [wolfstream](https://wolfstream.tv)
- [youtube](https://www.youtube.com)
