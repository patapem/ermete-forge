use anyhow::{Context, Result};
use tracing::{info, warn};
use tokio::process::Command;
use serde_json::Value;

#[derive(Debug)]
pub enum UpdateStatus {
    NoUpdates,
    Layer1AppliedLive,
    Layer0RebootRequired,
}

pub struct UpdateEngine {
    // Future ZBus proxy objects could live here
}

impl UpdateEngine {
    pub async fn new() -> Result<Self> {
        info!("Initializing UpdateEngine...");
        Ok(Self {})
    }

    pub async fn check_and_apply(&mut self) -> Result<UpdateStatus> {
        // Step 1: Check Layer 1 (Userspace) updates via rpm-ostree
        info!("Checking rpm-ostree for Layer 1 updates...");
        
        let ostree_output = Command::new("rpm-ostree")
            .arg("status")
            .arg("--json")
            .output()
            .await
            .context("Failed to execute rpm-ostree")?;

        if ostree_output.status.success() {
            let ostree_json: Value = serde_json::from_slice(&ostree_output.stdout).unwrap_or(Value::Null);
            
            // `rpm-ostree status --json` usually has a "deployments" array. 
            // We check if there's a staged deployment waiting, or updates available.
            // Placeholder heuristic for now:
            let has_layer1_updates = ostree_json.to_string().contains("AvailableUpdate"); 
            
            if has_layer1_updates {
                info!("Layer 1 updates found. Applying live...");
                // Command::new("rpm-ostree").arg("apply-live").output().await?;
                return Ok(UpdateStatus::Layer1AppliedLive);
            }
        }

        // Step 2: Check Layer 0 (Core OS) updates via bootc
        info!("Checking bootc for Layer 0 updates...");
        
        let bootc_output = Command::new("bootc")
            .arg("status")
            .arg("--json")
            .output()
            .await
            .context("Failed to execute bootc")?;

        if bootc_output.status.success() {
            let bootc_json: Value = serde_json::from_slice(&bootc_output.stdout).unwrap_or(Value::Null);
            
            // `bootc status` JSON contains "status": {"type": "..."}
            let is_bootc_available = bootc_json["status"]["type"] == "updateAvailable" 
                || bootc_json.to_string().contains("updateAvailable");
                
            if is_bootc_available {
                info!("Layer 0 core update found. Stage for next reboot...");
                // Command::new("bootc").arg("upgrade").output().await?;
                warn!("A system reboot is required to apply the new Kernel/DKMS layer.");
                return Ok(UpdateStatus::Layer0RebootRequired);
            }
        }

        Ok(UpdateStatus::NoUpdates)
    }
}
