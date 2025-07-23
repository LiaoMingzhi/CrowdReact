use web3::Web3;
use web3::transports::Http;
use log::{info, error, debug};
use ethers::utils::parse_ether;
use std::error::Error;

pub async fn verify_contract(
    web3: &Web3<Http>, 
    contract_address: &str,
    from_address: &str,
    amount: f64
) -> Result<bool, Box<dyn Error>> {
    debug!("Verifying contract interaction");
    
    // 检查合约地址
    let code = web3.eth().code(
        contract_address.parse()?,
        None
    ).await?;
    
    if code.0.is_empty() {
        error!("No contract code found at address: {}", contract_address);
        return Ok(false);
    }
    
    // 检查发送者余额
    let balance = web3.eth().balance(
        from_address.parse()?,
        None
    ).await?;
    
    let required_wei = parse_ether(amount.to_string())?;
    if balance < required_wei {
        error!("Insufficient balance. Required: {}, Available: {}", required_wei, balance);
        return Ok(false);
    }
    
    debug!("Contract verification passed");
    Ok(true)
} 