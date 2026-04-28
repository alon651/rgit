TARGET := rgit
RELEASE_BIN := target/release/$(TARGET)
DEBUG_BIN := target/debug/$(TARGET)

LOCAL_BIN := $(HOME)/.local/bin/$(TARGET)
COMPLETION_DIR := $(HOME)/.zsh/completions
COMPLETION_FILE := $(COMPLETION_DIR)/_$(TARGET)

.PHONY: all build release install install-dev clean help completions

all: help

help:
	@echo "Available targets:"
	@echo "  make build        - Build debug version"
	@echo "  make release      - Build release version"
	@echo "  make install-dev  - Build + symlink + completions"
	@echo "  make install      - Release install + completions"
	@echo "  make completions  - Generate zsh completions"
	@echo "  make clean        - Clean build artifacts"

build:
	cargo build

release:
	cargo build --release

completions:
	@mkdir -p $(COMPLETION_DIR)
	cargo run -- completions zsh > $(COMPLETION_FILE)
	@echo "Installed completions → $(COMPLETION_FILE)"

install-dev: build completions
	@mkdir -p $(HOME)/.local/bin
	@ln -sf $(shell realpath $(DEBUG_BIN)) $(LOCAL_BIN)
	@echo "Installed (dev) → $(LOCAL_BIN)"

install: release completions
	sudo install -m 755 $(RELEASE_BIN) /usr/local/bin/$(TARGET)
	@echo "Installed → /usr/local/bin/$(TARGET)"

clean:
	cargo clean
	@rm -f $(LOCAL_BIN)
	@rm -f $(COMPLETION_FILE)
	@echo "Cleaned build artifacts, symlink, and completions"