# Architecture

## Application and Dapr State Store

The app uses a **Dapr State Store component** (Kubernetes `Component` resource) to persist state in Azure PostgreSQL. The Dapr sidecar reads the component config and connects to PostgreSQL; the app never talks to the database directly.

```mermaid
flowchart TB
    subgraph Component [Dapr Component - statestore]
        Comp["kind: Component<br/>spec.type: state.postgresql<br/>metadata: connectionString, etc."]
    end

    subgraph Pod [Pod: example-rust-dapr-otel]
        App["Rust HTTP Server<br/>:8080"]
        Sidecar["Dapr Sidecar (daprd)<br/>:50001"]
    end

    DB[(Azure PostgreSQL<br/>state store backend)]

    Comp -->|configures| Sidecar
    App -->|gRPC| Sidecar
    Sidecar -->|state.postgresql| DB
```

## Observability (Logs, Traces, Metrics)

### Logs

The app uses **tracing** and writes JSON logs to stdout. Container logs are collected by a log collector (Promtail, Fluent Bit, Datadog Agent, etc.) and sent to Grafana Loki or Datadog Logs.

```mermaid
flowchart LR
    subgraph AppPod [Pod: example-rust-dapr-otel]
        App["Rust HTTP Server"]
        Tracing["tracing (JSON)"]
    end

    subgraph Collectors [Log Collectors]
        Promtail["Promtail / Fluent Bit"]
        DDAgent["Datadog Agent"]
    end

    subgraph LogBackends [Log Backends]
        Loki["Grafana Loki"]
        DDLogs["Datadog Logs"]
    end

    App --> Tracing
    Tracing -->|stdout| Promtail
    Tracing -->|stdout| DDAgent
    Promtail --> Loki
    DDAgent --> DDLogs
```

### Traces and Metrics (OpenTelemetry)

The app emits **OpenTelemetry** traces and metrics for HTTP requests when `OTEL_EXPORTER_OTLP_ENDPOINT` is set. Data flows to an OTLP-compatible backend (Grafana or Datadog).

```mermaid
flowchart LR
    subgraph AppPod [Pod: example-rust-dapr-otel]
        HTTP["Rust HTTP Server"]
        OTel["tower-http TraceLayer"]
    end

    subgraph OTLPBackends [OTLP Backends]
        Grafana["Grafana<br/>Alloy + Tempo + Prometheus"]
        Datadog["Datadog Agent"]
    end

    HTTP --> OTel
    OTel -->|OTLP traces + metrics| Grafana
    OTel -->|OTLP traces + metrics| Datadog
```

## End-to-End Flow

```mermaid
flowchart TB
    subgraph AKS [AKS Cluster]
        subgraph AppPod [Pod: example-rust-dapr-otel]
            App["Rust HTTP Server :8080"]
            Sidecar["Dapr Sidecar :50001"]
        end

        Component["Dapr Component<br/>statestore"]
    end

    AzurePostgres[(Azure PostgreSQL)]
    OTLP["OTLP Collector<br/>Grafana / Datadog"]
    Logs["Log Collector<br/>Loki / Datadog"]

    Client["HTTP Client"] --> App
    App -->|gRPC| Sidecar
    Component -->|configures| Sidecar
    Sidecar -->|state.postgresql| AzurePostgres
    App -->|traces + metrics| OTLP
    App -->|logs stdout| Logs
```
