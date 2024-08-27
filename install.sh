#!/bin/bash

set -e

echo "Copying resources folder into $HOME/.config/caveripper/"
mkdir -p ~/.config/caveripper
cp -r resources ~/.config/caveripper
# CaveRipper requires $HOME/.config/caveripper/assets to exist as a directory, even if it is empty.
mkdir ~/.config/caveripper/assets

echo "Installing Caveripper"
cargo install --path ./cli

echo "Done. Make sure your Cargo bin directory is on PATH (~/.cargo/bin on Linux/Macos)."
