# titan
Blazingly fast scraper

# Install

#### Linux/Mac
- First of all install rust then
````
git clone 'https://github.com/Based-Programmer/titan' && \
cd titan && \
cargo build --release
````

- Then add it to your $PATH
````
sudo cp target/release/titan /usr/local/bin/
````

## Usage
````
titan <argument> <url>
````

#### Example

- Get data
````
titan 'https://dooood.com/d/0hdlp0641u82'
````
- Play
````
titan -p 'https://www.youtube.com/watch?v=luOgEhLE2sg'
````
- More at help
````
titan -h
````

## Optimal Dependencies
- mpv (Streaming video)
- aria2 (for downloading)
- ffmpeg (merging downloaded video & audio)
