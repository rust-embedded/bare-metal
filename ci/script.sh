set -euxo pipefail

main() {
    cargo check --target $TARGET

    if [ $TRAVIS_RUST_VERSION = nightly ]; then
        cargo check --features const-fn --target $TARGET
    fi
}

main
