# Renovate Setup Guide

Automated dependency management for this project.

## Quick Start

1. Go to [Renovate GitHub App](https://github.com/apps/renovate)
2. Click **Configure**
3. Select this repository (or your fork)
4. Grant the requested permissions
5. Click **Install**
6. Merge the onboarding PR Renovate creates
7. Check the Dependency Dashboard issue for available updates

## What Renovate Does

- Opens PRs for outdated **Cargo dependencies** (`Cargo.toml` / `Cargo.lock`)
- Opens PRs for outdated **GitHub Actions** in `.github/workflows/`
- Opens PRs for outdated **Docker base images** in `build/package/Dockerfile`
- Labels PRs with `dependencies`

## Configuration

The [renovate.json](../renovate.json) at the repo root configures:

| Feature        | Setting              |
| -------------- | -------------------- |
| Base preset    | `config:recommended` |
| Cargo crates   | Enabled              |
| Dockerfile     | Base images updated  |

## Local Dry-Run

To preview available Cargo dependency updates without opening PRs:

```bash
make deps
```

## Resources

- [Renovate Docs](https://docs.renovatebot.com/)
- [Config Validator](https://app.renovatebot.com/config-validator)
