#!/bin/bash
#
# Installation script for git hooks
# Run this script to set up pre-commit hooks for context-pilot-rs
#

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🔧 Installing git hooks for context-pilot-rs...${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${YELLOW}⚠️  Warning: Cargo.toml not found${NC}"
    echo "Please run this script from the root of the context-pilot-rs repository"
    exit 1
fi

# Check if .git directory exists
if [ ! -d ".git" ]; then
    echo -e "${YELLOW}⚠️  Warning: .git directory not found${NC}"
    echo "This doesn't appear to be a git repository"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy pre-commit hook
echo "📝 Installing pre-commit hook..."
cp .git-hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

echo -e "${GREEN}✅ Pre-commit hook installed successfully!${NC}"
echo ""
echo "The hook will:"
echo "  • Check Rust code formatting before each commit"
echo "  • Run clippy to catch common issues (with warning)"
echo "  • Prevent commits with formatting errors"
echo ""
echo "To test the hook, try making a commit."
echo "To bypass the hook (not recommended), use: ${YELLOW}git commit --no-verify${NC}"
echo ""
echo -e "${GREEN}Done!${NC}"
