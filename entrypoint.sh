#!/bin/bash

set -e

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Change to the action directory where Cargo.toml is located
cd "${SCRIPT_DIR}"

echo "aws_access_key: \"$1\"" > "config.yaml"
echo "aws_secret_key: \"$2\"" >> "config.yaml"
echo "aws_region: \"$3\"" >> "config.yaml"

# Build the project
cargo build --release

# Copy the binary to both the workspace volume and global bin
cp "${SCRIPT_DIR}/target/release/aws-secret-wrapper" "/github/workspace/aws-secret-wrapper"
chmod +x "/github/workspace/aws-secret-wrapper"
