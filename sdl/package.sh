#!/bin/bash
VERSION=$(grep "version" Cargo.toml | awk -F'"' '{print $2}')

echo "Cleaning up old build artifacts..."
rm -rf packages
cargo clean

echo "Building rustico-sdl version ${VERSION}"
mkdir packages

echo "=== Linux - x86_64 ==="
cargo build --release --target x86_64-unknown-linux-gnu
cp README.md target/x86_64-unknown-linux-gnu/release
cp LICENSE.txt target/x86_64-unknown-linux-gnu/release
cd target/x86_64-unknown-linux-gnu/release
tar zcf rustico-sdl-${VERSION}.Linux.x86_64.tar.gz rustico-sdl README.md LICENSE.txt
cd ../../..
mv target/x86_64-unknown-linux-gnu/release/rustico-sdl-${VERSION}.Linux.x86_64.tar.gz packages

echo "=== Windows - x86_64 ==="
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
cp README.md target/x86_64-pc-windows-gnu/release
cp LICENSE.txt target/x86_64-pc-windows-gnu/release
cd target/x86_64-pc-windows-gnu/release
cp ../../../SDL2.dll .
zip rustico-sdl-${VERSION}.Windows.x86_64.zip rustico-sdl.exe SDL2.dll README.md LICENSE.txt
cd ../../..
mv target/x86_64-pc-windows-gnu/release/rustico-sdl-${VERSION}.Windows.x86_64.zip packages

echo "Complete!"