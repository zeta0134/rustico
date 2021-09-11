# RusticNES - SDL
This is a graphical shell for the [RusticNES](https://github.com/zeta0134/rusticnes-core) emulator, targeting [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2). It is meant to serve as an official release of the emulator for desktop PCs, and should compile and run on Windows and Linux systems. It should theoretically work on Mac systems, but I don't have one to test.

The interface for the emulator is undergoing heavy development. I will try my best to keep this documentation up to date.

## Usage Instructions

Build and run the project with:

```
cargo run --release
```

![Game Window](http://rusticnes.nicholasflynt.com/documentation/game_window.png) 

The emulator opens with no game loaded, and only the Game Window visible. Use **Ctrl-O** to select a .nes file from your computer, which should automatically begin playing. From here, the following keyboard controls do interesting things, mostly useful for debugging.

- F1: VRAM Viewer
- F2: Audio Visualizer
- F3: Memory Viewer
- F4: Live Disassembly
- F5: Piano Roll
- F6: Event Viewer
- Ctrl-O: Open and run a different game
- R: Pause / Play current emulation
- Space: Advance by one instruction
- C: Advance by one *CPU cycle*
- H: Advance to the next scanline
- V: Advance to the start of the next vblank
- S: Write SRAM immediately (if supported, see below)
- Esc: Close the emulator

The following control the Standard NES controller plugged into port 1:

- Arrow Keys: D-Pad
- X: A Button
- Z: B Button
- Enter: Start Button
- R. Shift: Select Button

## Known Issues

At present, not all information has been worked into the GUI windows, so you may find it useful to launch the emulator from a command window or terminal. Extra debug information is printed to stdout during play. In particular, this includes many games that fail to load or boot due to unsupported mappers, and crash states like STP or undefined opcodes. Bug reports are welcome!

Debug panel performance is decent but not great. Some of my computers struggle when all debug windows are open, others are fine. I believe this to be due to the manner in which I'm updating textures with SDL presently, and varying performance of the graphics drivers involved when new textures are created.

Keyboard mapping is planned but not implemented. I intend to support Windows and Linux style platform conventions. I would welcome pull requests for other platforms, ie, Mac-style keyboard mappings, but have no ability to test these myself.

SRAM saving is limited by core project support, and is missing from some mapper types that should have it. The shell will attempt to save regardless, so once saving support is added upstream, a rebuild here should pull it in. Save states and TAS features are planned, but presently unimplemented due to the WIP nature of the core emulator project.

Bug reports are welcome! Ideally emulator bugs should go to [RusticNES-Core](https://github.com/zeta0134/rusticnes-core) and shell specific bugs should go here, but don't sweat the details, I can sort it out.
