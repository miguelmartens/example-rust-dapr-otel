# Example Rust App with Dapr and OpenTelemetry

A minimal Rust application demonstrating Dapr SDK usage with Azure Database for PostgreSQL as a state store, and OpenTelemetry for traces and metrics. When deployed to AKS with Dapr, the app uses the Dapr sidecar to persist state in Azure PostgreSQL and emits OTLP telemetry when configured.

## Requirements

- Rust 1.93+
- Dapr CLI (optional; only needed for local dev with Dapr sidecar)

## Project Layout

```
.
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Environment and configuration
│   ├── server/           # HTTP handlers and Dapr state logic
│   │   ├── mod.rs
│   │   └── memstore.rs
│   └── telemetry.rs      # OpenTelemetry trace/metric init
├── build/package/        # Dockerfile
├── docs/                 # Deployment examples (Dapr, Kubernetes, ArgoCD)
├── .env.example          # Env template; copy to .env for local config
├── Makefile
└── Cargo.toml
```

## Quick Start

### Local (without Dapr)

The app runs locally without Dapr by default. When the Dapr sidecar is unavailable, it automatically falls back to an in-memory state store. No Docker or Dapr setup required.

For optional local config (e.g. OpenTelemetry to a local collector), copy `.env.example` to `.env` and adjust:

```bash
cp .env.example .env
# Edit .env to set OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 etc.
make dev
```

Then test the API:

```bash
curl -X POST http://localhost:8080/api/v1/state/foo -d 'hello'
curl http://localhost:8080/api/v1/state/foo
curl -X DELETE http://localhost:8080/api/v1/state/foo
curl http://localhost:8080/health
```

### Local (with Dapr)

For local development with the Dapr sidecar (e.g. to test against Redis or another state store):

1. Start Dapr with a state store (e.g., Redis):

   ```bash
   dapr run --app-id example-rust-dapr-otel --app-port 8080 -- cargo run
   ```

2. Or run the built binary:

   ```bash
   make run
   ```

3. Test the API (same curl commands as above).

### Build

```bash
make build
./target/release/example-rust-dapr-otel
```

### Docker

```bash
docker build -f build/package/Dockerfile -t example-rust-dapr-otel .
docker run -p 8080:8080 example-rust-dapr-otel
```

Pre-built images are available from [GitHub Container Registry](https://github.com/miguelmartens/example-rust-dapr-otel/pkgs/container/example-rust-dapr-otel) (after the first CI run):

```bash
docker pull ghcr.io/miguelmartens/example-rust-dapr-otel:latest
```

## API

| Method | Path | Description |
| ------ | ---- | ----------- |
| GET | `/livez` | Liveness probe (should process be restarted?) |
| GET | `/readyz` | Readiness probe (ready to accept traffic?) |
| GET | `/health` | Alias for `/readyz` (backwards compatibility) |
| GET | `/api/v1/state/{key}` | Retrieve state value |
| POST | `/api/v1/state/{key}` | Save state (body = value) |
| DELETE | `/api/v1/state/{key}` | Delete state |

## Configuration

| Environment | Default | Description |
| ----------- | ------- | ----------- |
| `APP_PORT` | `8080` | HTTP server port |
| `STATESTORE_NAME` | `statestore` | Dapr state store component name |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | (none) | OTLP endpoint for traces/metrics (e.g. `http://localhost:4318`) |
| `OTEL_SERVICE_NAME` | `example-rust-dapr-otel` | Service name for telemetry |

**Local dev**: The app loads `.env` from the working directory if present. Copy `.env.example` to `.env` and customize. `.env` is gitignored.

## Observability

- **Logs**: The app uses tracing and writes JSON logs to stdout. Container logs are collected by Promtail, Fluent Bit, Datadog Agent, or similar, and sent to Grafana Loki or Datadog Logs. No app configuration needed.
- **Traces**: HTTP request spans with method, path, status code (when `OTEL_EXPORTER_OTLP_ENDPOINT` is set)
- **Metrics**: Request count and duration from tower-http TraceLayer (when `OTEL_EXPORTER_OTLP_ENDPOINT` is set)

Compatible with Grafana (Loki + OTLP/Tempo), Datadog (Logs + OTLP), and other OTLP backends. When the OTLP endpoint is not set, traces and metrics are disabled (no-op).

## Deployment to AKS

See [docs/deployment.md](docs/deployment.md) for step-by-step instructions to deploy to AKS with Azure PostgreSQL. Architecture diagrams: [docs/architecture.md](docs/architecture.md). Example manifests:

- [docs/dapr-component.yaml](docs/dapr-component.yaml) – Azure PostgreSQL state store component
- [docs/kubernetes-deployment.yaml](docs/kubernetes-deployment.yaml) – Deployment and Service
- [docs/argocd-application.yaml](docs/argocd-application.yaml) – ArgoCD Application

## Development

```bash
make dev      # Clean, build, and run (local dev without Dapr)
make build    # Build binary
make run      # Build and run
make test     # Run tests
make lint     # Run clippy and fmt --check
make fmt      # Format code
make tidy     # Update dependencies
make deps     # List dependency tree and outdated deps
make clean    # Remove build artifacts
```

**Local dev without Dapr**: `make dev` runs the app with an in-memory state store when Dapr is unavailable. No Docker or Dapr required.

## Automated Dependency Management

This project uses [Renovate](https://github.com/apps/renovate) for automated dependency updates:

- Automatic PRs for Cargo dependencies and GitHub Actions
- Setup:
  1. Install the [Renovate GitHub App](https://github.com/apps/renovate) on the repository
  2. Merge the onboarding PR Renovate creates
  3. Check the Dependency Dashboard issue for available updates

See [renovate.json](renovate.json) for configuration.

## Contributing

See CONTRIBUTING.md for development setup and pull request guidelines.

## License

[MIT](LICENSE)

## Tooling

- **clippy**: Linting
- **rustfmt**: Code formatting
- **Prettier**: JSON, YAML, Markdown (see Makefile)
- **Renovate**: Dependency updates (see `renovate.json`)
