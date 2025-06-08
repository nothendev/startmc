# startmc

A CLI tool for launching Minecraft clients.

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

## Credits

- [wiki.vg](https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge) and all its contibutions for documenting, like, the entire asset, library, etc download and launch progress.
- [pacman](https://gitlab.archlinux.org/pacman/pacman) for inspiration for the CLI interface. Seriously, it's awesome.
