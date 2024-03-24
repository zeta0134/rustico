# Rustico - SDL
This is a graphical shell for the Rustico emulator, targeting [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2). It is the legacy desktop build, and should compile and run on Windows and Linux systems. It should theoretically work on Mac systems, but I don't test this often.

This shell is almost certainly to be deprecated in favor of the shiny new `/egui` interface, but for the moment it is the more complete implementation out of the pair.

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

![Game Window](https://rusticnes.reploid.cafe/documentation/game_window.png) 

The emulator opens with no cartridge loaded, and only the Game Window visible. Use **Ctrl-O** to select either a `.nes`, `.nsf` or `.fds` file from your computer, which should automatically begin playing. From here, the following keyboard controls do interesting things, mostly useful for debugging.

- F1: VRAM Viewer
- F2: Audio Visualizer
- F3: Memory Viewer
- F4: Live Disassembly
- F5: Piano Roll
- F6: Event Viewer
- Ctrl-O: Open and run a different file.
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

The following keys operate the Standard Controller plugged into port 1:

- Arrow Keys: D-Pad
- X: A Button
- Z: B Button
- Enter: Start Button
- R. Shift: Select Button

Both the Audio Visualizer (F2) and Piano Roll (F5) support channel muting. Click the waveforms to toggle.

## Known Issues

Error messages and extended debug output is not yet presented in the GUI. You may find it useful to launch the emulator from a command window or terminal. Extra debug information is printed to stdout during play. In particular, this includes many cartridges that fail to load or boot due to unsupported mappers, and crash states like STP or undefined opcodes. Bug reports are welcome!

Debug panel performance is decent but not great. Machines with weaker GPUs (especially underpowered devices, like Raspberry Pi) may struggle to paint debug windows at high framerates. The Audio Visualizer (F2) has trouble displaying fully on some monitors when many expansion channels are present. The Piano Roll currently misses APU frame events when stepping one frame at a time; this will correct itself when emulation is resumed.

## Notable Missing Features

Keyboard mapping is planned but not implemented.

SRAM saving is limited by `/core` project support, and is missing from a small number of mapper types that should have it. Save states and TAS features are planned, but presently unimplemented.

PAL support is entirely unimplemented upstream. If PAL titles run at all, expect detuned audio, timing problems and major visual glitches.

NSF and FDS are implemented, however for FDS you'll need to supply a BIOS file. The hardware is emulated, so a homebrew BIOS may work if you don't have access to the original, though this is untested. VRC7 is implemented but the audio is not yet perfect.