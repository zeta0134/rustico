# Rustico

This is a 2A03+2C02 console emulator written in the Rust programming language. Its emulated capabilities are similar to the original NES and Famicom consoles and their _many_ clones. I started this project to teach myself Rust, and it quickly got out of hand, so... here we are. Rustico's primary utility is providing a stable and reasonably accurate base to run "modern retro" software, including homebrew and some of my own original games. Music and chiptunes are my primary focus, so audio emulation is a high priority.

The emulator is split up into the Core library and platform specific shells. The `/core` crate contains the main emulator with as few external dependencies as possible (presently just Rust's standard FileIO functions) so that it remains reasonably portable. This is the only crate you should need if you are building your own shell or custom game wrapper. At the moment the project is in constant flux and lacks what I'd call a stable API, so do proceed with caution.

For the moment, the SDL shell at `/sdl` is the most complete implementation with all features available. I've tested it on Windows and Arch Linux, and it should run on Mac, and any other platform that [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) supports. An updated `/egui` shell is in the works, and will eventually replace the SDL build as the primary recommendation, but it is currently a bit barebones.

I'm striving for cycle accuracy with the emulator. While it works and runs a wide variety of software, it presently falls short of this goal. I am presently most focused on getting the base emulator to run properly, and pass various [accuracy tests](http://tasvideos.org/EmulatorResources/NESAccuracyTests.html). Mapper support should be easy to add as I go. Here is the current state of the major systems:

## 6502 CPU

- All instructions, including unofficial instructions, NOPs, and STPs
- Cycle-stepped; CPU can pause "between" instruction ticks, and implements dummy access patterns
- Cycle delays with the DMA unit are implemented, but known to have flawed timings. It reproduces DPCM corruption glitches, but not in the same manner as the original hardware.

## APU

- Feature complete as far as I can tell. Pulse, Triangle, Noise, and DMC are all working properly.
- DMC wait delay is implemented, but not particularly accurately (see above)
- Audio is emulated at 1.7 MHz then downsampled, so high noise and other unusual timbres are reproduced faithfully
- Mixing of 2A03 channels is hardware accurate to within +/- a few dB. Mixing of expansion channels is under active research
- MMC5, VRC6, S5B, N163, and FDS expansion audio is working correctly
- VRC7 is implemented, but not quite perfect; more research on the ADSR behavior is needed

## PPU

- Memory mapping, including cartridge mapper support, is all implemented and should be working.
- Nametable mirroring modes work correctly, and are fully controlled by advanced mappers.
- Cycle timing is quite good, though not perfect. Tricky games like Battletoads appear to run correctly, and most advanced raster tricks and homebrew tomfoolery is stable.
- Sprite overflow is implemented correctly. The sprite overflow bug is not, so games relying on the behavior of the sprite overflow flag will encounter accuracy problems relative to real hardware.

## Input

- Standard Controllers plugged into ports 1 and 2 is implemented. 
- Multiple controllers and additional peripheral support (Light Zapper, Track and Field Mat, Knitting Machine, etc) is planned, but not implemented.

## Mappers

- Many common mappers are supported, but there are literally hundreds of the things still missing. There are a *lot* of these things.
    - I'm getting to these at my own pace, but if you need a particular mapper, file an issue. I'm happy to reprioritize if it unblocks a cool project!
- Advanced mappers like MMC5 and Rainbow are implemented, though not fully tested due to a lack of adequate software. Uncommon features may have bugs! Reports are quite welcome.
- Some of blarggs mapper tests do not pass, especially those involving timing
- FDS is now implemented! A separate BIOS is currently required, though the hardware is properly emulated so a homebrew replacement should in theory work as well as the original. Shells supporting FDS will prompt for the BIOS path on first load.
- Non-NTSC features (PAL, Vs System, etc) are entirely unimplemented. PAL support is planned.
