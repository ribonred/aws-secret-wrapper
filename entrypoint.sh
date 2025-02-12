#!/bin/bash

set -e

echo "aws_access_key: \"$1\"" > "${GITHUB_WORKSPACE}/config.yaml"
echo "aws_secret_key: \"$2\"" >> "${GITHUB_WORKSPACE}/config.yaml"
echo "aws_region: \"$3\"" >> "${GITHUB_WORKSPACE}/config.yaml"

# Copy the binary to the GitHub workspace
cp /usr/local/bin/aws-secret-wrapper "${GITHUB_WORKSPACE}/aws-secret-wrapper"
chmod +x "${GITHUB_WORKSPACE}/aws-secret-wrapper"

# Set the output for other steps to use
echo "binary=${GITHUB_WORKSPACE}/aws-secret-wrapper" >> "${GITHUB_OUTPUT}" || {
    echo "Warning: Could not write to GITHUB_OUTPUT at ${GITHUB_OUTPUT}"
    exit 1
}

# set the builder image
echo "docker_target=builder" >> "${GITHUB_OUTPUT}" || {
    echo "Warning: Could not write to GITHUB_OUTPUT at ${GITHUB_OUTPUT}"
    exit 1
}