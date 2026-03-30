//! Kubernetes RBAC UX helpers

use axum::{extract::State, Json};
use k8s_openapi::api::authorization::v1::{
    ResourceAttributes, SelfSubjectAccessReview, SelfSubjectAccessReviewSpec,
};
use kube::api::{Api, PostParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct RbacCheckRequest {
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RbacVerbMatrix {
    pub get: bool,
    pub list: bool,
    pub watch: bool,
    pub create: bool,
    pub update: bool,
    pub patch: bool,
    pub delete: bool,
}

#[derive(Debug, Serialize)]
pub struct RbacResourceCheck {
    pub resource: String,
    pub namespaced: bool,
    pub verbs: RbacVerbMatrix,
}

#[derive(Debug, Serialize)]
pub struct SecretAccessHints {
    pub has_get: bool,
    pub has_list: bool,
    pub has_watch: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RbacCheckResponse {
    pub namespace: Option<String>,
    pub resources: Vec<RbacResourceCheck>,
    pub secrets_hints: SecretAccessHints,
}

async fn can_i(
    api: &Api<SelfSubjectAccessReview>,
    group: &str,
    resource: &str,
    verb: &str,
    namespace: Option<&str>,
) -> Result<bool> {
    let review = SelfSubjectAccessReview {
        metadata: Default::default(),
        spec: SelfSubjectAccessReviewSpec {
            non_resource_attributes: None,
            resource_attributes: Some(ResourceAttributes {
                group: if group.is_empty() {
                    None
                } else {
                    Some(group.to_string())
                },
                name: None,
                namespace: namespace.map(str::to_string),
                resource: Some(resource.to_string()),
                subresource: None,
                verb: Some(verb.to_string()),
                version: None,
            }),
        },
        status: None,
    };

    let created = api
        .create(&PostParams::default(), &review)
        .await
        .map_err(|e| Error::Kubernetes(format!("RBAC SelfSubjectAccessReview failed: {e}")))?;

    Ok(created
        .status
        .as_ref()
        .map(|s| s.allowed)
        .unwrap_or(false))
}

async fn check_resource(
    api: &Api<SelfSubjectAccessReview>,
    group: &str,
    resource: &str,
    namespaced: bool,
    namespace: Option<&str>,
) -> Result<RbacResourceCheck> {
    let ns = if namespaced { namespace } else { None };
    Ok(RbacResourceCheck {
        resource: resource.to_string(),
        namespaced,
        verbs: RbacVerbMatrix {
            get: can_i(api, group, resource, "get", ns).await?,
            list: can_i(api, group, resource, "list", ns).await?,
            watch: can_i(api, group, resource, "watch", ns).await?,
            create: can_i(api, group, resource, "create", ns).await?,
            update: can_i(api, group, resource, "update", ns).await?,
            patch: can_i(api, group, resource, "patch", ns).await?,
            delete: can_i(api, group, resource, "delete", ns).await?,
        },
    })
}

pub async fn check_kubernetes_rbac(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RbacCheckRequest>,
) -> Result<Json<RbacCheckResponse>> {
    let client = state.kubernetes_client()?;
    let review_api: Api<SelfSubjectAccessReview> = Api::all(client.raw().clone());
    let ns = payload.namespace.as_deref();

    let mut resources = Vec::new();
    resources.push(check_resource(&review_api, "", "namespaces", false, ns).await?);
    resources.push(check_resource(&review_api, "", "services", true, ns).await?);
    resources.push(check_resource(&review_api, "", "configmaps", true, ns).await?);
    resources.push(check_resource(&review_api, "", "secrets", true, ns).await?);
    resources.push(check_resource(&review_api, "", "persistentvolumeclaims", true, ns).await?);
    resources.push(check_resource(&review_api, "", "persistentvolumes", false, ns).await?);
    resources.push(check_resource(&review_api, "storage.k8s.io", "storageclasses", false, ns).await?);
    resources.push(check_resource(&review_api, "networking.k8s.io", "ingresses", true, ns).await?);
    resources.push(check_resource(&review_api, "networking.k8s.io", "networkpolicies", true, ns).await?);
    resources.push(check_resource(&review_api, "networking.k8s.io", "ingressclasses", false, ns).await?);
    resources.push(check_resource(&review_api, "batch", "jobs", true, ns).await?);
    resources.push(check_resource(&review_api, "batch", "cronjobs", true, ns).await?);
    resources.push(check_resource(&review_api, "policy", "poddisruptionbudgets", true, ns).await?);
    resources.push(
        check_resource(
            &review_api,
            "scheduling.k8s.io",
            "priorityclasses",
            false,
            ns,
        )
        .await?,
    );
    resources.push(check_resource(&review_api, "", "serviceaccounts", true, ns).await?);
    resources.push(
        check_resource(
            &review_api,
            "rbac.authorization.k8s.io",
            "roles",
            true,
            ns,
        )
        .await?,
    );
    resources.push(
        check_resource(
            &review_api,
            "rbac.authorization.k8s.io",
            "rolebindings",
            true,
            ns,
        )
        .await?,
    );
    resources.push(
        check_resource(
            &review_api,
            "rbac.authorization.k8s.io",
            "clusterroles",
            false,
            ns,
        )
        .await?,
    );
    resources.push(
        check_resource(
            &review_api,
            "rbac.authorization.k8s.io",
            "clusterrolebindings",
            false,
            ns,
        )
        .await?,
    );
    resources.push(
        check_resource(
            &review_api,
            "autoscaling",
            "horizontalpodautoscalers",
            true,
            ns,
        )
        .await?,
    );
    resources.push(check_resource(&review_api, "", "resourcequotas", true, ns).await?);
    resources.push(check_resource(&review_api, "", "limitranges", true, ns).await?);
    resources.push(
        check_resource(
            &review_api,
            "apiextensions.k8s.io",
            "customresourcedefinitions",
            false,
            ns,
        )
        .await?,
    );

    let secrets = resources
        .iter()
        .find(|r| r.resource == "secrets")
        .ok_or_else(|| Error::Other("Secrets RBAC check missing".to_string()))?;

    let warning = if secrets.verbs.get && secrets.verbs.list && !secrets.verbs.watch {
        Some("User can get/list secrets but cannot watch; live updates should be disabled and UI should show manual refresh.".to_string())
    } else if (secrets.verbs.get || secrets.verbs.list) && !secrets.verbs.watch {
        Some("Limited secret permissions detected (no watch).".to_string())
    } else {
        None
    };
    let has_get = secrets.verbs.get;
    let has_list = secrets.verbs.list;
    let has_watch = secrets.verbs.watch;

    Ok(Json(RbacCheckResponse {
        namespace: payload.namespace,
        resources,
        secrets_hints: SecretAccessHints {
            has_get,
            has_list,
            has_watch,
            warning,
        },
    }))
}
