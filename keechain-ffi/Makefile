CURRENT_DIR := $(shell pwd)
SOURCES=$(sort $(wildcard ./src/*.rs ./src/**/*.rs))
UNAME := $(shell uname)

ifeq ($(UNAME), Linux)
COPY = cp
EXT = so
else ifeq ($(UNAME), Darwin)
COPY = cp
EXT = dylib
else
COPY = copy
EXT = dll
endif

python:
	rm -rf bindings-python/dist
	pip install -r bindings-python/requirements.txt
	cargo build --release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library ../target/release/libkeechain_ffi.$(EXT) --language python --no-format -o bindings-python/src/keechain/
	cp ../target/release/libkeechain_ffi.$(EXT) bindings-python/src/keechain/ | true
	cd bindings-python && python setup.py --verbose bdist_wheel
	pip install ./bindings-python/dist/keechain-*.whl --force-reinstall
