set -euxo pipefail

export RUSTFLAGS="-D warnings"

main() {
    cargo check --target $TARGET

    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo test
    fi
}

main
