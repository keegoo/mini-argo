# Mini Argo CD

A minimal GitOps controller for a home k3s cluster, built with Rust + kube-rs.

Development plan: [PLAN.md](PLAN.md)

## Layout

```text
├── config                          # manifest (namespace, RBAC, development) 
│   ├── deployment.yaml
│   ├── namespace.yaml
│   ├── role-binding.yaml
│   ├── role.yaml
│   └── service-account.yaml
└── src
    └── main.rs                     # controller code
```

```text
src/            controller code
config/         Kubernetes manifests (namespace, RBAC, deployment)
```

## Development

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```
