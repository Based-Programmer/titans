# titans
Blazingly fast scraper

# Install

#### Linux/Mac
- First of all install rust then
````
git clone 'https://github.com/Based-Programmer/titans' && \
cd titans && \
cargo build --release
````

- Then add it to your $PATH
````
sudo cp target/release/titans /usr/local/bin/
````

## Usage
````
titans <argument> <url>
````

#### Example

- Get data
````
titans 'https://dooood.com/d/0hdlp0641u82'
````
- Play
````
titans -p 'https://www.youtube.com/watch?v=luOgEhLE2sg'
````
- More at help
````
titans -h
````

## Optimal Dependencies
- mpv (Streaming video)
- aria2 (for downloading)
- ffmpeg (merging downloaded video & audio)
