use std::error::Error;
use zbus::ConnectionBuilder;

struct Bedrock;

#[zbus::dbus_interface(name = "os.ermete.Bedrock")]
impl Bedrock {
    async fn ping(&self) -> String {
        "pong".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _conn = ConnectionBuilder::session()?
        .name("os.ermete.Bedrock")?
        .serve_at("/os/ermete/Bedrock", Bedrock)?
        .build()
        .await?;

    println!("Ermete Bedrock Daemon started.");
    std::future::pending::<()>().await;
    Ok(())
}
