VER := `dasel -i toml 'package.version' < typst.toml | tr -d "'"`
PKG_NAME := `dasel -i toml 'package.name' < typst.toml | tr -d "'" | tr '-' '_'`
DIST_DIR := "dist/{{ PKG_NAME }}/" + VER
LOCAL_PACKAGES_MACOS := "$HOME/Library/Application Support/typst/packages/local"
WASM_OUT := "./lib/wasm"

mod package "./justscripts/package"

install:
    rustup target add wasm32-unknown-unknown

assets:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ ! -d assets ]; then
      exit 0
    fi

    cd assets
    
    # typst c --ppi 300 sheet.typ sheet.png
    # typst c --ppi 300 banner.typ banner.png
    # typst c --ppi 300 --input "banner=github" banner.typ banner_1280_640.png
    
    # oxipng *.png

examples:
  # typst c examples/*.typ --root .. --format png --ppi 300
  # oxipng examples/*.png

manual *args:
    # typst c --root . manual/manual.typ --input version="{{VER}}" {{ args }}

[env("RUSTFLAGS", "--cfg getrandom_backend=\"custom\" -C link-arg=--allow-undefined")]
_build:
    cargo build \
      --release \
      --target wasm32-unknown-unknown \
      --target-dir rust/target \
      --manifest-path rust/Cargo.toml

    mkdir -p {{ WASM_OUT }}
    cp rust/target/wasm32-unknown-unknown/release/{{ PKG_NAME }}.wasm {{ WASM_OUT }}/{{ PKG_NAME }}.wasm

build: _build assets manual examples

bundle: build
    rm -rf dist
    mkdir -p "{{ DIST_DIR }}/src"

    cp -r src/* "{{ DIST_DIR }}/src"
    cp LICENSE "{{ DIST_DIR }}"
    cp README.md "{{ DIST_DIR }}"
    cp typst.toml "{{ DIST_DIR }}"

[macos]
install-dist: clean bundle
    rm -rf "{{ LOCAL_PACKAGES_MACOS }}/{{ PKG_NAME }}"
    mkdir -p "{{ LOCAL_PACKAGES_MACOS }}/{{ PKG_NAME }}"
    cp -r "{{ DIST_DIR }}" "{{ LOCAL_PACKAGES_MACOS }}/{{ PKG_NAME }}"

clean:
    rm -f examples/*.png
    rm -rf dist
    cd rust && cargo clean && rm -rf target

