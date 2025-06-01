# hdiff-apply

## Requirements:
- [Rust](https://www.rust-lang.org/tools/install) for compiling

## How to use (easiest way):
1. Download the latest version from [releases](https://github.com/nie4/hdiff-apply/releases)
2. Move `hdiff-apply.exe` to the same folder where the game is located
3. Put the hdiff update package to SR folder (without extracting)
4. Run `hdiff-apply.exe` and wait for it to finish

## CLI usage:
```bash
Usage: hdiff-apply.exe [OPTIONS] [GAME_PATH]

Arguments:
  [GAME_PATH]

Options:
      --skip-version-check
  -h, --help                Print help
  ```

## Compiling:
```bash
cargo build -r
```

## Credits:
- [HDiffPatch](https://github.com/sisong/HDiffPatch) for the patching utility (`hpatchz.exe`)