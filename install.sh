#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Building gogit in release mode...${NC}"
cargo build --release

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Security check: ensure the directory is not world-writable
if [ "$(stat -c %a "$INSTALL_DIR" | cut -c3)" -ge 2 ]; then
    echo -e "${YELLOW}⚠️  Security Note: $INSTALL_DIR is currently world-writable.${NC}"
fi

echo -e "${BLUE}📦 Installing gogit to ${YELLOW}$INSTALL_DIR${NC}..."
cp target/release/gogit "$INSTALL_DIR/"

# Exact shell config path detection
SHELL_CONFIG=""
if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
    SHELL_CONFIG="$HOME/.bashrc"
fi

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠️  $INSTALL_DIR is not in your PATH.${NC}"
    if [ -n "$SHELL_CONFIG" ]; then
        echo -e "Add this to your ${BLUE}$SHELL_CONFIG${NC}:"
        echo -e "${GREEN}  export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo -e "Then run: ${BLUE}source $SHELL_CONFIG${NC}"
    fi
fi

echo -e "${GREEN}✅ gogit installed successfully!${NC}"
echo -e "Try running: ${GREEN}gogit --help${NC}"
