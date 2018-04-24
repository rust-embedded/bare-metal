set -euxo pipefail

main() {
    cargo check --target $TARGET --no-default-features

    if [ $TRAVIS_RUST_VERSION = nightly ]; then
        cargo check --target $TARGET
    fi
}

main
