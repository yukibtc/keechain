CURRENT_DIR := $(shell pwd)
SOURCES=$(sort $(wildcard ./src/*.rs ./src/**/*.rs))

python:
	rm -rf bindings-python/dist
	pip install -r bindings-python/requirements.txt
	cargo build --release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library ../target/release/libkeechain_ffi.so --language python --no-format -o bindings-python/src/keechain/
	cp ../target/release/libkeechain_ffi.so bindings-python/src/keechain/ | true
	cp ../target/release/libkeechain_ffi.dylib bindings-python/src/keechain/ | true
	cd bindings-python && python setup.py --verbose bdist_wheel
	pip install ./bindings-python/dist/keechain-*.whl --force-reinstall
