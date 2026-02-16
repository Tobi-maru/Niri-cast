#!/usr/bin/env bash
set -euo pipefail

cargo build --release
install -Dm755 target/release/niri-cast "$HOME/.cargo/bin/niri-cast"
echo "Installed niri-cast to $HOME/.cargo/bin/niri-cast"
