# Deployment Guide: Dapr Rust Example App to AKS

This guide describes how to deploy the example Rust application to Azure Kubernetes Service (AKS) with Dapr and Azure Database for PostgreSQL as the state store.

## Prerequisites

- Azure CLI (`az`) logged in
- `kubectl` configured for your AKS cluster
- Dapr installed on the cluster (`dapr init -k`)
- Azure Database for PostgreSQL created and accessible from the cluster

## Architecture

See [architecture.md](./architecture.md) for Mermaid diagrams.

### Application and Dapr State Store

The app uses a **Dapr State Store component** (Kubernetes `Component` resource) to persist state in Azure PostgreSQL. The Dapr sidecar reads the component config and connects to PostgreSQL; the app never talks to the database directly.

### Observability (Logs, Traces, Metrics)

- **Logs**: The app uses **tracing** and writes JSON logs to stdout. Container logs are collected by Promtail, Fluent Bit, Datadog Agent, or similar, and sent to Grafana Loki or Datadog Logs. No app configuration needed.
- **Traces and metrics**: The app emits **OpenTelemetry** traces and metrics for HTTP requests when `OTEL_EXPORTER_OTLP_ENDPOINT` is set. Data flows to an OTLP-compatible backend:

| Backend                                  | OTLP endpoint example       |
| ---------------------------------------- | --------------------------- |
| **Grafana** (Alloy + Tempo + Prometheus) | `http://alloy:4318`         |
| **Datadog Agent**                        | `http://datadog-agent:4318` |

## Step 1: Create Azure Database for PostgreSQL

1. Create a resource group and PostgreSQL Flexible Server (if not exists):

   ```bash
   az postgres flexible-server create --name <server-name> --resource-group <rg> --location <location> \
     --admin-user <admin> --admin-password <password> --sku-name Standard_B1ms --tier Burstable
   ```

2. Create the database:

   ```bash
   az postgres flexible-server db create --resource-group <rg> --server-name <server-name> --database-name daprstate
   ```

3. Configure firewall to allow AKS egress IPs or use Private Endpoint for production.

## Step 2: Configure Dapr Azure PostgreSQL State Store Component

Create a Kubernetes secret with the connection string. **Keep the entire connection string on one line**:

```bash
kubectl create secret generic azurepostgres-secret --from-literal=connectionString='host=<server>.postgres.database.azure.com user=<admin> password=<password> port=5432 database=daprstate sslmode=require'
```

Apply the Dapr component:

```bash
kubectl apply -f docs/dapr-component.yaml
```

See [dapr-component.yaml](./dapr-component.yaml) for the full example.

For Azure AD authentication (recommended for production), use `useAzureAD: true` and configure the component with Managed Identity or service principal.

## Step 3: Build and Push Container Image

**Option A: Use pre-built image from GitHub Container Registry**

Images are built automatically on push to `main`. Pull the latest:

```bash
docker pull ghcr.io/miguelmartens/example-rust-dapr-otel:latest
```

**Option B: Build and push to your own registry (ACR, Docker Hub, etc.)**

```bash
docker build -f build/package/Dockerfile -t <registry>/example-rust-dapr-otel:latest .
docker push <registry>/example-rust-dapr-otel:latest
```

## Step 4: Deploy to Kubernetes

Apply the deployment manifest. See [kubernetes-deployment.yaml](./kubernetes-deployment.yaml) for the example.

The deployment includes [Kubernetes health probes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/) using `/livez` and `/readyz`:

- **Startup probe** (`/readyz`): Allows up to 60s for the app and Dapr sidecar to initialize.
- **Readiness probe** (`/readyz`): Removes the pod from Service endpoints when not ready.
- **Liveness probe** (`/livez`): Restarts the container when unrecoverable.

Update the image reference and any environment variables, then:

```bash
kubectl apply -f docs/kubernetes-deployment.yaml
```

## Step 5: Verify Deployment

```bash
kubectl get pods -l app=example-rust-dapr-otel
kubectl logs -l app=example-rust-dapr-otel -c example-rust-dapr-otel -f
```

### Port-forward (local testing)

```bash
kubectl port-forward svc/example-rust-dapr-otel 8080:80
```

Then test the API at `http://localhost:8080`:

```bash
# Save state
curl -X POST http://localhost:8080/api/v1/state/foo -d 'hello'

# Get state
curl http://localhost:8080/api/v1/state/foo

# Delete state
curl -X DELETE http://localhost:8080/api/v1/state/foo

# Health check
curl http://localhost:8080/health
```

Press `Ctrl+C` to stop the port-forward.

## Step 6: Configure Observability (Optional)

- **Logs**: The app writes JSON logs to stdout. Deploy a log collector (Promtail, Fluent Bit, Grafana Alloy, or Datadog Agent) to ship container logs to Grafana Loki or Datadog Logs.
- **Traces and metrics**: Set `OTEL_EXPORTER_OTLP_ENDPOINT` to your OTLP receiver (e.g. `http://alloy.monitoring:4318` or `http://datadog-agent.datadog:4318`).

Add the env vars to your Deployment (see [kubernetes-deployment.yaml](./kubernetes-deployment.yaml) for commented examples).

## Step 7: Deploy with ArgoCD (Optional)

```bash
kubectl apply -f docs/argocd-application.yaml
```

## Environment Variables

| Variable                      | Default                  | Description                      |
| ----------------------------- | ------------------------ | -------------------------------- |
| `APP_PORT`                    | `8080`                   | HTTP server port                 |
| `STATESTORE_NAME`             | `statestore`             | Dapr state store component name  |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | (none)                   | OTLP endpoint for traces/metrics |
| `OTEL_SERVICE_NAME`           | `example-rust-dapr-otel` | Service name for telemetry       |

## Troubleshooting

- **Dapr sidecar not starting**: Check pod annotations (`dapr.io/enabled`, `dapr.io/app-id`, `dapr.io/app-port`).
- **State operations fail**: Verify the Dapr component is applied and the Azure PostgreSQL secret exists. Check Dapr sidecar logs: `kubectl logs -c daprd`.
- **No traces/metrics**: Ensure `OTEL_EXPORTER_OTLP_ENDPOINT` is set and the OTLP receiver is reachable from the pod.
- **Connection refused to Dapr**: Ensure the app listens on the port specified in `dapr.io/app-port`.

## Cleanup

```bash
kubectl delete -f docs/kubernetes-deployment.yaml
kubectl delete -f docs/dapr-component.yaml
kubectl delete secret azurepostgres-secret
```
