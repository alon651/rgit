TARGET := rgit
RELEASE_BIN := target/release/$(TARGET)
DEBUG_BIN := target/debug/$(TARGET)

LOCAL_BIN := $(HOME)/.local/bin/$(TARGET)

ZSH_COMPLETION_DIR := $(HOME)/.zsh/completions
ZSH_COMPLETION_FILE := $(ZSH_COMPLETION_DIR)/_$(TARGET)

BASH_COMPLETION_DIR := $(HOME)/.local/share/bash-completion/completions
BASH_COMPLETION_FILE := $(BASH_COMPLETION_DIR)/$(TARGET)

FISH_COMPLETION_DIR := $(HOME)/.config/fish/completions
FISH_COMPLETION_FILE := $(FISH_COMPLETION_DIR)/$(TARGET).fish

.PHONY: all build release install install-dev clean help completions

all: help

help:
	@echo "Available targets:"
	@echo "  make build        - Build debug version"
	@echo "  make release      - Build release version"
	@echo "  make install-dev  - Build + symlink + completions"
	@echo "  make install      - Release install + completions"
	@echo "  make completions  - Generate zsh, bash, and fish completions"
	@echo "  make clean        - Clean build artifacts"

build:
	cargo build

release:
	cargo build --release

# Generate completions using whichever binary is already built (prefer release).
completions: build
	@BIN="$(RELEASE_BIN)"; [ -x "$$BIN" ] || BIN="$(DEBUG_BIN)"; \
	mkdir -p $(ZSH_COMPLETION_DIR) $(BASH_COMPLETION_DIR) $(FISH_COMPLETION_DIR); \
	"$$BIN" completions zsh  > $(ZSH_COMPLETION_FILE); \
	"$$BIN" completions bash > $(BASH_COMPLETION_FILE); \
	"$$BIN" completions fish > $(FISH_COMPLETION_FILE); \
	echo "Installed zsh  completions → $(ZSH_COMPLETION_FILE)"; \
	echo "Installed bash completions → $(BASH_COMPLETION_FILE)"; \
	echo "Installed fish completions → $(FISH_COMPLETION_FILE)"

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
	@rm -f $(ZSH_COMPLETION_FILE) $(BASH_COMPLETION_FILE) $(FISH_COMPLETION_FILE)
	@echo "Cleaned build artifacts, symlink, and completions"
