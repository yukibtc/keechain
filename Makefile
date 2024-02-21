all: gui cli

help:
	@echo ""
	@echo "make                                 - Build binaries files"
	@echo "make gui                             - Build only GUI binary files"
	@echo "make cli                             - Build only CLI binary files"
	@echo "make appimage                        - Build Linux AppImage (GUI only)"
	@echo "make x86_64-unknown-linux-gnu        - Build target x86_64-unknown-linux-gnu"
	@echo "make precommit                       - Execute precommit steps"
	@echo "make clean                           - Clean"
	@echo "make loc                             - Count lines of code in src folder"
	@echo ""

gui:
	cargo build -p keechain --release

cli:
	cargo build -p keechain-cli --release

dev-gui:
	cargo run -p keechain

appimage: x86_64-unknown-linux-gnu
	cd ./contrib/cross/appimage/ && docker build -t keechain/appimage-builder:latest -f Dockerfile.x86_64-unknown-linux-gnu .
	docker run -v $$(PWD)/target:/target:ro -v $$(PWD)/output:/output keechain/appimage-builder

x86_64-unknown-linux-gnu: cross
	cross build --release --target x86_64-unknown-linux-gnu

x86_64-unknown-linux-musl:
	rustup target add x86_64-unknown-linux-musl
	TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl

cross:
	cargo install cross --version 0.2.4

precommit: test
	@cargo fmt --all -- --config format_code_in_doc_comments=true
	cargo clippy --all
	cargo clippy -p keechain-core --target wasm32-unknown-unknown

test:
	cargo test -p keechain-core

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find keechain*/ -type f -name "*.rs" -exec cat {} \; | wc -l
