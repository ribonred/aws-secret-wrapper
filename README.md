# AWS Secrets Manager Wrapper

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
   cd s3-secret-wrapper
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
s3-secret-wrapper --secret-id <SECRET_ID> -- <COMMAND> [ARGS...]
```

support multiple secret ids with comma separated values:
```bash
s3-secret-wrapper --secret-id <SECRET_ID1>,<SECRET_ID2> -- <COMMAND> [ARGS...]
```

support change region:
```bash
s3-secret-wrapper --secret-id <SECRET_ID> --region <REGION> -- <COMMAND> [ARGS...]
```

### Examples

1. **Run a Node.js app**
   ```bash
   ./target/release/s3-secret-wrapper --secret-id dev/myapp/secrets -- node app.js
   ```

2. **Run with arguments**
   ```bash
   ./target/release/s3-secret-wrapper --secret-id dev/myapp/secrets -- npm start --port 3000
   ```

3. **Run Python script**
   ```bash
   ./target/release/s3-secret-wrapper --secret-id dev/myapp/secrets -- python script.py arg1 arg2
   ```
4. **Run with linux runtime**
   ```bash
   ./target/release/s3-secret-wrapper --secret-id <SECRET_ID> -- printenv | grep YOUR_SECRET_KEY
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

## 📜 License

MIT