name := 'observatory'
export APPID := 'org.cosmic-utils.observatory'

rootdir := ''
prefix := '/usr'
flatpak-prefix := '/app'

base-dir := absolute_path(clean(rootdir / prefix))
flatpak-base-dir := absolute_path(clean(rootdir / flatpak-prefix))

export INSTALL_DIR := base-dir / 'share'

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name
flatpak-bin-dst := flatpak-base-dir / 'bin' / name

desktop := APPID + '.desktop'
desktop-src := 'res' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

metainfo := APPID + '.metainfo.xml'
metainfo-src := 'res' / metainfo
metainfo-dst := clean(rootdir / prefix) / 'share' / 'metainfo' / metainfo

icons-src := 'res' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

dev *args:
    cargo fmt
    just run {{args}}

# Run with debug logs
run *args:
    env RUST_LOG=observatory=debug RUST_BACKTRACE=full cargo run --release {{args}}

# Installs files
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}
    for size in `ls {{icons-src}}`; do \
        install -Dm0644 "{{icons-src}}/$size/apps/{{APPID}}.png" "{{icons-dst}}/$size/apps/{{APPID}}.png"; \
    done

sources-gen:
  python3 flatpak-cargo-generator.py ./Cargo.lock -o cargo-sources.json

install-sdk:
  flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
  flatpak install --noninteractive --user flathub \
    org.freedesktop.Platform//24.08 \
    org.freedesktop.Sdk//24.08 \
    org.freedesktop.Sdk.Extension.rust-stable//24.08 \
    org.freedesktop.Sdk.Extension.llvm18//24.08

package-flatpak:
    flatpak-builder --force-clean --user --install-deps-from=flathub --repo=repo --install builddir org.cosmic-utils.observatory.json --verbose
    flatpak build-bundle repo org.cosmic-utils.observatory.flatpak org.cosmic-utils.observatory --runtime-repo=https://github.com/cosmic-utils/observatory

debpkg: build-release
    cargo deb --no-build

# Uninstalls installed files
uninstall:
    rm {{bin-dst}}
    rm {{desktop-dst}}
    rm {{metainfo-dst}}
    for size in `ls {{icons-src}}`; do \
        rm "{{icons-dst}}/$size/apps/{{APPID}}.png"; \
    done

# Vendor dependencies locally
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
