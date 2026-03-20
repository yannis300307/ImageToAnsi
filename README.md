# Image To Ansi converter
A very basic image to ansi escape sequences converter. It takes an image, a width and a color mode as input and prints the image to the stdout. The program supports both 24 bit and 8 bit color modes.

## Build
Use the standard cargo build command:
```bash
cargo build --release
```

## Usage
```bash
./image_to_ansi <image path> <width> <mode full/8bit>
```
The result can be piped into a file or to the clipboard using xclip (that you must install) e.g.
```bash
./image_to_ansi image.png 64 full | xclip -sel clip
```
```bash
./image_to_ansi image.png 42 8bit > out.txt
```

The error management is very basic and the program will panic in case of an error. I could enhance this behavior in the future.
## Screenshot

Full color mode:
<img width="913" height="475" alt="image" src="https://github.com/user-attachments/assets/898594ea-2069-4ff1-a482-9e0c45c23ea1" />

8bit color mode:
<img width="913" height="476" alt="image" src="https://github.com/user-attachments/assets/1eafd5f4-b4a2-43b0-8abe-494ec744bedf" />
