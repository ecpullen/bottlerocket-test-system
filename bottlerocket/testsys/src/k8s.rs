use crate::error::{self, Result};
use kube::{
    api::{Api, Patch, PatchParams, PostParams, ResourceExt},
    config::{KubeConfigOptions, Kubeconfig},
    Client, Config,
};
use serde::{de::DeserializeOwned, Serialize};
use snafu::ResultExt;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::path::PathBuf;

/// Create the k8s client. If the `kubeconfig` path is given then we use it. If not, then we
/// construct use the `Client`'s default construction which attempts to use environment variables
/// (e.g. `KUBECONFIG`).
pub(crate) async fn k8s_client(kubeconfig: &Option<PathBuf>) -> Result<Client> {
    match kubeconfig {
        None => Ok(Client::try_default()
            .await
            .context(error::ClientCreateSnafu)?),
        Some(config_path) => {
            let kubeconfig = Kubeconfig::read_from(config_path).context(error::ConfigReadSnafu)?;
            let config = Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default())
                .await
                .context(error::ClientCreateKubeconfigSnafu)?;
            Ok(config.try_into().context(error::ClientCreateSnafu)?)
        }
    }
}

const MAX_RETRIES: i32 = 3;
const BACKOFF_MS: u64 = 500;

/// Create or update an object in `api` with `data`'s name
pub(crate) async fn create_or_update<T>(api: &Api<T>, data: T, what: &str) -> Result<()>
where
    T: Clone + DeserializeOwned + Debug + kube::Resource + Serialize,
{
    let mut error = None;

    for _ in 0..MAX_RETRIES {
        match create_or_update_internal(api, data.clone(), what).await {
            Ok(()) => return Ok(()),
            Err(e) => error = Some(e),
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(BACKOFF_MS)).await;
    }
    match error {
        None => Ok(()),
        Some(error) => Err(error),
    }
}

async fn create_or_update_internal<T>(api: &Api<T>, data: T, what: &str) -> Result<()>
where
    T: Clone + DeserializeOwned + Debug + kube::Resource + Serialize,
{
    // If the data already exists, update it with the new one using a `Patch`. If not create a new one.
    match api.get(&data.name_any()).await {
        Ok(deployment) => {
            api.patch(
                &deployment.name_any(),
                &PatchParams::default(),
                &Patch::Merge(data),
            )
            .await
        }
        Err(_err) => api.create(&PostParams::default(), &data).await,
    }
    .context(error::CreationSnafu { what })?;

    Ok(())
}

#[derive(Serialize)]
pub(crate) struct DockerConfigJson {
    auths: HashMap<String, DockerConfigAuth>,
}

#[derive(Serialize)]
struct DockerConfigAuth {
    auth: String,
}

impl DockerConfigJson {
    pub(crate) fn new(username: &str, password: &str, registry: &str) -> DockerConfigJson {
        let mut auths = HashMap::new();
        let auth = base64::encode(format!("{}:{}", username, password));
        auths.insert(registry.to_string(), DockerConfigAuth { auth });
        DockerConfigJson { auths }
    }
}
