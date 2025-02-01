name := 'observatory'
named := 'monitord'
appid := 'io.github.CosmicUtils.Observatory'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / name
bind-src := named / 'target' / 'release' / named
bin-dst := base-dir / 'bin' / name
bind-dst := base-dir / 'bin' / named

daemon := 'monitord.service'
daemon-src := 'resources' / daemon
daemon-dst := clean(rootdir / 'etc') / 'systemd' / 'system' / daemon

policy := appid + '.conf'
policy-src := 'resources' / 'app.policy'
policy-dst := clean(rootdir / prefix) / 'share' / 'dbus-1' / 'system.d' / policy

desktop := appid + '.desktop'
desktop-src := 'resources' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

appdata := appid + '.metainfo.xml'
appdata-src := 'resources' / appdata
appdata-dst := clean(rootdir / prefix) / 'share' / 'appdata' / appdata

icons-src := 'resources' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

icon-svg-src := icons-src / 'scalable' / 'apps' / 'icon.svg'
icon-svg-dst := icons-dst / 'scalable' / 'apps' / appid + '.svg'

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
    cargo build --manifest-path ./monitord/Cargo.toml {{args}}
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

# Run the application for testing purposes
run *args:
    env RUST_BACKTRACE=full cargo run --release {{args}}

# Installs files
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0755 {{bind-src}} {{bind-dst}}
    install -Dm0664 {{daemon-src}} {{daemon-dst}}
    install -Dm0664 {{policy-src}} {{policy-dst}}
    systemctl daemon-reload
    systemctl enable {{daemon}}
    systemctl start {{daemon}}
    install -Dm0644 resources/app.desktop {{desktop-dst}}
    install -Dm0644 resources/app.metainfo.xml {{appdata-dst}}

# Uninstalls installed files
uninstall:
    rm {{bin-dst}} {{bind-dst}} {{desktop-dst}} {{appdata-dst}}

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
