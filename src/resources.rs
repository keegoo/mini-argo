use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "miniargo.dev",
    version = "v1",
    kind = "Application",
    namespaced
)]
pub struct ApplicationSpec {
    pub image: String,
    pub replicas: i32
}
