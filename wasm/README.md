# RusticNES-wasm

A web based interface for [RusticNES](https://github.com/zeta0134/rusticnes-core), running on Web Assembly for sweet retro action in the browser. Includes a basic web shell to operate the emulator, and relies on [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) to simplify the interface between the emulator core and the JavaScript UI. This is a reasonably early work in progress, expect bugs!

A live demo can be found [here](http://rusticnes.reploid.cafe/wasm/?cartridge=super-bat-puncher.nes), running the AWESOME homebrew [Super Bat Puncher](http://morphcat.de/superbatpuncher/), hosted with permission from [Morphcat Games](http://morphcat.de/) whom you should totally check out.

## Building

First, install the wasm32-unknown-unknown target. As of this writing, this is only available in rust nightly, so install that too as the build script expects it:

```
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Next, install wasm-bindgen, and add your local cargo bin directory to your $PATH if you haven't done so already:

```
cargo install wasm-bindgen-cli
export PATH=$PATH:~/.cargo/bin
```

Finally, run the `./build.sh` script in the main folder. Afterwards, for Firefox you should be able to open `public/index.html` and run the emulator. For Chrome, you'll need to host the "public" folder on a (possibly local) webserver first, as Chrome will not permit the project to load the .wasm files from the file:// protocol.

## Usage

Use the "Load" button to open up a `.nes` file from your computer. Alternately, you can pass in a query string: `?cartridge=game.nes`. This is read as a standard URL, so paths relative to the index page work, as do fully qualified URLs that point to a valid `.nes` file. Note that `.zip` and other archives are not supported. 

The emulator will try to maintain 60 FPS, the success of which depends on how powerful your computer, tablet, or phone is. You may enter fullscreen mode by double-clicking on the game screen. If your game supports SRAM (not many NES games do), it will be persisted to local storage in your browser; note that the filename / URL is used to determine which save file is loaded. Gamepad controls default to the following, and may be remapped:

```
D-Pad: Arrow Keys
A: Z
B: X
Start: Enter
Select: Shift
```

## Planned Features

- Speed Improvements
- Better support for touchscreens

Needs rusticnes-core support:

- NES Zapper
- Save States

## General Notes

All shells to RusticNES are limited by features present in the [core emulator](https://github.com/zeta0134/rusticnes-core), so some features need to be implemented upstream first. This applies to emulation bugs as well, especially with regards to missing mapper support. Bug reports are welcome! I don't have every cartridge out there to test with, so I may not be aware that your favorite game doesn't boot, but I love a good challenge.

Several technologies involved in this project are moving targets, not the least of which being WebAssembly itself. Web Audio is particularly new, and might run into strange issues; I've heard reports of it deciding to mix audio at 192 KHz on some Windows systems, which should technically work but will definitely make the emulator work a lot harder. As of this writing, the emulator is known to work in both Firefox and Chrome, and has faster performance on Firefox. It should in theory be able to run on Microsoft Edge, but wasm-bindgen fails to load due to a missing (planned) feature. I'll try to correct for this later, but might need to wait on Microsoft for a proper fix. I'd like to make it as standards compliant as possible, so if it's not working in your favorite browser, (and you're reasonably confident that browser at least supports WASM), bug reports are welcome!

If you'd like to test the emulator in more detail, right now the [RusticNES-SDL](https://github.com/zeta0134/rusticnes-sdl) frontend is much more mature, and supports many debug features. You should give it a try, as both shells use the same [rusticnes-core](https://github.com/zeta0134/rusticnes-core) backend. Emulation accuracy should be identical between the various frontends.

Feel free to distribute this emulator! I mostly ask for common courtesy and respect. Fork, modify, extend, and have fun with it! However, please do be respectful of copyright laws. The emulator will play almost any game, even commercial games, but this does not give you the legal right to distribute those game files on your personal website. Please obey the local laws in your country / state, and if you can, be sure to give Nintendo and other publishers your support by buying their current titles and Virtual Console releases. Unless you are distributing a homebrew game that you wrote, I would not recommend hosting game files alongside the emulator if you choose make your build public.
