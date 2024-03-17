use log::{error, info};
use tokio::process::Command;

pub async fn restart_coredns() -> Result<(), Box<dyn std::error::Error>> {
    // Restart coredns service using systemctl command
    let output = Command::new("systemctl")
        .arg("restart")
        .arg("coredns")
        .output()
        .await;

    match output {
        Ok(output) => {
            if output.status.success() {
                info!("CoreDNS service restarted successfully");
                Ok(())
            } else {
                error!("Failed to restart CoreDNS service");
                Err("Failed to restart CoreDNS service".into())
            }
        }
        Err(e) => {
            Err(e.into())
        }
    }
}
