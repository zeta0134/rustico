# Rustico - WASM

A web based shell for Rustico, running on Web Assembly for sweet retro action in the browser. Includes a basic web shell to operate the emulator, and relies on [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) to simplify the interface between the emulator core and the JavaScript UI. This is a reasonably early work in progress, expect bugs!

A live demo can be found [here](http://rustico.reploid.cafe/wasm/?cartridge=super-bat-puncher.nes), running the AWESOME homebrew [Super Bat Puncher](http://morphcat.de/superbatpuncher/), hosted with permission from [Morphcat Games](http://morphcat.de/) whom you should totally check out.

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

Use the "Load" button to open up a `.nes` or `.nsf` file from your computer. (FDS is not currently supported, as there is no way to provide the BIOS.) Alternately, you can pass in a query string: `?cartridge=my_chiptune_album.nsf`. This is read as a standard URL, so paths relative to the index page work, as do fully qualified URLs that point to a valid cartridge file, subject to CORS limitations and usual web-based networking restrictions. Note that `.zip` and other archive formats are not supported. 

The emulator will try to maintain 60.01 FPS, the success of which depends on how powerful your computer, tablet, or phone is. If your software supports SRAM, it will be persisted to local storage in your browser; note that the filename / URL is used to determine which SRAM file is loaded on the next run. Gamepad controls default to the following, and may be remapped:

```
D-Pad: Arrow Keys
A: Z
B: X
Start: Enter
Select: Shift
```

## Planned Features

- Speed Improvements
- Much better support for touchscreens
- Additional debugging features (mostly limited by canvas bandwidth)
- SRAM Import / Export

Needs core support:

- Zapper Light Gun
- Save States

## General Notes

All shells to Rustico are limited by features present in the `/core` emulator. This applies to emulation bugs as well, especially with regards to missing mapper support. Bug reports are welcome!

Several technologies involved in this project are moving targets, not the least of which being WebAssembly itself. Web Audio is particularly new, and might run into strange issues; I've heard reports of it deciding to mix audio at 192 KHz on some Windows systems, which should technically run but will definitely make the emulator work a lot harder. As of this writing, the emulator is known to work in both Firefox and Chrome, and has faster performance on Firefox. It should in theory be able to run on Microsoft Edge, but wasm-bindgen fails to load due to a missing (planned) feature. Touchscreen support seems to work well on Mobile Safari, though fullscreen mode trips a strange "fullscreen keyboard" protection on some iOS versions. Performance on Android browsers is generally not great.

If you'd like to test the emulator in more detail, right now the `/sdl` frontend is much more mature, and supports many debug features. All shells share the same core library, so emulation accuracy should be identical between them.

Feel free to distribute this emulator! However, please do be respectful of copyright laws. The emulator will play almost any game, even commercial games, but this does not give you the legal right to distribute those game files on your personal website. Unless you are distributing your own software, I would not recommend hosting game files alongside your build. Obey the local laws in your country / region, and if you can, be sure to support game publishers by buying their current titles and Virtual Console releases.
