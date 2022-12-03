# Use 'DEBUG=1' to build debug binary'.
ifdef DEBUG
  RELEASE := 
else
  RELEASE := --release
endif

# Use 'VERBOSE=1' to echo all commands, for example 'make help VERBOSE=1'.
ifdef VERBOSE
  Q :=
else
  Q := @
endif

all: build

help:
	$(Q)echo ""
	$(Q)echo "make build             - Build binary files"
	$(Q)echo "make precommit         - Execute precommit steps"
	$(Q)echo "make clean         	 - Clean"
	$(Q)echo "make loc               - Count lines of code in src folder"
	$(Q)echo ""

build:
	$(Q)cargo build $(RELEASE)

precommit:
	$(Q)cargo fmt && cargo clippy

clean:
	$(Q)cargo clean

loc:
	$(Q)echo "--- Counting lines of .rs files (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l
