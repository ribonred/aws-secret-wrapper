#!/bin/bash
set -e  # Exit on error

# First build our action's Docker image
echo "Building action image..."
docker build -t aws-secrets-wrapper-action .

# Create test environment variables
export AWS_ACCESS_KEY="xx"
export AWS_SECRET_KEY="xxx"
export AWS_REGION="ap-southeast-3"
export AWS_BUCKET="some-artifact"
export GITHUB_OUTPUT=$(pwd)/github_output
touch $GITHUB_OUTPUT

# Simulate GitHub workspace environment
export GITHUB_WORKSPACE=$(pwd)/test-app
mkdir -p $GITHUB_WORKSPACE

# Run the action's container
echo "Running action container..."
docker run --rm \
  -e GITHUB_WORKSPACE=$GITHUB_WORKSPACE \
  -e GITHUB_OUTPUT=$GITHUB_OUTPUT \
  -e AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY \
  -e AWS_SECRET_ACCESS_KEY=$AWS_SECRET_KEY \
  -e AWS_REGION=$AWS_REGION \
  -e S3_CACHE_BUCKET=$AWS_BUCKET \
  -v $GITHUB_WORKSPACE:/github/workspace \
  -v $GITHUB_OUTPUT:$GITHUB_OUTPUT \
  aws-secrets-wrapper-action

# Now test using the binary in a sample app build
echo "Testing binary usage in app build..."
cat > $GITHUB_WORKSPACE/Dockerfile <<EOF
FROM python:3.9-slim
COPY . .
ENTRYPOINT ["./aws-secret-wrapper", "--secret-id", "wowo", "--"]
CMD ["python", "app.py"]
EOF

cat > $GITHUB_WORKSPACE/app.py <<EOF
import os
print("Environment variables:")
for key, value in os.environ.items():
    print(f"{key}={value}")
EOF

# Build the test application
echo "Building test application..."
docker build -t test-app $GITHUB_WORKSPACE
docker run --rm test-app
echo "Test setup complete! You can now run: docker run test-app"

# Cleanup
echo "Cleaning up test files..."
rm -f $GITHUB_OUTPUT
rm -rf $GITHUB_WORKSPACE