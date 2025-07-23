use std::sync::Arc;
use crate::services::health_check::check_rpc_health;
use crate::services::web3_service::Web3Service;

pub async fn start_health_monitor(web3_service: Arc<Web3Service>) {
    tokio::spawn(async move {
        loop {
            if !check_rpc_health(&web3_service).await {
                log::warn!("RPC endpoint health check failed, will try fallback endpoints");
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}