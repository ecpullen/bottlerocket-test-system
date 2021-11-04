mod provider;

use crate::provider::{EksCreator, EksDestroyer};
use resource_agent::clients::{DefaultAgentClient, DefaultInfoClient};
use resource_agent::error::AgentResult;
use resource_agent::{Agent, BootstrapData, Types};
use std::marker::PhantomData;

#[tokio::main]
async fn main() {
    let data = match BootstrapData::from_env() {
        Ok(ok) => ok,
        Err(e) => {
            eprintln!("Unable to get bootstrap data: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = run(data).await {
        eprintln!("{}", e);
        std::process::exit(1);
    };
}

async fn run(data: BootstrapData) -> AgentResult<()> {
    let types = Types {
        info_client: PhantomData::<DefaultInfoClient>::default(),
        agent_client: PhantomData::<DefaultAgentClient>::default(),
    };

    let agent = Agent::new(types, data, EksCreator {}, EksDestroyer {}).await?;
    agent.run().await
}
