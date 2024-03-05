# RusticNES - SDL
This is a graphical shell for the [RusticNES](https://github.com/zeta0134/rusticnes-core) emulator, targeting [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2). It is meant to serve as an official release of the emulator for desktop PCs, and should compile and run on Windows and Linux systems. It should theoretically work on Mac systems, but I don't have one to test.

The interface for the emulator is undergoing heavy development. I will try my best to keep this documentation up to date.

## Usage Instructions

Install dependencies `gtk3`, `sdl2`, and if you intend to package for Windows, also `mingw-w64`. On Debian systems:

```
sudo apt install libgtk-3-dev libsdl2-dev mingw-w64
```

Now you can build and run the project with:

```
cargo run --release
```

And produce archives for Linux and Windows with:

```
./package.sh
```

![Game Window](http://rusticnes.nicholasflynt.com/documentation/game_window.png) 

The emulator opens with no game loaded, and only the Game Window visible. Use **Ctrl-O** to select a .nes file from your computer, which should automatically begin playing. From here, the following keyboard controls do interesting things, mostly useful for debugging.

- F1: VRAM Viewer
- F2: Audio Visualizer
- F3: Memory Viewer
- F4: Live Disassembly
- F5: Piano Roll
- F6: Event Viewer
- Ctrl-O: Open and run a different game.
- P: Pause / Resume emulation
- R: Send Reset signal
- Space: Advance by one instruction
- C: Advance by one *CPU cycle*
- H: Advance to the next scanline
- V: Advance to the start of the next vblank
- S: Write SRAM immediately (if supported, see below)
- Esc: Close the emulator
- Numpad +: Increase zoom on the main screen
- Numpad -: Decrease zoom on the main screen
- Numpad \*: Disable overscan (Show full 256x240 PPU output)
- Ctrl+A: Begin dumping audio to `audiodump.raw` (Signed 16bit, Big Endian, Mono)

The following control the Standard NES controller plugged into port 1:

- Arrow Keys: D-Pad
- X: A Button
- Z: B Button
- Enter: Start Button
- R. Shift: Select Button

Both the Audio Visualizer (F2) and Piano Roll (F5) support channel muting. Click the waveforms to toggle.

## Known Issues

Error messages and extended debug output is not yet presented in the GUI. You may find it useful to launch the emulator from a command window or terminal. Extra debug information is printed to stdout during play. In particular, this includes many games that fail to load or boot due to unsupported mappers, and crash states like STP or undefined opcodes. Bug reports are welcome!

Debug panel performance is decent but not great. Machines with weaker GPUs (especially underpowered devices, like Raspberry Pi) may struggle to paint debug windows at high framerates. The Audio Visualizer (F2) has trouble displaying fully on some monitors when many expansion channels are present. The Piano Roll currently misses APU frame events when stepping one frame at a time; this will correct itself when emulation is resumed.

Bug reports are welcome! Ideally emulator bugs should go to [RusticNES-Core](https://github.com/zeta0134/rusticnes-core) and shell specific bugs should go here, but don't sweat the details, I can sort it out.

## Notable Missing Features

Keyboard mapping is planned but not implemented. I intend to support Windows and Linux style platform conventions. I would welcome pull requests for other platforms, ie, Mac-style keyboard mappings, but have no ability to test these myself.

SRAM saving is limited by `rusticnes-core` project support, and is missing from some mapper types that should have it. Save states and TAS features are planned, but presently unimplemented.

PAL support is entirely unimplemented upstream. If PAL titles run at all, expect detuned audio, timing problems and major visual glitches.

While NSF files are supported, note that the FamiCom Disk System and VRC7 mappers are not yet implemented upstream. Any NSF files depending on these channels should still run, but the relevant expansion audio will be missing entirely.