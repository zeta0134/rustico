#!/bin/bash
VERSION=$(grep "version" Cargo.toml | awk -F'"' '{print $2}')

echo "Cleaning up old build artifacts..."
rm -rf packages
cargo clean

echo "Building rusticnes-sdl version ${VERSION}"
mkdir packages

echo "=== Linux - x86_64 ==="
cargo build --release --target x86_64-unknown-linux-gnu
cd target/x86_64-unknown-linux-gnu/release
tar zcf rusticnes-sdl-${VERSION}.Linux.x86_64.tar.gz rusticnes-sdl
cd ../../..
mv target/x86_64-unknown-linux-gnu/release/rusticnes-sdl-${VERSION}.Linux.x86_64.tar.gz packages

echo "=== Windows - x86_64 ==="
cargo build --release --target x86_64-pc-windows-gnu
cd target/x86_64-pc-windows-gnu/release
cp ../../../SDL2.dll .
zip rusticnes-sdl-${VERSION}.Windows.x86_64.tar.gz rusticnes-sdl.exe SDL2.dll
cd ../../..
mv target/x86_64-pc-windows-gnu/release/rusticnes-sdl-${VERSION}.Windows.x86_64.tar.gz packages

echo "Complete!"