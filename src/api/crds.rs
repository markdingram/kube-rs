use crate::{
    api::{raw::RawApi, typed::Api},
    client::APIClient,
};
use inflector::{cases::pascalcase::is_pascal_case, string::pluralize::to_plural};
use std::marker::PhantomData;

/// A data equivalent of the Resource trait for for Custom Resources
///
/// This is the smallest amount of info we need to run the API against a CR
/// The version, and group must be set by the user.
pub struct CustomResource {
    kind: String,
    group: String,
    version: String,
    api_version: String,
    namespace: Option<String>,
}

impl CustomResource {
    /// Construct a CrBuilder
    pub fn new(kind: &str) -> CrBuilder {
        CrBuilder::new(kind)
    }
}

/// A builder for CustomResource
#[derive(Default)]
pub struct CrBuilder {
    pub(crate) kind: String,
    pub(crate) version: Option<String>,
    pub(crate) group: Option<String>,
    pub(crate) namespace: Option<String>,
}
impl CrBuilder {
    /// Create a CrBuilder
    ///
    /// ```
    /// use kube::api::{CustomResource, RawApi};
    /// struct Foo {
    ///     spec: FooSpec,
    ///     status: FooStatus,
    /// };
    /// let foos : RawApi<Foo> = CustomResource::new("Foo") // <.spec.kind>
    ///    .group("clux.dev") // <.spec.group>
    ///    .version("v1")
    ///    .build()
    ///    .into();
    /// ```
    fn new(kind: &str) -> Self {
        assert!(to_plural(kind) != kind); // no plural in kind
        assert!(is_pascal_case(&kind)); // PascalCase kind
        Self {
            kind: kind.into(),
            ..Default::default()
        }
    }

    /// Set the api group of a custom resource
    pub fn group(mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self
    }

    /// Set the api version of a custom resource
    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Set the namespace of a custom resource
    pub fn within(mut self, ns: &str) -> Self {
        self.namespace = Some(ns.into());
        self
    }

    // Build a RawApi from Crd properties
    pub fn build(self) -> CustomResource {
        let version = self.version.expect("Crd must have a version");
        let group = self.group.expect("Crd must have a group");
        CustomResource {
            api_version: format!("{}/{}", group, version),
            kind: self.kind,
            version,
            group,
            namespace: self.namespace,
        }
    }
}

/// Make RawApi useable on CRDs without k8s_openapi
impl<K> From<CustomResource> for RawApi<K> {
    fn from(c: CustomResource) -> Self {
        Self {
            api_version: c.api_version,
            kind: c.kind,
            group: c.group,
            version: c.version,
            namespace: c.namespace,
            phantom: PhantomData,
        }
    }
}

/// Make Api useable on CRDs without k8s_openapi
impl CustomResource {
    pub fn to_api<K>(self, client: APIClient) -> Api<K> {
        Api {
            client,
            api: self.into(),
            phantom: PhantomData,
        }
    }
}


#[cfg(test)]
mod test {
    use crate::api::{CustomResource, PatchParams, PostParams, RawApi};
    // non-openapi tests
    #[test]
    fn raw_custom_resource() {
        struct Foo {};
        let r: RawApi<Foo> = CustomResource::new("Foo")
            .group("clux.dev")
            .version("v1")
            .within("myns")
            .build()
            .into();
        let pp = PostParams::default();
        let req = r.create(&pp, vec![]).unwrap();
        assert_eq!(req.uri(), "/apis/clux.dev/v1/namespaces/myns/foos?");
        let patch_params = PatchParams::default();
        let req = r.patch("baz", &patch_params, vec![]).unwrap();
        assert_eq!(req.uri(), "/apis/clux.dev/v1/namespaces/myns/foos/baz?");
        assert_eq!(req.method(), "PATCH");
    }


    #[cfg(feature = "openapi")]
    #[tokio::test]
    async fn convenient_custom_resource() {
        use crate::{api::Api, client::APIClient, config};
        struct Foo {};
        let config = config::load_kube_config().await.unwrap();
        let client = APIClient::new(config);
        let _r: Api<Foo> = CustomResource::new("Foo")
            .group("clux.dev")
            .version("v1")
            .within("myns")
            .build()
            .to_api(client);
    }
}
