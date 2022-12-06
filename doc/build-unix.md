# BUILD FOR UNIX

## Introduction

Before build, see [build requirements](#build-requirements) for your specific platform.

## Build

### CLI only

```
cargo build --release
```

### Both CLI and GUI

To build GUI you need to install additional dependencies (see [build requirements](#build-requirements)).

```
cargo build --release --features gui
```

When build is completed, you can find `keechain` binary in `target/release` folder.

## Usage

Before using `keechain`, take a look at [usage](./usage.md) guide.

## Build requirements

### Ubuntu & Debian

```
sudo apt install build-essential 
```

GUI dependencies:

```
sudo apt install build-essential libclang-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev
```

### Fedora

```
sudo dnf group install "C Development Tools and Libraries" "Development Tools"
```

GUI dependencies:

```
sudo dnf install dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel
```