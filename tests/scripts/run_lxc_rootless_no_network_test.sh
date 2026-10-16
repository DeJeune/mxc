#!/bin/bash
# LXC rootless test: a config with no network section runs without root
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
LXC_EXEC="$REPO_DIR/src/target/release/lxc-exec"

if [ ! -f "$LXC_EXEC" ]; then
    LXC_EXEC="$REPO_DIR/src/target/debug/lxc-exec"
fi

if [ ! -f "$LXC_EXEC" ]; then
    echo "Error: lxc-exec not found. Run build.sh first."
    exit 1
fi

if [ -z "${SUDO_USER:-}" ]; then
    echo "Error: run this under sudo, so the account that invoked sudo can run lxc-exec without root."
    exit 1
fi

echo "Running LXC rootless no-network test as $SUDO_USER..."
OUTPUT=$(sudo -u "$SUDO_USER" -H "$LXC_EXEC" "$REPO_DIR/tests/configs/lxc_rootless_no_network.json" 2>&1) || {
    echo "FAIL: lxc-exec without root could not run a config that has no network section."
    echo "$OUTPUT"
    exit 1
}

if ! grep -q "MXC_ROOTLESS_OK" <<< "$OUTPUT"; then
    echo "FAIL: the container ran without root but the command printed nothing."
    echo "$OUTPUT"
    exit 1
fi

echo "LXC rootless no-network test complete."