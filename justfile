# Set default log level if not provided through environment
export LOG_LEVEL := env_var_or_default("RUST_LOG", "info")

default: test

cov:
	cargo llvm-cov --lcov

cov-html:
	cargo llvm-cov --html

cov-open:
	cargo llvm-cov --open

build-release:
	cargo build --workspace --release

# For log level use RUST_LOG=<<level>> just run
run:
	RUST_LOG={{LOG_LEVEL}} cargo run

check:
	cargo check

fmt:
	cargo fmt --all

test:
	cargo test
