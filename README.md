# startmc

A CLI tool and library for launching Minecraft clients.

## Features

### Run Minecraft

Run a Minecraft instance named `default` (config is `~/.config/startmc/default.toml`):
```sh
startmc
```

Run using a custom startmc config file:
```sh
startmc ./myinstance.toml
```

### Download content from the internet

Download mods:
```sh
startmc -U MOD_URL ANOTHER_MOD_URL MANY_MORE_MOD_URLS
```

Download resourcepacks:
```sh
startmc -U RESOURCEPACK_URL ANOTHER_RESOURCEPACK_URL MANY_MORE_RESOURCEPACK_URLS
```

### (TODO) Download and update content from Modrinth (TODO) and CurseForge (TODO (TODO))

Download mods from Modrinth:
```sh
startmc -S fabric-api sodium
```

Update all your installed mods:
```sh
startmc -Syu
```

## Installation

```sh
git clone https://github.com/startmc/startmc.git
cd startmc
cargo install --path .
```

### Cross-compiling to Windows from Linux

1. Install mingw-w64-gcc (this is for Arch Linux, for other distros it might be different)
```sh
sudo pacman -S mingw-w64-gcc
```
2. Cross-compile!
```sh
cargo build --target x86_64-pc-windows-gnu --release
```
3. The binary should be in `target/x86_64-pc-windows-gnu/release/startmc.exe`.

## Usage as a library

You can use this library to integrate any part of startmc's functionality into your own program, or invoke it programmatically, or make a wrapper around it, or anything else that comes to your mind!

If you want to know more, please either generate API docs with `cargo doc --package startmc --lib`, or read the source code.

If you see bad docs, or library-unfriendly code, or whatever else that is making it harder to use, please open an issue or, if you want, fix it and open a PR, I will greatly appreciate it.

### Examples

Invoke the CLI with your own args programmatically:
```rust
use startmc::cli::Cli;
let cli = Cli::parse_from(["startmc", "-I", "-m", "1.20.1", "-f", "0.16.9"]).unwrap();
cli.exec().await.unwrap();
```

Or construct it yourself:
```rust
use startmc::cli::*;

let cli = Cli {
    instance: "default".to_string(),
    command: CliCommand::Init(CliInit {
        version: Some("1.20.1".to_string()),
        fabric: Some("0.16.9".to_string()),
    }),
};

cli.exec().await.unwrap();
```

Use the cache:
```rust
use startmc::cache::*;
use startmc::mojapi::model::fabric::*;

let minecraft_version = "1.20.1";
let fabric_versions = use_cached_json::<FabricVersionsGame>(&format!(
    "{}/{minecraft_version}",
    FABRIC_VERSIONS_GAME
)).await.unwrap();
```

## Credits

- [wiki.vg](https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge) and all its contibutions for documenting, like, the entire asset, library, etc download and launch progress.
- [pacman](https://gitlab.archlinux.org/pacman/pacman) for inspiration for the CLI interface. Seriously, it's awesome.
