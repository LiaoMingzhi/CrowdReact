use crate::services::web3_service::Web3Service;

pub async fn check_rpc_health(web3_service: &Web3Service) -> bool {
    match web3_service.get_contract_balance().await {
        Ok(_) => true,
        Err(e) => {
            log::error!("RPC health check failed: {}", e);
            false
        }
    }
}