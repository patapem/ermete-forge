use zbus::{interface, Result};
use tracing::info;
use crate::github::GitHubReporter;

pub struct TelemetryIface;

#[interface(name = "os.ermete.Telemetry")]
impl TelemetryIface {
    /// Submits a crash report to the Forge. Requires Polkit auth if we want to ensure only the console user can do it.
    async fn submit_crash(&self, crash_id: String) -> Result<String> {
        info!("Received D-Bus request to submit crash: {}", crash_id);
        
        let reporter = GitHubReporter::new();
        // Here we would fetch the crash dump using `coredumpctl info <crash_id>`
        let dummy_data = format!("Crash dump for {}", crash_id);
        
        if let Err(e) = reporter.report_crash(&dummy_data).await {
            return Ok(format!("Failed to submit: {}", e));
        }
        
        Ok("Crash report successfully submitted.".into())
    }
}
