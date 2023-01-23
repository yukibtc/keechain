# Use 'VERBOSE=1' to echo all commands, for example 'make help VERBOSE=1'.
ifdef VERBOSE
  Q :=
else
  Q := @
endif

all: gui cli

help:
	$(Q)echo ""
	$(Q)echo "make                                 - Build binaries files"
	$(Q)echo "make gui                             - Build only GUI binary files"
	$(Q)echo "make cli                             - Build only CLI binary files"
	$(Q)echo "make appimage                        - Build Linux AppImage (GUI only)"
	$(Q)echo "make x86_64-unknown-linux-gnu        - Build target x86_64-unknown-linux-gnu"
	$(Q)echo "make precommit                       - Execute precommit steps"
	$(Q)echo "make clean                           - Clean"
	$(Q)echo "make loc                             - Count lines of code in src folder"
	$(Q)echo ""

gui:
	$(Q)cargo build -p keechain --release --all-features

cli:
	$(Q)cargo build -p keechain-cli --release --all-features

appimage: x86_64-unknown-linux-gnu
	$(Q)cd ./contrib/cross/appimage/ && docker build -t keechain/appimage-builder:latest -f Dockerfile.x86_64-unknown-linux-gnu .
	$(Q)docker run -v $$(PWD)/target:/target:ro -v $$(PWD)/output:/output keechain/appimage-builder

x86_64-unknown-linux-gnu: cross
	$(Q)cross build --release --all-features --target x86_64-unknown-linux-gnu

x86_64-unknown-linux-musl:
	$(Q)rustup target add x86_64-unknown-linux-musl
	$(Q)TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl

cross:
	$(Q)cargo install cross --version 0.2.4

precommit: test
	$(Q)cargo fmt --all
	$(Q)cargo clippy --all
	$(Q)cargo clippy --all --all-features

test:
	$(Q)cargo test -p keechain-core

clean:
	$(Q)cargo clean

loc:
	$(Q)echo "--- Counting lines of .rs files (LOC):" && find keechain*/ -type f -name "*.rs" -exec cat {} \; | wc -l
