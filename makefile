TARGET := rgit
RELEASE_BIN := target/release/$(TARGET)
DEBUG_BIN := target/debug/$(TARGET)
LOCAL_BIN := $(HOME)/.local/bin/$(TARGET)

.PHONY: all build release install install-dev clean help

all: help

help: ## Show available commands
	@echo "Available targets:"
	@echo "  make build        - Build debug version"
	@echo "  make release      - Build release version"
	@echo "  make install-dev   - Build debug + symlink to ~/.local/bin"
	@echo "  make install       - Build release + install to /usr/local/bin"
	@echo "  make clean        - Clean build artifacts"

# Debug build (Cargo decides what needs rebuilding)
build:
	cargo build

# Release build
release:
	cargo build --release

# Development install (symlink)
install-dev: build
	@mkdir -p $(HOME)/.local/bin
	@ln -sf $(shell realpath $(DEBUG_BIN)) $(LOCAL_BIN)
	@echo "Installed (dev) → $(LOCAL_BIN)"

# System install (release binary)
install: release
	sudo install -m 755 $(RELEASE_BIN) /usr/local/bin/$(TARGET)
	@echo "Installed → /usr/local/bin/$(TARGET)"

# Clean everything
clean:
	cargo clean
	@rm -f $(LOCAL_BIN)
	@echo "Cleaned build artifacts and dev symlink"
