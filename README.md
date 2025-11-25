# NHK Radio Player

<img width="884" height="404" alt="image" src="https://github.com/user-attachments/assets/59cb8d9a-8f4d-4529-b7cd-917714124e55" />

A CLI radio player for NHK Radio (Japan) written in Rust.

## Features

- Play NHK radio streams (R1, R2, FM)
  - R1: NHK ラジオ第 1
  - R2: NHK ラジオ第 2
  - FM: NHK-FM
- List available areas and streams
- Show current program information
- Support for HLS streaming with AES-128 encryption
- Command-line interface

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

### List Available Areas

```bash
nhk-radio-player area
```

### Show Current Program Information

```bash
nhk-radio-player program <area_code>
```

Example:

```bash
nhk-radio-player program tokyo
```

### Play Radio Stream

```bash
nhk-radio-player play <area_code> <channel>
```

Where `<channel>` is one of: `r1`, `r2`, or `fm`

Example:

```bash
nhk-radio-player play tokyo r1  # Play NHK Radio 1 in Tokyo
```

### List All Available Streams

```bash
nhk-radio-player list
```

## Architecture

The application consists of several modules:

- `client`: HTTP client for fetching NHK Radio API data
- `types`: Type definitions for NHK Radio API responses
- `m3u8`: M3U8 playlist parser
- `crypto`: AES-128-CBC decryption for encrypted segments
- `player`: Audio streaming and playback
- `cli`: Command-line interface

## License

MIT

## Author

Mao Nabeta

