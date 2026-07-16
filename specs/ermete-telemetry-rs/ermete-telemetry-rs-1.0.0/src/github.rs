use anyhow::Result;
use reqwest::Client;
use tracing::info;

pub struct GitHubReporter {
    client: Client,
}

impl GitHubReporter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn report_crash(&self, crash_data: &str) -> Result<()> {
        info!("Sending crash report to Ermete Forge GitHub Issues...");
        // Placeholder for HTTP POST to api.github.com/repos/patapem/ermete-forge/issues
        // Needs a Fine-Grained Personal Access Token with read/write issues scope.
        Ok(())
    }
}
