use zbus::{Connection, proxy};

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;
}

pub async fn get_network_status_dbus() -> (String, String, String) {
    if let Ok(connection) = Connection::system().await {
        if let Ok(proxy) = NetworkManagerProxy::new(&connection).await {
            // Simplified fallback for now: just check if wifi is enabled
            if let Ok(enabled) = proxy.wireless_enabled().await {
                if enabled {
                    return ("".to_string(), "Rete Wi-Fi".to_string(), "Connesso".to_string());
                }
            }
        }
    }
    ("󰖪".to_string(), "Rete Wi-Fi".to_string(), "Disattivato".to_string())
}
