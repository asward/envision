# Source this file: . dev.sh
export PATH="$PWD/target/debug:$PATH"
cargo build && eval "$(envision hook ${SHELL##*/})"
