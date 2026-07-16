use zbus::{interface, Connection};
use tracing::{info, warn, error};
use std::collections::HashMap;
use std::process::Command;

pub struct ErmetePortal;

impl ErmetePortal {
    async fn request_permission(resource: &str, app_id: &str) -> bool {
        info!("Prompting user for {} permission for app: {}", resource, app_id);
        
        // Spawns the GTK4 prompt from ermete-shell-rs
        let status = Command::new("ermete-shell-rs")
            .arg("--privacy-prompt")
            .arg(format!("{}:{}", resource, app_id))
            .status();

        match status {
            Ok(exit_status) => {
                let granted = exit_status.success();
                if granted {
                    info!("Permission GRANTED for {}.", resource);
                    // Notify Topbar to turn on the Privacy Indicator (Green/Orange/Purple dot)
                    tokio::spawn(async move {
                        let res = resource.to_string();
                        if let Ok(conn) = Connection::session().await {
                            let _ = conn.call_method(
                                Some("os.ermete.Shell"),
                                "/os/ermete/Shell",
                                Some("os.ermete.Shell"),
                                "SetPrivacyIndicator",
                                &(res, true),
                            ).await;
                        }
                    });
                } else {
                    info!("Permission DENIED for {}.", resource);
                }
                granted
            }
            Err(e) => {
                error!("Failed to launch privacy prompt: {}", e);
                false // Deny by default on error
            }
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.ScreenCast")]
impl ErmetePortal {
    async fn create_session(&self, _handle: String, _session_handle: String, app_id: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<u32, zbus::fdo::Error> {
        if Self::request_permission("ScreenCast", &app_id).await {
            Ok(0) // 0 = Success
        } else {
            Ok(1) // 1 = User Cancelled
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.Camera")]
impl ErmetePortal {
    async fn access_camera(&self, _handle: String, app_id: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<u32, zbus::fdo::Error> {
        if Self::request_permission("Camera", &app_id).await {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.Location")]
impl ErmetePortal {
    #[zbus(name = "CreateSession")]
    async fn create_location_session(&self, _handle: String, _session_handle: String, app_id: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<u32, zbus::fdo::Error> {
        if Self::request_permission("Location", &app_id).await {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.Microphone")]
impl ErmetePortal {
    async fn access_microphone(&self, _handle: String, app_id: String, _options: HashMap<String, zbus::zvariant::Value<'_>>) -> std::result::Result<u32, zbus::fdo::Error> {
        if Self::request_permission("Microphone", &app_id).await {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}
