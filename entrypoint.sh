#!/bin/bash

set -e

echo "aws_access_key: \"$1\"" > "config.yaml"
echo "aws_secret_key: \"$2\"" >> "config.yaml"
echo "aws_region: \"$3\"" >> "config.yaml"
cargo build --release && cp target/release/aws-secret-wrapper "${GITHUB_WORKSPACE}/aws-secret-wrapper" && \
cp target/release/aws-secret-wrapper /usr/local/bin/
chmod +x "${GITHUB_WORKSPACE}/aws-secret-wrapper"

# Set the output for other steps to use
echo "binary=${GITHUB_WORKSPACE}/aws-secret-wrapper" >> "${GITHUB_OUTPUT}" || {
    echo "Warning: Could not write to GITHUB_OUTPUT at ${GITHUB_OUTPUT}"
    exit 1
}
