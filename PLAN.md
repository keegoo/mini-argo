# Mini Argo CD 0.1 — Rust / kube-rs 开发计划

## 1. 项目目标

构建一个运行在家庭 k3s 集群中的、极简的 GitOps Kubernetes Controller。

项目的主要目的不是生产替代 Argo CD，而是通过自己实现一个最小版本，理解：

* Kubernetes Controller
* Reconciliation Loop
* CRD
* Kubernetes Watch
* Desired State / Actual State
* OwnerReference
* Status
* GitOps
* Git polling
* Kubernetes API
* GitHub Actions → GHCR → GitOps → Kubernetes 的完整链路

最终形成：

```text
Application Source Repo
        |
        | GitHub Actions
        v
   Build / Test
        |
        v
      GHCR
        |
        | update image version
        v
     GitOps Repo
        |
        | Git polling
        v
 Rust GitOps Controller
      (kube-rs)
        |
        | Kubernetes API
        v
      Home k3s
```

---

# 2. 核心设计原则

## 2.1 Kubernetes-native

Controller 必须通过 Kubernetes API 操作资源。

禁止：

```text
Rust Controller
      |
      └── shell → kubectl apply
```

应该：

```text
Rust Controller
      |
      └── kube-rs
             |
             └── Kubernetes API
```

---

## 2.2 Reconciliation First

Controller 的核心逻辑必须始终遵循：

```text
Desired State
      ↓
Observe Actual State
      ↓
Compare
      ↓
Reconcile
      ↓
Observe Again
```

Controller 不是：

```text
收到事件
  ↓
执行一次部署
  ↓
结束
```

而是：

```text
持续保证：

Actual State == Desired State
```

---

## 2.3 GitHub 不应该访问家庭网络

GitHub Actions 不需要访问：

```text
192.168.x.x
```

也不需要暴露：

```text
Kubernetes API :443
```

家庭 Kubernetes 主动访问 GitHub：

```text
Home Controller
       |
       | HTTPS outbound
       v
GitHub
```

因此整个架构不需要公网暴露 Kubernetes API。

---

## 2.4 第一版保持极简

v0.1 不做：

* UI
* Helm
* Kustomize
* Multi-cluster
* Multi-user
* ApplicationSet
* Webhook
* Automatic rollback
* Canary deployment
* Progressive delivery
* Notification system
* GitHub App integration
* Argo CD API compatibility
* Production-grade secret management
* Complex dependency management

先把核心 reconciliation loop 做正确。

---

# 3. 技术栈

## Controller

```text
Rust
Cargo
Tokio
kube
kube-runtime
kube-derive
k8s-openapi
serde
serde_yaml
schemars
tracing
tracing-subscriber
```

推荐使用：

```text
Rust stable
kube-rs latest compatible stable release
```

版本应该在项目初始化时锁定，并提交：

```text
Cargo.lock
```

---

## Kubernetes

```text
k3s
```

Controller 运行在家庭 k3s 集群内部。

---

## Container Registry

```text
GitHub Container Registry (GHCR)
```

---

## Source / GitOps

```text
GitHub
```

两个 repository：

```text
family-menu
family-menu-gitops
```

其中：

```text
family-menu-gitops
```

为 Private Repository。

---

## CI

```text
GitHub Actions
```

---

# 4. Repository Architecture

建议创建三个 repository。

## 4.1 Controller Repository

```text
mini-argocd-controller/
```

结构：

```text
mini-argocd-controller/
├── src/
│   ├── main.rs
│   ├── controller.rs
│   ├── git.rs
│   ├── resources.rs
│   └── error.rs
│
├── crd/
│   └── homeapp.yaml
│
├── config/
│   ├── deployment.yaml
│   ├── service-account.yaml
│   ├── role.yaml
│   ├── role-binding.yaml
│   └── namespace.yaml
│
├── examples/
│   └── homeapp.yaml
│
├── tests/
│
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── .github/
│   └── workflows/
│       └── build.yml
└── README.md
```

---

## 4.2 Application Repository

例如：

```text
family-menu/
```

结构：

```text
family-menu/
├── client/
├── server/
├── Dockerfile
└── .github/
    └── workflows/
        └── build.yml
```

---

## 4.3 GitOps Repository

```text
family-menu-gitops/
```

第一版：

```text
family-menu-gitops/
├── apps/
│   └── family-menu.yaml
└── README.md
```

---

# 5. Phase 0 — Rust Project Bootstrap

## Goal

建立一个能够运行在 Kubernetes 中的最小 Rust Controller。

---

## Tasks

### 0.1 初始化 Cargo Project

创建：

```text
mini-argocd-controller
```

使用 Cargo。

---

### 0.2 添加核心 dependencies

至少包括：

```text
kube
kube-runtime
k8s-openapi
tokio
serde
serde_yaml
schemars
tracing
tracing-subscriber
```

具体版本由当前稳定兼容版本决定。

---

### 0.3 建立 Kubernetes client

Controller 启动时：

```text
load Kubernetes config
        ↓
create kube::Client
        ↓
log successful startup
```

---

### 0.4 Dockerfile

使用 multi-stage build。

目标：

```text
Rust build image
        ↓
compile binary
        ↓
minimal runtime image
```

尽量使用：

* non-root
* minimal runtime
* no compiler in final image

---

### 0.5 GitHub Actions

Controller repository 自己也需要 CI：

```text
cargo fmt --check
cargo check
cargo test
cargo clippy
docker build
docker push GHCR
```

---

## Definition of Done

```text
cargo test       PASS
cargo clippy     PASS
docker build     PASS
GHCR image       available
```

并且能够：

```bash
kubectl apply -f config/deployment.yaml
```

让 Controller Pod：

```text
Running
```

---

# 6. Phase 1 — HomeApp CRD

这是整个项目的第一个 Kubernetes abstraction。

定义：

```yaml
apiVersion: home.yang.dev/v1alpha1
kind: HomeApp

metadata:
  name: family-menu

spec:
  image: ghcr.io/example/family-menu:sha-abc123
  replicas: 2
```

---

## 6.1 Rust definition

使用 `kube-derive`：

```rust
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "home.yang.dev",
    version = "v1alpha1",
    kind = "HomeApp",
    namespaced
)]
pub struct HomeAppSpec {
    pub image: String,
    pub replicas: i32,
}
```

---

## 6.2 Generate CRD

项目应该能够生成：

```text
crd/homeapp.yaml
```

并将其安装到 Kubernetes。

---

## 6.3 Watch HomeApp

使用 kube-rs runtime controller：

```text
HomeApp
   ↓
watch
   ↓
reconcile()
```

---

## Definition of Done

执行：

```bash
kubectl apply -f crd/homeapp.yaml
```

然后：

```bash
kubectl apply -f examples/homeapp.yaml
```

Controller logs 能看到：

```text
HomeApp detected
```

---

# 7. Phase 2 — Reconcile HomeApp → Deployment

这是整个项目最重要的 Phase。

---

## Desired State

Controller 根据：

```yaml
spec:
  image: ghcr.io/example/family-menu:sha-abc123
  replicas: 2
```

生成：

```text
Deployment
```

例如：

```yaml
apiVersion: apps/v1
kind: Deployment

spec:
  replicas: 2

  template:
    spec:
      containers:
        - name: family-menu
          image: ghcr.io/example/family-menu:sha-abc123
```

---

## Reconciliation

实现：

```text
reconcile(HomeApp)
        |
        +-- Get HomeApp
        |
        +-- Calculate desired Deployment
        |
        +-- Get current Deployment
        |
        +-- Not found?
        |      |
        |      └── Create
        |
        +-- Exists?
        |      |
        |      └── Compare
        |
        └── Different?
               |
               └── Update
```

---

## Important

不要：

```rust
Command::new("kubectl")
```

不要执行 shell：

```text
kubectl apply
kubectl get
kubectl delete
```

全部使用：

```text
kube-rs
```

访问 Kubernetes API。

---

## OwnerReference

Deployment 应该拥有：

```text
HomeApp
```

作为 Owner。

关系：

```text
HomeApp
   │
   │ ownerReference
   ▼
Deployment
```

这样删除：

```bash
kubectl delete homeapp family-menu
```

之后 Kubernetes 可以自动 garbage collect Deployment。

---

## Definition of Done

```text
Create HomeApp
      ↓
Deployment automatically created
```

修改：

```yaml
replicas: 3
```

Deployment 自动变成：

```text
replicas = 3
```

修改：

```yaml
image: ...:sha-def456
```

Deployment 自动更新。

删除 HomeApp：

```text
HomeApp deleted
      ↓
Deployment deleted
```

---

# 8. Phase 3 — Status

HomeApp 不应该只有：

```text
spec
```

还应该有：

```text
status
```

例如：

```yaml
status:
  phase: Running
  replicas: 2
  availableReplicas: 2
```

---

## Phase values

第一版只需要：

```text
Pending
Progressing
Running
Degraded
```

---

## Status flow

```text
HomeApp created
      ↓
Pending
      ↓
Deployment created
      ↓
Pod starting
      ↓
Progressing
      ↓
Pods ready
      ↓
Running
```

---

## Definition of Done

执行：

```bash
kubectl get homeapp family-menu -o yaml
```

能够看到：

```yaml
status:
  phase: Running
  replicas: 2
  availableReplicas: 2
```

---

# 9. Phase 4 — Robust Reconciliation

现在开始验证 Controller 是否真的具有 Kubernetes Controller 的特征。

---

## Test 1 — Manual Drift

正常状态：

```text
HomeApp replicas = 2
Deployment replicas = 2
```

手动：

```bash
kubectl scale deployment family-menu --replicas=1
```

Controller 应该发现：

```text
Desired = 2
Actual = 1
```

然后恢复：

```text
Actual → 2
```

---

## Test 2 — Deployment Deleted

手动：

```bash
kubectl delete deployment family-menu
```

Controller 应该重新创建。

---

## Test 3 — Controller Restart

删除 Controller Pod：

```bash
kubectl delete pod ...
```

Controller 重启后应该读取当前 Kubernetes state，并恢复 reconciliation。

---

## Test 4 — Pod Failure

如果 Pod 出现：

```text
CrashLoopBackOff
```

Controller 不应该自己重新实现 Kubernetes Deployment controller。

它只需要正确维护：

```text
HomeApp → Deployment
```

并根据 Deployment 状态更新：

```text
HomeApp.status
```

---

# 10. Phase 5 — GitOps Repository

现在才真正加入 GitOps。

建立：

```text
family-menu-gitops
```

例如：

```yaml
apps/family-menu.yaml
```

内容：

```yaml
apiVersion: home.yang.dev/v1alpha1
kind: HomeApp

metadata:
  name: family-menu
  namespace: family-menu

spec:
  image: ghcr.io/yang/family-menu:sha-abc123
  replicas: 2
```

---

# 11. Phase 6 — Git Polling

Controller 增加 Git polling。

第一版**不做 webhook**。

例如：

```text
every 30 seconds
        |
        ▼
GitHub
        |
        ▼
GitOps repository
        |
        ▼
read apps/*.yaml
        |
        ▼
desired HomeApp
        |
        ▼
Kubernetes
```

---

## 为什么第一版用 polling

因为我们首先要验证：

```text
Git
 ↓
Desired State
 ↓
Kubernetes
```

而不是花时间解决：

```text
GitHub webhook
Ingress
TLS
public endpoint
authentication
```

Polling 足以完成 GitOps 的核心闭环。

---

# 12. Git Authentication

因为 GitOps repository 是 Private：

```text
family-menu-gitops
```

Controller 需要 credential。

第一版可以使用：

```text
Kubernetes Secret
```

例如：

```text
GITHUB_TOKEN
```

Controller 从 environment variable 或 Secret-mounted configuration 获取。

---

## Security rule

禁止：

```text
token hard-coded in source code
```

禁止：

```text
token committed into GitOps repo
```

Controller 只需要：

```text
read repository
```

因此 credential 应尽量是：

```text
read-only
```

---

# 13. Phase 7 — GitOps → HomeApp

Controller 每次 polling：

```text
GitOps repo
     ↓
read YAML
     ↓
parse HomeApp
     ↓
Get current HomeApp
     ↓
Create / Update
```

最终：

```text
Git commit
    ↓
Git polling
    ↓
HomeApp update
    ↓
HomeApp reconciliation
    ↓
Deployment update
    ↓
Pod rollout
```

---

# 14. Phase 8 — GitHub Actions → GitOps

这是完整 CI/CD 的最后一段。

Application repository：

```text
family-menu
```

GitHub Actions：

```text
git push
   ↓
tests
   ↓
Docker build
   ↓
GHCR push
   ↓
update GitOps repository
```

---

## Image Tag

不要使用：

```text
latest
```

推荐：

```text
sha-<git commit>
```

例如：

```text
ghcr.io/yang/family-menu:sha-abc123
```

这样每个 deployment 都可以明确对应 source commit。

更进一步，可以考虑最终使用 image digest：

```text
ghcr.io/yang/family-menu@sha256:...
```

但 v0.1 可以先使用 commit SHA tag。

---

# 15. GitHub Actions 更新 GitOps Repo

Workflow 需要对 Private GitOps Repo 有：

```text
contents: write
```

权限。

第一版可以使用：

```text
fine-grained GitHub token
```

并且限制到：

```text
family-menu-gitops
```

而不是给整个 GitHub account 的广泛权限。

---

## Workflow

逻辑：

```text
Build image
     ↓
Push GHCR
     ↓
Clone GitOps repo
     ↓
Update image field
     ↓
git diff
     ↓
commit
     ↓
push
```

---

## Example

GitOps：

```yaml
image: ghcr.io/yang/family-menu:sha-old
```

变成：

```yaml
image: ghcr.io/yang/family-menu:sha-new
```

Commit：

```text
Update family-menu image to sha-new
```

---

# 16. Phase 9 — Add Service

第一版先只管理：

```text
Deployment
```

GitOps loop 跑通以后，再加入：

```yaml
spec:
  image: ...
  replicas: 2
  port: 3000
```

Controller 自动创建：

```text
Deployment
Service
```

关系：

```text
HomeApp
 ├── Deployment
 └── Service
```

两个资源都使用 OwnerReference。

---

# 17. Phase 10 — End-to-End Test

最终必须验证完整链路。

开发者：

```text
change source code
       ↓
git push
```

GitHub：

```text
GitHub Actions
       ↓
test
       ↓
Docker build
       ↓
GHCR
       ↓
update GitOps repo
```

Home：

```text
GitOps repo
       ↓
Rust Controller
       ↓
HomeApp
       ↓
Deployment
       ↓
Pod
```

最终：

```text
new application version
        ↓
running in home k3s
```

---

# 18. Phase 11 — Security Hardening

Controller ServiceAccount 必须使用最小 RBAC。

允许：

```text
get
list
watch
create
update
patch
delete
```

但只针对 Controller 真正管理的 resources。

至少涉及：

```text
HomeApp
Deployments
Services
HomeApp status
```

不要直接：

```text
cluster-admin
```

---

## Container Security

Controller container：

```text
non-root
```

尽量：

```text
read-only filesystem
```

并：

```text
drop unnecessary Linux capabilities
```

---

# 19. Testing Strategy

## Unit Tests

重点测试：

```text
HomeApp → desired Deployment
```

例如：

```text
image = A
replicas = 2

→ Deployment image = A
→ replicas = 2
```

---

## Reconciliation Tests

测试：

```text
Deployment absent
→ create

Deployment wrong image
→ update

Deployment wrong replicas
→ update

Deployment correct
→ no unnecessary update
```

---

## Git Tests

测试：

```text
Git YAML
   ↓
parse
   ↓
HomeApp desired state
```

---

## End-to-End

最终至少测试：

```text
Git commit
→ GitHub Actions
→ GHCR
→ GitOps commit
→ Controller
→ HomeApp
→ Deployment
→ Pod
```

---

# 20. Observability

第一版不做 UI。

使用：

```text
tracing
```

日志至少包含：

```text
HomeApp name
namespace
reconciliation result
Git commit/version
Deployment action
error
retry
```

例如：

```text
INFO reconciled HomeApp
    namespace=family-menu
    name=family-menu
    image=ghcr.io/yang/family-menu:sha-abc123
```

---

# 21. Error Handling

Controller 必须正确处理：

```text
GitHub unavailable
Git authentication failure
invalid YAML
Kubernetes API unavailable
Deployment creation failure
Deployment update failure
```

Git polling失败不能导致 Controller crash。

应该：

```text
error
 ↓
log
 ↓
retry
```

而不是：

```text
error
 ↓
process exit
```

---

# 22. Reconciliation Model

最终代码应该体现这个核心模型：

```text
                ┌──────────────┐
                │ GitOps Repo  │
                └──────┬───────┘
                       │
                       │ desired state
                       ▼
              ┌─────────────────┐
              │ Rust Controller │
              │    kube-rs      │
              └────────┬────────┘
                       │
                       │ reconcile
                       ▼
              ┌─────────────────┐
              │   Kubernetes    │
              │     Actual      │
              │     State       │
              └────────┬────────┘
                       │
                       │ compare
                       ▼
                desired != actual
                       │
                       ▼
                    correct
```

这就是整个项目最重要的东西。

---

# 23. v0.1 Definition of Done

当下面整个链路成功时，Mini Argo CD 0.1 完成：

```text
Developer
   │
   │ git push
   ▼
GitHub
   │
   ▼
GitHub Actions
   │
   ├── test
   ├── build
   └── Docker image
          │
          ▼
         GHCR
          │
          ▼
   GitOps repository
          │
          │ Git commit
          ▼
   Rust kube-rs Controller
          │
          │ reconcile
          ▼
       HomeApp
          │
          ▼
      Deployment
          │
          ▼
         Pods
```

并且：

```text
✓ 不需要公网暴露 Kubernetes API
✓ GitHub 不需要主动访问家庭网络
✓ Controller 能自动纠正 Kubernetes drift
✓ Controller restart 后可以恢复
✓ GitOps repo 是 desired state source
✓ Docker image 使用 immutable version
✓ Controller 使用 kube-rs
✓ Controller 不执行 kubectl
✓ RBAC 使用 least privilege
```

---

# 24. Explicitly Deferred

以下功能不要进入 v0.1：

```text
Web UI
Webhook
Helm
Kustomize
Multi-cluster
Multi-tenancy
ApplicationSet
Automatic rollback
Canary
Blue/Green
Notifications
SSO
Advanced RBAC
GitHub App
Image updater
Signed images
Policy engine
Dependency graph
Application health framework
```

---

# 25. Future Roadmap

完成 v0.1 后可以逐步增加：

```text
v0.2
 └── Service

v0.3
 └── Multiple applications

v0.4
 └── Git webhook

v0.5
 └── Health checks

v0.6
 └── Rollout status

v0.7
 └── Automatic rollback

v0.8
 └── Helm support

v0.9
 └── Kustomize support

v1.0
 └── Mini Argo CD
```

然后再与真正的 Argo CD 对比：

```text
Mini Argo CD
      vs
Argo CD
```

看看真正的 Argo CD 为了解决 production-scale GitOps 又增加了哪些东西。

---

# 26. Development Rules for AI Coding Agent

Coding agent 必须遵守：

1. **使用 Rust + kube-rs，不使用 Go。**
2. 不使用 Kubebuilder。
3. 不使用 controller-runtime。
4. Controller 不执行 `kubectl`。
5. Kubernetes API 操作全部通过 kube-rs。
6. 不引入数据库。
7. 不引入 message queue。
8. 不引入 Web UI。
9. 不为了未来功能提前设计复杂 abstraction。
10. 每完成一个 Phase，必须运行对应 tests。
11. 不进入下一 Phase，直到当前 Phase 的 Definition of Done 满足。
12. 优先使用 Kubernetes 原生机制。
13. 保持代码简单、显式、容易阅读。
14. 不为了“生产级”而提前实现 Argo CD 的大量功能。
15. 所有 secret 禁止进入 Git。
16. 所有 Docker image 使用 immutable tag。
17. Controller 必须支持 restart/reconciliation。
18. Git polling failure 必须 retry，而不是导致 process crash。

---

# 27. 最终 Architecture Decision

本项目最终采用：

```text
Language:
    Rust

Kubernetes client:
    kube-rs

Kubernetes runtime:
    kube-runtime

CRD:
    kube-derive

Async runtime:
    Tokio

Serialization:
    serde / serde_yaml

Logging:
    tracing

Cluster:
    k3s

CI:
    GitHub Actions

Registry:
    GHCR

GitOps:
    Private GitHub repository

Git synchronization:
    Polling

Deployment model:
    Kubernetes Deployment

Controller model:
    Reconciliation loop
```

核心设计：

```text
GitHub Actions
      │
      │ push image
      ▼
     GHCR
      │
      │ update
      ▼
GitOps Repository
      │
      │ pull
      ▼
Rust + kube-rs
      │
      │ reconcile
      ▼
 Kubernetes API
      │
      ▼
     k3s
```

**这就是 Mini Argo CD 0.1。**
