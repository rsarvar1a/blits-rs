inspect:
    cargo remark build --open

profile:
    RUSTFLAGS="-C force-frame-pointers=yes" cargo build --profile perf-dev
    mkdir -p perf
    -cd perf && perf record -F 200 -g --call-graph fp -o perf-dev.data ../target/perf-dev/blits -v
    -cd perf && perf report -i perf-dev.data

profile-release:
    RUSTFLAGS="-C force-frame-pointers=yes" cargo build --profile perf
    mkdir -p perf
    -cd perf && perf record -F 600 -g --call-graph fp -o perf.data ../target/perf/blits -v
    -cd perf && perf report -i perf.data

profile-opt:
    RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release
    mkdir -p perf
    -cd perf && perf record -F 1000 --call-graph fp -o perf-release.data ../target/release/blits -v
    -cd perf && perf report -i perf-release.data

run:
    cargo run --release

sizes:
    RUSTFLAGS="-Zprint-type-sizes" cargo +nightly build -j 1 > perf/sizes.txt

verbose:
    cargo run --release -- --log-level debug --verbose
