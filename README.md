# hdiff-apply

## Requirements:
- [Rust](https://www.rust-lang.org/tools/install)
- [HDiffPatch](https://github.com/sisong/HDiffPatch)

## Command line usage:
hdiff-apply "game_folder"

## How to use (easiest way):
1. Make sure you have HDiffPatch in your system environment variables or in the same directory as ```hdiff-apply.exe```
2. Extract hdiff update package to SR folder
3. Download the repository
4. Run cmd in the root directory
5. Enter ```cargo build -r``` to build the executable in release mode
6. Move ```hdiff-apply\target\release\hdiff-apply.exe ``` to the same folder where the game is located
7. Run ```hdiff-apply.exe``` and wait for it to finish
