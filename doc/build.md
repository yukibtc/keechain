# BUILD

## Download source code

```
git clone https://github.com/yukibtc/keechain.git && cd keechain
```

## Verify commits

Import gpg keys:

```
gpg --keyserver hkps://keys.openpgp.org --recv-keys $(<contrib/verify-commits/trusted-keys)
```

Verify commit:

```
git verify-commit HEAD
```

## Install Rust

Follow this instructions: https://www.rust-lang.org/tools/install

## Build

```
cargo build --release
```

When build is completed, you can find `keechain` binary in `target/release` folder.

## Usage

Before using `keechain`, take a look at [usage](doc/usage.md) guide.