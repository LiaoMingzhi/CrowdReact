use web3::types::{U256, H160};
use web3::Web3;
use std::sync::Arc;
use crate::utils::error::ServiceError;

pub struct Web3Service {
    pub rpc_urls: Vec<String>,
    current_index: std::sync::atomic::AtomicUsize,
    contract_address: H160,
}

impl Web3Service {
    pub fn new(rpc_urls: Vec<String>, contract_address: H160) -> Self {
        Self {
            rpc_urls,
            current_index: std::sync::atomic::AtomicUsize::new(0),
            contract_address,
        }
    }

    pub async fn get_contract_balance(&self) -> Result<U256, ServiceError> {
        let mut last_error = None;
        
        // 尝试所有可用的 RPC 端点
        for _ in 0..self.rpc_urls.len() {
            let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst) 
                % self.rpc_urls.len();
            
            let rpc_url = &self.rpc_urls[index];
            log::info!("Trying RPC endpoint: {}", rpc_url);
            
            match self.get_balance_from_rpc(rpc_url).await {
                Ok(balance) => {
                    log::info!("Successfully got balance from {}: {} wei", rpc_url, balance);
                    return Ok(balance);
                }
                Err(e) => {
                    log::warn!("Failed to get balance from {}: {}", rpc_url, e);
                    last_error = Some(e);
                    continue;
                }
            }
        }
        
        Err(ServiceError::InternalServerError(
            last_error.map(|e| e.to_string())
                .unwrap_or_else(|| "All RPC endpoints failed".to_string())
        ))
    }
    
    async fn get_balance_from_rpc(&self, rpc_url: &str) -> Result<U256, ServiceError> {
        let transport = web3::transports::Http::new(rpc_url)
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to create transport: {}", e)))?;
            
        let web3 = Web3::new(transport);
        
        // 添加超时设置
        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(180),
            web3.eth().balance(self.contract_address, None)
        ).await
        .map_err(|e| ServiceError::InternalServerError(format!("RPC request timeout: {}", e)))?
        .map_err(|e| ServiceError::InternalServerError(format!("Failed to get balance: {}", e)))?;
        
        Ok(timeout)
    }
}