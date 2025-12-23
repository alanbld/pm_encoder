#!/bin/bash
set -euo pipefail

echo "========================================"
echo "LSP vs Regex Experiment - Phase 3"
echo "========================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check for rust-analyzer
echo "Checking for rust-analyzer..."
if ! command -v rust-analyzer &> /dev/null; then
    echo -e "${YELLOW}rust-analyzer not found. Attempting to install...${NC}"
    if command -v rustup &> /dev/null; then
        rustup component add rust-analyzer
    else
        echo -e "${RED}ERROR: rustup not found. Please install rust-analyzer manually.${NC}"
        exit 1
    fi
fi

RUST_ANALYZER_VERSION=$(rust-analyzer --version 2>/dev/null || echo "unknown")
echo -e "${GREEN}Found: $RUST_ANALYZER_VERSION${NC}"
echo ""

# Build the comparison tool
echo "Building comparison tool (release mode)..."
cargo build --release --bin compare 2>&1 | tail -5

if [ ! -f "target/release/compare" ]; then
    echo -e "${RED}ERROR: Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}Build successful!${NC}"
echo ""

# Determine source directory
SRC_DIR="${1:-../../rust/src}"
if [ ! -d "$SRC_DIR" ]; then
    # Try alternative paths
    for alt in "../rust/src" "./rust/src" "./src"; do
        if [ -d "$alt" ]; then
            SRC_DIR="$alt"
            break
        fi
    done
fi

if [ ! -d "$SRC_DIR" ]; then
    echo -e "${RED}ERROR: Source directory not found: $SRC_DIR${NC}"
    echo "Usage: $0 [SOURCE_DIR]"
    exit 1
fi

echo "Source directory: $(realpath "$SRC_DIR")"
echo "Rust files: $(find "$SRC_DIR" -name "*.rs" | wc -l)"
echo ""

# Run the experiment
echo "========================================"
echo "Running experiment..."
echo "========================================"
echo ""

./target/release/compare "$SRC_DIR"

# Check outputs
echo ""
echo "========================================"
echo "Output Files"
echo "========================================"

if [ -f "comparison_results.csv" ]; then
    ROWS=$(wc -l < comparison_results.csv)
    echo -e "${GREEN}comparison_results.csv: $ROWS rows${NC}"
else
    echo -e "${RED}comparison_results.csv: NOT FOUND${NC}"
fi

if [ -f "experiment_summary.txt" ]; then
    echo -e "${GREEN}experiment_summary.txt: $(wc -l < experiment_summary.txt) lines${NC}"
    echo ""
    echo "Summary contents:"
    echo "----------------------------------------"
    cat experiment_summary.txt
    echo "----------------------------------------"
else
    echo -e "${RED}experiment_summary.txt: NOT FOUND${NC}"
fi

echo ""
echo -e "${GREEN}Experiment complete!${NC}"
