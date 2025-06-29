# AWS Secrets Manager Wrapper

<p align="center">
  <img src="bg.png" alt="AWS Secrets Manager Wrapper Logo" width="200">
</p>

A command-line tool that injects AWS Secrets Manager secrets as environment variables for your applications.

## 🚀 Features

- Fetches secrets from AWS Secrets Manager
- Injects secrets as environment variables
- Supports JSON-formatted secrets
- Clean environment isolation

## 📋 Prerequisites

- Rust toolchain (stable)
- AWS credentials
- AWS Secrets Manager access

## 🔧 Setup

1. **Clone and Build**
   ```bash
   git clone [repository-url]
   cd aws-secret-wrapper
   cargo build --release
   ```

2. **Configure AWS Credentials**
   
   Create a `config.yaml` file:
   ```yaml
   aws_access_key: "YOUR_AWS_ACCESS_KEY"
   aws_secret_key: "YOUR_AWS_SECRET_KEY"
   aws_region: "us-east-1"
   ```

   or copy the example file:
   ```bash
   cp config.example.yaml config.yaml
   ```

## 📖 Usage

Basic syntax:
```bash
aws-secret-wrapper --secret-id <SECRET_ID> -- <COMMAND> [ARGS...]
```

support multiple secret ids with comma separated values:
```bash
aws-secret-wrapper --secret-id <SECRET_ID1>,<SECRET_ID2> -- <COMMAND> [ARGS...]
```

support change region:
```bash
aws-secret-wrapper --secret-id <SECRET_ID> --region <REGION> -- <COMMAND> [ARGS...]
```

support file read secret id:

assume your file have name `secretIds`

and the content like this. it will fetch line by line and injected to env

```bash
my-secret-id-1
```

or

```bash
my-secret-id-1
my-secret-id-2
```


```bash
aws-secret-wrapper --sf <FILENAME> -- <COMMAND> [ARGS...]
```

### Examples

1. **Run a Node.js app**
   ```bash
   ./target/release/aws-secret-wrapper --secret-id dev/myapp/secrets -- node app.js
   ```

2. **Run with arguments**
   ```bash
   ./target/release/aws-secret-wrapper --secret-id dev/myapp/secrets -- npm start --port 3000
   ```

3. **Run Python script**
   ```bash
   ./target/release/aws-secret-wrapper --secret-id dev/myapp/secrets -- python script.py arg1 arg2
   ```
4. **Run with linux runtime**
   ```bash
   ./target/release/aws-secret-wrapper --secret-id <SECRET_ID> -- printenv | grep YOUR_SECRET_KEY
   ```
5. **Run with linux runtime and secret id file**
   ```bash
   ./target/release/aws-secret-wrapper --sf <FILENAME> -- printenv | grep YOUR_SECRET_KEY
   ```

### Secret Format

Your AWS Secrets Manager secret should be in JSON format:
```json
{
    "DB_PASSWORD": "mysecret123",
    "API_KEY": "abc123xyz",
    "DATABASE_URL": "postgresql://user:pass@localhost:5432/db"
}
```

## ⚠️ Important Notes

- The `--` separator is required
- Everything after `--` is treated as the command to run
- Never commit `config.yaml` to version control
- Secrets must be valid JSON objects

## 🔍 Error Codes

- Returns the wrapped command's exit code on success
- Returns 1 if:
  - Secret retrieval fails
  - Secret is not valid JSON
  - Command execution fails

## 🔒 Security

- Store AWS credentials securely
- Use appropriate IAM permissions
- Keep `config.yaml` private
- Use environment-specific secrets

## 🛠️ Development

```bash
# Build debug version
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Check for errors
cargo check
```

## 🐳 Using in Docker Builds

There are two main ways to use this tool in Docker:

### 1. Using as a GitHub Action (Recommended for CI/CD)

When using this tool as a GitHub Action, the binary will be automatically copied to your workspace. This means when you do `COPY . .` in your Dockerfile, the `aws-secret-wrapper` binary will already be in your build context.

Example workflow and Dockerfile usage:
## ⚠️ Important Notes

- is not going to work github restriction if using `secrets.AWS_ACCESS_KEY` should use `env.XXX` by this github is masking the value and safe to pass into action

```yaml
# Your GitHub workflow
steps:
  - uses: actions/checkout@v1.0.5
  - uses: ribonred/aws-secret-wrapper@v1.0.7
    with:
      aws_access_key: ${{ env.AWS_ACCESS_KEY }}
      aws_secret_key: ${{ env.AWS_SECRET_KEY }}
      aws_region: 'us-east-1'
   - name: Build and push Docker image
       run: |
          docker build -t myapp .
          docker push myapp
```

assuming your dockerfile is like this:

```dockerfile
# Your application's Dockerfile
FROM python:3.9-slim
# Copy your application code including the aws-secret-wrapper binary
COPY . .
# The binary will be available in your application directory
# you can do as follow
ENTRYPOINT ["./aws-secret-wrapper", "--secret-id", "your-secret-id", "--"]
CMD ["python", "app.py"]
```

## 📜 License

MIT