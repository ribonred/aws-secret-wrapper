#!/bin/bash

set -e
# Get the directory where the script is located
SCRIPT_DIR="/app"
# Change to the action directory where Cargo.toml is located
cd "${SCRIPT_DIR}"

if [[ "$S3_CACHE_BUCKET" ]]; then
    export SCCACHE_BUCKET="$S3_CACHE_BUCKET"
    export RUSTC_WRAPPER=/usr/local/bin/sccache
    export SCCACHE_REGION="$AWS_REGION"

    echo "Using sccache with bucket: $SCCACHE_BUCKET"
fi
if [[ -z "$AWS_ACCESS_KEY_ID" || -z "$AWS_SECRET_ACCESS_KEY" || -z "$AWS_REGION" ]]; then
    echo "<aws_access_key_id> <aws_secret_access_key> <aws_region>"
    echo "Please provide all three parameters."
    exit 1
fi
echo "[ALL INPUTS ARE PROVIDED]"
echo "aws_access_key: $AWS_ACCESS_KEY_ID" > "config.yaml"
echo "aws_secret_key: $AWS_SECRET_ACCESS_KEY" >> "config.yaml"
echo "aws_region: $AWS_REGION" >> "config.yaml"

# Build the project
cargo build --release

# Copy the binary to both the workspace volume and global bin
cp "${SCRIPT_DIR}/target/release/aws-secret-wrapper" "/github/workspace/aws-secret-wrapper"
chmod +x "/github/workspace/aws-secret-wrapper"
