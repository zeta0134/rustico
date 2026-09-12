#!/bin/bash

# if the user hasn't provided an argument, print a help message and bail
# (otherwise the constructed zip will uselessly contain no ROM)
if [ $# -eq 0 ]; then
    echo "Usage: make_itch_embed.sh path/to/game.nes"
    exit 1
fi


# first make sure the core files are built and up to date
bash ./build.sh

# copy build artifacts into the folder
cp public/rustico_wasm.d.ts rustico-itch.io-embed
cp public/rustico_wasm.js rustico-itch.io-embed
cp public/rustico_wasm_bg.wasm rustico-itch.io-embed
cp public/rustico_wasm_bg.wasm.d.ts rustico-itch.io-embed

# copy the provided rom path into the folder, so it starts on load
cp "$1" rustico-itch.io-embed/autorun.nes

# zip that all up, which for zip utility reasons requires irritating
# current directory management
cd rustico-itch.io-embed
zip rustico-itch.io-embed.zip *
cd ..
mv -f rustico-itch.io-embed/rustico-itch.io-embed.zip rustico-itch.io-embed.zip
