
project := "worker-lib"

alias b := build
alias t := test
alias ta := test-all

# run the standard tests
test:
    clear
    cargo test

# run the standard tests + clippy and fmt
test-all:
    clear
    cargo test && cargo fmt && cargo clippy

# build the debug target
build:
    clear
    cargo build

# build the docs
docs:
    cargo doc --no-deps --open

# cover - runs code test coverage report and writes to coverage folder
cover:
  cargo tarpaulin --out html --output-dir coverage && scp coverage/tarpaulin-report.html dpw@raincitysoftware.com:raincitysoftware.com/key-mesh/index.html

# start a http server in the coverage folder
serve-cover:
  cd coverage && mv tarpaulin-report.html index.html && python3 -m http.server 8080

