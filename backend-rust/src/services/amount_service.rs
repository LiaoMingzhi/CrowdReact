extern crate web3;

use crate::models::agent_model::{self, Entity as Agent, Model as AgentModel};
use hex;
use rand::seq::SliceRandom;
use sea_orm::prelude::Decimal;
use sea_orm::{Condition, DatabaseConnection, DbErr, EntityTrait, QuerySelect, QueryFilter,
              ColumnTrait, ConnectionTrait, Statement, DatabaseBackend, ActiveModelTrait, NotSet};
use sea_orm::sea_query::Expr;
use std::str::FromStr;
use std::sync::Arc;
use web3::ethabi::ethereum_types::Secret;
use web3::transports::Http;
use web3::types::{Bytes, TransactionParameters, TransactionRequest, H160, U256, H256, CallRequest, U64, TransactionReceipt};
use web3::signing::Signature;

use web3::{Web3, Error as Web3Error};

use rand::rngs::OsRng;
use web3::signing::{self, SecretKey, keccak256, Key};
use secp256k1::{Message, Secp256k1};
use std::ops::Deref;
use chrono::Utc;
use ethers::abi::Address;
use log::{info, error};
use rust_decimal::prelude::FromPrimitive;
use sea_orm::ActiveValue::Set;
use serde_json::json;
use web3::contract::{Contract, Options};
use crate::models::commission_model::{self, Entity as Commission, Model as CommissionModel, ActiveModel as CommissionActiveModel};
use crate::models::platform_funds_flow_model::{self, Entity as PlatformFundsFlow, ActiveModel as PlatformFundsFlowActiveModel};
use crate::models::platform_prize_pool_model::{self, Entity as PlatformPrizePool, ActiveModel as PlatformPrizePoolActiveModel};
use crate::models::lottery_distribution_detail_model::{
    self, 
    Entity as LotteryDistributionDetail, 
    Model as LotteryDistributionDetailModel,
    ActiveModel as LotteryDistributionDetailActiveModel
};
use crate::models::platform_transaction_gas_model::{
    self, 
    Entity as PlatformTransactionGas, 
    Model as PlatformTransactionGasModel,
    ActiveModel as PlatformTransactionGasActiveModel};
use crate::services::web3_service::Web3Service;
use crate::utils::error::ServiceError;
use web3::ethabi::{Token, self};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct EthSigner {
    secret_key: SecretKey,
}

impl EthSigner {
    pub fn new(private_key_hex: &str) -> Result<Self, DbErr> {
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|_| DbErr::Custom("无效的十六进制字符串".to_string()))?;

        let secret_key = SecretKey::from_slice(&private_key_bytes)
            .map_err(|_| DbErr::Custom("无效的私钥".to_string()))?;

        Ok(Self { secret_key })
    }

    pub fn as_secret_key(&self) -> &SecretKey {
        &self.secret_key
    }
}

#[derive(Debug)]
pub struct TransactionResult {
    pub hash: String,
    pub status: bool,
    pub gas_used: U256,
}

#[derive(Clone)]
pub struct AmountService {
    db: Arc<DatabaseConnection>,
    web3: Arc<Web3<Http>>,
    contract_address: String,
    owner_address: String,
    prize_pool_account: String,
}

impl AmountService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        web3: Web3<Http>,
        contract_address: String,
        owner_address: String,
        prize_pool_account: String,
    ) -> Self {
        // Read the owner address from config
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/production"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()
            .expect("Failed to load config");
            
        let owner_address = config.get_string("contract.owner")
            .unwrap_or_else(|_| owner_address);

        Self {
            db,
            web3: Arc::new(web3),
            contract_address,
            owner_address,  // This will now use the correct owner address from config
            prize_pool_account,
        }
    }

    // 获取用户余额
    pub async fn get_balance(&self, address: &str) -> Result<Decimal, DbErr> {
        // 确保地址是有效的
        let address = address
            .parse::<web3::types::H160>()
            .map_err(|_| DbErr::Custom("Invalid Ethereum address".to_string()))?;

        // 查询以太坊余额
        let balance: U256 = self
            .web3
            .eth()
            .balance(address, None)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to fetch balance: {}", e)))?;

        // 将余额转换为 Decimal 类型
        let balance_decimal = Decimal::from_str_exact(&balance.to_string())
            .map_err(|_| DbErr::Custom("Failed to convert balance to Decimal".to_string()))?;

        Ok(balance_decimal)
    }

    // 检查是否是代理
    pub async fn is_agent(&self, address: &str) -> Result<bool, DbErr> {
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(address))
            .one(&*self.db)
            .await?;

        Ok(agent.is_some())
    }

    // 获取代理信息
    pub async fn get_agent(&self, address: &str) -> Result<Option<AgentModel>, DbErr> {
        Agent::find()
            .filter(agent_model::Column::UserAddress.eq(address))
            .one(&*self.db)
            .await
    }

    // 检查代理级别
    pub async fn check_agent_level(&self, address: &str) -> Result<Option<String>, DbErr> {
        let agent = self.get_agent(address).await?;
        Ok(agent.map(|a| a.level_agent))
    }

    // 转账
    pub async fn transfer(&self, from: &str, to: &str, amount: Decimal) -> Result<String, DbErr> {
        // 确保地址是有效的
        let from_address = from
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的发送地址".to_string()))?;
        let to_address = to
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的接收地址".to_string()))?;

        // 将 Decimal 转换为 U256
        let amount_u256 = U256::from_dec_str(&(amount * Decimal::from_str("1e18").unwrap()).to_string())
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        // 获取当前 nonce
        let nonce = self
            .web3
            .eth()
            .transaction_count(from_address, None)
            .await
            .map_err(|e| DbErr::Custom(format!("获取 nonce 失败: {}", e)))?;

        // 获取当前 gas price
        let gas_price = self
            .web3
            .eth()
            .gas_price()
            .await
            .map_err(|e| DbErr::Custom(format!("获取 gas 价格失败: {}", e)))?;

        // 获取链 ID
        let chain_id = self
            .web3
            .eth()
            .chain_id()
            .await
            .map_err(|e| DbErr::Custom(format!("获取链 ID 失败: {}", e)))?;

        // 将 U256 转换为 Option<u64>
        let chain_id_option = if chain_id <= U256::from(u64::MAX) {
            Some(chain_id.as_u64())
        } else {
            None // 如果链 ID 超出 u64 范围，返回 None
        };

        // 构建交易请求
        let tx_request = TransactionRequest {
            from: from_address,
            to: Some(to_address),
            value: Some(amount_u256),
            gas: Some(U256::from(21000)),
            gas_price: Some(gas_price),
            nonce: Some(nonce),
            data: None,
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            condition: None,
        };

        // 使用 get_private_key 获取正确的私钥
        let private_key = self.get_private_key().await?;
        let key = create_key_from_private_key(&private_key)?;

        // 将 TransactionRequest 转换为 TransactionParameters
        let tx_params = TransactionParameters {
            to: tx_request.to,
            value: tx_request.value.unwrap_or(U256::zero()),
            gas: tx_request.gas.unwrap_or(U256::zero()),
            gas_price: Some(tx_request.gas_price.unwrap_or(U256::zero())),
            nonce: tx_request.nonce,
            data: tx_request.data.unwrap_or_else(Bytes::default),
            transaction_type: tx_request.transaction_type,
            access_list: tx_request.access_list,
            max_fee_per_gas: tx_request.max_fee_per_gas,
            max_priority_fee_per_gas: tx_request.max_priority_fee_per_gas,
            chain_id: chain_id_option,
        };

        // 签名交易
        let signed_tx = self
            .web3
            .accounts()
            .sign_transaction(tx_params, &key)
            .await
            .map_err(|e| DbErr::Custom(format!("签名交易失败: {}", e)))?;

        // 发送交易
        let tx_hash = self
            .web3
            .eth()
            .send_raw_transaction(signed_tx.raw_transaction)
            .await
            .map_err(|e| DbErr::Custom(format!("发送交易失败: {}", e)))?;

        Ok(tx_hash.to_string())
    }

    pub async fn check_balance(&self, account_address: &str, amount: U256) -> Result<bool, DbErr> {
        let balance = self
            .web3
            .eth()
            .balance(account_address.parse().unwrap(), None)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        if balance < amount {
            return Err(DbErr::Custom("Insufficient balance".to_string()));
        }

        Ok(true)
    }

    pub async fn process_transaction(
        &self,
        account_address: &str,
        amount: U256,
    ) -> Result<String, DbErr> {
        // 处理基础交易
        let tx_hash = self
            .send_transaction(account_address, amount)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        Ok(tx_hash.to_string())
    }

    pub async fn process_competition_transaction(
        &self,
        account_address: &str,
        amount: U256,
    ) -> Result<String, DbErr> {
        // 处理竞猜交易
        let tx_hash = self
            .send_competition_transaction(account_address, amount)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        Ok(tx_hash.to_string())
    }

    pub async fn process_regular_bet(
        &self,
        account_address: &str,
        amount: Decimal,
    ) -> Result<String, DbErr> {
        info!("Processing regular bet for account: {}, amount: {}", account_address, amount);

        // Amount is already in Wei, just convert to U256
        let amount_u256 = match U256::from_dec_str(&amount.to_string()) {
            Ok(a) => a,
            Err(e) => {
                error!("Failed to convert amount to U256: {}", e);
                return Err(DbErr::Custom(e.to_string()));
            }
        };
        info!("Converted amount to U256: {}", amount_u256);
        // First check contract balance
        let contract_address = self.contract_address.parse::<H160>()
            .map_err(|_| DbErr::Custom("Invalid contract address".to_string()))?;
        
        let contract_balance = self.web3.eth().balance(contract_address, None)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get contract balance: {}", e)))?;
        info!("Contract balance: {} wei", contract_balance);
        
        if contract_balance < amount_u256 {
            return Err(DbErr::Custom(format!(
                "Contract has insufficient balance. Required: {} wei, Available: {} wei",
                amount_u256, contract_balance
            )));
        }

        info!("Contract balance sufficient: {} wei", contract_balance);
        
        // let tx_hash = self.contract_transfer(account_address, amount_u256).await.map_err(|e| {
        //     error!("Failed to send transaction: {}", e);
        //     DbErr::Custom(e.to_string())
        // })?;
        info!("start process_regular_bet: account_address: {}", account_address);
        let tx_hash = self.send_transaction2(account_address, amount_u256).await.map_err(|e| {
            error!("Failed to send transaction: {}", e);
            DbErr::Custom(e.to_string())
        })?;

        info!("Transaction sent successfully, tx_hash: {}", tx_hash);

        Ok(tx_hash.to_string())
    }

    pub async fn process_winner_reward(
        &self,
        winner_address: &str,
        reward_amount: Decimal,
    ) -> Result<String, DbErr> {
        // 处理获奖奖励
        let amount_u256 = U256::from_dec_str(&reward_amount.to_string())
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let tx_hash = self
            .send_reward_transaction(winner_address, amount_u256)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        Ok(tx_hash.to_string())
    }

    // 内部辅助方法...

    async fn send_transaction(&self, to_address: &str, amount: U256) -> Result<String, DbErr> {
        // 确保地址是有效的
        let to_address = to_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的接收地址".to_string()))?;
        
        // 1. 解析地址
        let contract_address = self.contract_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的接收地址".to_string()))?;
        let owner_address: Address = self.owner_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的接收地址".to_string()))?;
        
        // 获取私钥和账户
        let private_key = self.get_private_key().await?;
        let key = create_key_from_private_key(&private_key)?;
        let from_address = Self::get_address_from_secret_key(&key);
        //let contract_address = self.contract_address.parse::<H160>();
        // 获取账户余额
        let balance = self.web3.eth().balance(contract_address, None)
            .await
            .map_err(|e| DbErr::Custom(format!("获取余额失败: {}", e)))?;
    
        // 获取 gas 价格
        let gas_price = self.web3.eth().gas_price()
            .await
            .map_err(|e| DbErr::Custom(format!("获取 gas 价格失败: {}", e)))?;
    
        // 计算所需总金额（交易金额 + gas费用）
        let gas_limit = U256::from(21000);
        let total_required = amount + (gas_price * gas_limit);
        
        // 检查余额是否足够
        if balance < total_required {
            return Err(DbErr::Custom(format!(
                "账户余额不足。需要 {} wei，但只有 {} wei",
                total_required, balance
            )));
        }
    
        // 构建交易参数
        let tx_params = TransactionParameters {
            to: Some(to_address),
            value: amount,
            gas: gas_limit,
            gas_price: Some(gas_price),
            nonce: None,
            data: Bytes::default(),
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            chain_id: Some(self.web3.eth().chain_id().await.unwrap().as_u64()),
        };
    
        // 签名并发送交易
        let signed_tx = self.web3.accounts()
            .sign_transaction(tx_params, &key)
            .await
            .map_err(|e| DbErr::Custom(format!("签名交易失败: {}", e)))?;
    
        let tx_hash = self.web3.eth()
            .send_raw_transaction(signed_tx.raw_transaction)
            .await
            .map_err(|e| DbErr::Custom(format!("发送交易失败: {}", e)))?;
    
        Ok(tx_hash.to_string())
    }

    async fn send_transaction2(&self, to_address: &str, amount: U256) -> Result<String, ServiceError> {
        // 1. 解析地址
        let contract_address = self.contract_address.parse::<H160>()
            .map_err(|_| ServiceError::InternalServerError("Invalid contract address".to_string()))?;
        
        let owner_address = self.owner_address.parse::<H160>()
            .map_err(|_| ServiceError::InternalServerError("Invalid owner address".to_string()))?;
        
        let to_address = to_address.parse::<H160>()
            .map_err(|_| ServiceError::InternalServerError("Invalid recipient address".to_string()))?;
        
        info!("start send_transaction2: to_address: {}", to_address);
        info!("start send_transaction2: contract_address: {}", contract_address);
        info!("start send_transaction2: owner_address: {}", owner_address);
        // 2. 检查合约余额
        let contract_balance = self.web3.eth().balance(contract_address, None).await
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to get contract balance: {}", e)))?;
        
        if contract_balance < amount {
            return Err(ServiceError::InsufficientFunds(format!(
                "Contract has insufficient funds. Required {} wei, but only has {} wei",
                amount, contract_balance
            )));
        }

        // 3. 获取 gas 估算和价格
        let gas_price = self.web3.eth().gas_price().await
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to get gas price: {}", e)))?;

        // 4. 创建合约实例并编码调用数据
        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address,
            include_bytes!("../../contracts/build/contracts/LuckGame.abi")
        ).map_err(|e| ServiceError::InternalServerError(format!("Failed to create contract: {}", e)))?;

        let data = contract.abi().function("transferFunds")
            .and_then(|function| function.encode_input(&[
                Token::Address(to_address),
                Token::Uint(amount),
            ]))
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to encode contract call: {}", e)))?;

        // 5. 估算 gas
        let gas_estimate = self.web3.eth().estimate_gas(
            CallRequest {
                from: Some(owner_address),    // 交易发起者的地址
                to: Some(contract_address),   // 要调用的智能合约地址
                gas: None,
                gas_price: Some(gas_price),
                value: None,
                data: Some(data.clone().into()), // 合约调用数据（包含了实际的转账信息）
                transaction_type: None,
                access_list: None,
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
            },
            None,
        ).await.map_err(|e| ServiceError::InternalServerError(format!("Failed to estimate gas: {}", e)))?;

        // 6. 检查 owner 账户余额是否足够支付 gas
        let gas_cost = gas_price.saturating_mul(gas_estimate);
        let owner_balance = self.web3.eth().balance(owner_address, None).await
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to get owner balance: {}", e)))?;

        if owner_balance < gas_cost {
            return Err(ServiceError::InsufficientFunds(format!(
                "Owner has insufficient funds for gas. Required {} wei, but only has {} wei",
                gas_cost, owner_balance
            )));
        }

        // 7. 获取 nonce
        let nonce = self.web3.eth().transaction_count(owner_address, None).await
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to get nonce: {}", e)))?;

            // 添加网络配置
        let chain_id = match self.web3.eth().chain_id().await {
            Ok(id) => id.as_u64(),
            Err(_) => 1337, // Ganache 默认值
        };
        // 8. 构建交易参数
        let tx_params = TransactionParameters {
            nonce: Some(nonce),
            to: Some(contract_address),  // 交易的目标地址（这里是智能合约的地址）
            value: U256::zero(),  // 随交易直接发送的 ETH 数量（这里是 0，因为我们通过合约函数转账）
            gas_price: Some(gas_price),
            gas: gas_estimate,
            data: Bytes::from(data),  // 合约调用数据（包含函数名和参数的编码）
            chain_id: Some(chain_id),  // Ganache, Mainnet
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
        };

        // 9. 使用 owner 的私钥签名交易
        let private_key = self.get_owner_private_key().await?;
        let key = SecretKey::from_str(&private_key)
            .map_err(|e| ServiceError::InternalServerError(
                format!("Failed to create secret key: {}", e)
            ))?;

        let signed = self.web3.accounts()
            .sign_transaction(tx_params, &key)
            .await
            .map_err(|e| ServiceError::InternalServerError(
                format!("Failed to sign transaction: {}", e)
            ))?;

        // 10. 发送交易
        let tx_hash = self.web3.eth()
            .send_raw_transaction(signed.raw_transaction)
            .await
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to send transaction: {}", e)))?;

        info!("Transaction details:");
        info!("  - From: {}", owner_address);
        info!("  - Contract: {}", contract_address);
        info!("  - To: {}", to_address);
        info!("  - Amount: {} wei", amount);
        info!("  - Hash: {}", tx_hash);
        info!("  - Gas price: {} wei", gas_price);
        info!("  - Gas estimate: {} units", gas_estimate);
        info!("  - Total gas cost: {} wei", gas_cost);
        info!("  - Nonce: {}", nonce);

        // 添加交易确认等待
        let receipt = self.wait_for_transaction_receipt(
            tx_hash,
            Duration::from_secs(300),  // 180秒超时
            Duration::from_secs(60)    // 每60秒检查一次
        ).await?;

        if let Some(receipt) = receipt {
            info!("Actual gas used: {} wei", receipt.gas_used.unwrap_or_default());
            info!("Gas price: {} wei", receipt.effective_gas_price.unwrap_or_default());
            if let (Some(gas_used), Some(gas_price)) = (receipt.gas_used, receipt.effective_gas_price) {
                let total_cost = gas_used.saturating_mul(gas_price);
                info!("Total transaction cost: {} wei", total_cost);
                // 记录到platform_transaction_gas数据库表中
                self.record_platform_transaction_gas(to_address, contract_address, total_cost, tx_hash.to_string()).await?;
            }
            
            if receipt.status == Some(U64::from(1)) {
                info!("Transaction confirmed: {:?}", receipt);
                Ok(tx_hash.to_string())
            } else {
                Err(ServiceError::TransactionFailed(
                    format!("Transaction failed: {:?}", receipt)
                ))
            }
        } else {
            info!("Transaction pending: {}", tx_hash);
            Ok(tx_hash.to_string())
        }

        //Ok(tx_hash.to_string())
    }

    // 获取 owner 私钥的辅助方法
    async fn get_owner_private_key(&self) -> Result<String, ServiceError> {
        // 从配置或环境变量获取私钥字符串
        let private_key_str = std::env::var("OWNER_PRIVATE_KEY").unwrap();
        //info!("Owner private key: {}", private_key_str);
        // 移除可能的 "0x" 前缀
        let clean_key = private_key_str.trim_start_matches("0x");
        
        // 验证私钥格式
        if clean_key.len() != 64 {
            return Err(ServiceError::InternalServerError(
                format!("Invalid private key length: expected 64 chars, got {}", clean_key.len())
            ));
        }

        // 验证是否为有效的十六进制字符串
        if !clean_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ServiceError::InternalServerError(
                "Invalid private key: contains non-hexadecimal characters".to_string()
            ));
        }

        Ok(clean_key.to_string())
    }

    // async fn send_transaction(&self, web3: &Web3<Http>, to: H160, amount: U256) -> Result<H256, ServiceError> {
    //     // 1. 获取当前 nonce
    //     let from = self.owner_address.parse::<H160>()
    //         .map_err(|_| ServiceError::InternalServerError("Invalid owner address".to_string()))?;
    // 
    //     let nonce = web3.eth().transaction_count(from, None).await
    //         .map_err(|e| ServiceError::InternalServerError(format!("Failed to get nonce: {}", e)))?;
    // 
    //     // 2. 获取当前 gas price
    //     let gas_price = web3.eth().gas_price().await
    //         .map_err(|e| ServiceError::InternalServerError(format!("Failed to get gas price: {}", e)))?;
    // 
    //     // 3. 构建交易参数
    //     let tx_params = TransactionParameters {
    //         nonce: Some(nonce),
    //         to: Some(to),
    //         value: amount,
    //         gas_price: Some(gas_price),
    //         gas: U256::from(67917),  // 使用估算的 gas 限制
    //         data: vec![].into(),     // 如果是合约调用，这里需要设置调用数据
    //         chain_id: Some(1),       // 主网 chain ID
    //         ..Default::default()
    //     };
    // 
    //     // 4. 获取私钥并创建签名器
    //     let private_key = self.get_private_key().await?;  // 从安全存储获取私钥
    //     let key = create_key_from_private_key(&private_key)?;
    //     // let key = web3::signing::SecretKey::from_slice(&hex::decode(private_key)?)
    //     //     .map_err(|e| ServiceError::InternalServerError(format!("Invalid private key: {}", e)))?;
    // 
    //     // 5. 签名交易
    //     let signed = web3.accounts().sign_transaction(tx_params, &key).await
    //         .map_err(|e| ServiceError::InternalServerError(format!("Failed to sign transaction: {}", e)))?;
    // 
    //     // 6. 发送已签名的交易
    //     let tx_hash = web3.eth().send_raw_transaction(signed.raw_transaction).await
    //         .map_err(|e| ServiceError::InternalServerError(format!("Failed to send transaction: {}", e)))?;
    // 
    //     Ok(tx_hash)
    // }

    async fn send_competition_transaction(&self, to_address: &str, amount: U256) -> Result<String, DbErr> {
        // 确保地址是有效的
        let to_address = to_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的接收地址".to_string()))?;

        // 获取当前 gas price
        let gas_price = self
            .web3
            .eth()
            .gas_price()
            .await
            .map_err(|e| DbErr::Custom(format!("获取 gas 价格失败: {}", e)))?;

        // 获取链 ID
        let chain_id = self
            .web3
            .eth()
            .chain_id()
            .await
            .map_err(|e| DbErr::Custom(format!("获取链 ID 失败: {}", e)))?;

        // 构建交易参数
        let tx_params = TransactionParameters {
            to: Some(to_address),
            value: amount,
            gas: U256::from(21000),  // 竞猜交易使用标准 gas 限制
            gas_price: Some(gas_price),
            nonce: None,
            data: Bytes::default(),
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            chain_id: Some(chain_id.as_u64()),
        };

        // 使用私钥签名交易
        let private_key = self.get_private_key().await?;
        let key = create_key_from_private_key(&private_key)?;

        // 签名并发送交易
        let signed_tx = self
            .web3
            .accounts()
            .sign_transaction(tx_params, &key)
            .await
            .map_err(|e| DbErr::Custom(format!("签名交易失败: {}", e)))?;

        let tx_hash = self
            .web3
            .eth()
            .send_raw_transaction(signed_tx.raw_transaction)
            .await
            .map_err(|e| DbErr::Custom(format!("发送交易失败: {}", e)))?;

        Ok(tx_hash.to_string())
    }

    async fn send_reward_transaction(&self, to_address: &str, amount: U256) -> Result<String, DbErr> {
        // 确保地址是有效的
        let to_address = to_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的接收地址".to_string()))?;

        // 获取当前 gas price
        let gas_price = self
            .web3
            .eth()
            .gas_price()
            .await
            .map_err(|e| DbErr::Custom(format!("获取 gas 价格失败: {}", e)))?;

        // 获取链 ID
        let chain_id = self
            .web3
            .eth()
            .chain_id()
            .await
            .map_err(|e| DbErr::Custom(format!("获取链 ID 失败: {}", e)))?;

        // 构建交易参数
        let tx_params = TransactionParameters {
            to: Some(to_address),
            value: amount,
            gas: U256::from(21000),  // 奖励发放使用标准 gas 限制
            gas_price: Some(gas_price),
            nonce: None,
            data: Bytes::default(),
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            chain_id: Some(chain_id.as_u64()),
        };

        // 使用私钥签名交易
        let private_key = self.get_private_key().await?;
        let key = create_key_from_private_key(&private_key)?;

        // 签名并发送交易
        let signed_tx = self
            .web3
            .accounts()
            .sign_transaction(tx_params, &key)
            .await
            .map_err(|e| DbErr::Custom(format!("签名交易失败: {}", e)))?;

        let tx_hash = self
            .web3
            .eth()
            .send_raw_transaction(signed_tx.raw_transaction)
            .await
            .map_err(|e| DbErr::Custom(format!("发送交易失败: {}", e)))?;

        Ok(tx_hash.to_string())
    }

    async fn get_private_key(&self) -> Result<String, DbErr> {
        // 1. 优先从环境变量获取
        if let Ok(key) = std::env::var("ETHEREUM_PRIVATE_KEY") {
            return Ok(key.trim_start_matches("0x").to_string());
        }
        
        // 2. 尝试从不同路径读取部署信息文件
        // let possible_paths = [
        //     "deploy-info.json",
        //     "./deploy-info.json",
        //     "../deploy-info.json",
        //     "./truffle-contract/deploy-info.json",
        //     "../truffle-contract/deploy-info.json",
        //     "../../truffle-contract/deploy-info.json"
        // ];
        // 
        // for path in possible_paths {
        //     if let Ok(content) = std::fs::read_to_string(path) {
        //         if let Ok(info) = serde_json::from_str::<serde_json::Value>(&content) {
        //             if let Some(key) = info["privateKey"].as_str() {
        //                 return Ok(key.trim_start_matches("0x").to_string());
        //             }
        //         }
        //     }
        // }
        // 
        // // 3. 如果都失败，试从配置文获取
        // if let Ok(config) = config::Config::builder()
        //     .add_source(config::File::with_name("config/development"))
        //     .build()
        // {
        //     if let Ok(key) = config.get_string("web3.private_key") {
        //         return Ok(key.trim_start_matches("0x").to_string());
        //     }
        // }
        // 
         Err(DbErr::Custom("无法找到有效的以太坊私钥".to_string()))
    }

    fn get_address_from_secret_key(secret_key: &SecretKey) -> H160 {
        let secp = Secp256k1::new();
        // 将 web3::signing::SecretKey 转换为字节
        let secret_key_bytes = secret_key.display_secret().to_string();
        let secret_key_bytes = hex::decode(secret_key_bytes)
            .expect("Invalid secret key hex");
        
        let secp_secret_key = secp256k1::SecretKey::from_slice(&secret_key_bytes)
            .expect("Invalid secret key");
        
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secp_secret_key);
        let public_key_bytes = public_key.serialize_uncompressed();
        let hash = keccak256(&public_key_bytes[1..]);
        H160::from_slice(&hash[12..])
    }

    pub async fn send_signed_transaction(&self, signed_tx: Bytes) -> Result<String, DbErr> {
        let tx_hash = self
            .web3
            .eth()
            .send_raw_transaction(signed_tx)
            .await
            .map_err(|e| DbErr::Custom(format!("发送交易失败: {}", e)))?;

        Ok(tx_hash.to_string())
    }

    pub async fn get_transaction_params(&self, from: &str, to: &str, amount: Decimal) 
        -> Result<TransactionParameters, DbErr> {
        // 确保地址是有效的
        let from_address = from
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的发送地址".to_string()))?;
        let to_address = to
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的接收地址".to_string()))?;

        // 将 Decimal 转换为 U256
        let amount_u256 = U256::from_dec_str(&(amount * Decimal::from_str("1e18").unwrap()).to_string())
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        // 获取当前 nonce
        let nonce = self
            .web3
            .eth()
            .transaction_count(from_address, None)
            .await
            .map_err(|e| DbErr::Custom(format!("获取 nonce 失败: {}", e)))?;

        // 获取当前 gas price
        let gas_price = self
            .web3
            .eth()
            .gas_price()
            .await
            .map_err(|e| DbErr::Custom(format!("获取 gas 价格失败: {}", e)))?;

        // 获取链 ID
        let chain_id = self
            .web3
            .eth()
            .chain_id()
            .await
            .map_err(|e| DbErr::Custom(format!("获取链 ID 失败: {}", e)))?;

        Ok(TransactionParameters {
            to: Some(to_address),
            value: amount_u256,
            gas: U256::from(21000),
            gas_price: Some(gas_price),
            nonce: Some(nonce),
            data: Bytes::default(),
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            chain_id: Some(chain_id.as_u64()),
        })
    }

    pub async fn get_bet_transaction_params(&self, from: &str, amount: Decimal) 
        -> Result<TransactionParameters, DbErr> {
        let from_address = from
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的发送地址".to_string()))?;
        
        let contract_address = self.contract_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("无效的合约地址".to_string()))?;

        let amount_u256 = U256::from_dec_str(&(amount * Decimal::from_str("1e18").unwrap()).to_string())
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let nonce = self.web3.eth()
            .transaction_count(from_address, None)
            .await
            .map_err(|e| DbErr::Custom(format!("获取 nonce 失败: {}", e)))?;

        let gas_price = self.web3.eth()
            .gas_price()
            .await
            .map_err(|e| DbErr::Custom(format!("获取 gas 价格失败: {}", e)))?;

        let chain_id = self.web3.eth()
            .chain_id()
            .await
            .map_err(|e| DbErr::Custom(format!("获取链 ID 失败: {}", e)))?;

        Ok(TransactionParameters {
            to: Some(contract_address),
            value: amount_u256,
            gas: U256::from(100000),
            gas_price: Some(gas_price),
            nonce: Some(nonce),
            data: Bytes::default(),
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            chain_id: Some(chain_id.as_u64()),
        })
    }

    pub async fn send_eth_transaction(
        &self,
        to_address: &str,
        amount: U256,
    ) -> Result<String, DbErr> {
        let to_address = to_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("Invalid recipient address".to_string()))?;

        let from_address = self.owner_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("Invalid owner address".to_string()))?;

        let params = self.build_transaction_params(&to_address, amount).await?;
        
        let tx_request = TransactionRequest {
            from: from_address,  // Using owner's address directly
            to: params.to,
            value: Some(params.value),
            gas: Some(params.gas),
            gas_price: params.gas_price,
            nonce: params.nonce,
            data: Some(params.data),
            transaction_type: params.transaction_type,
            access_list: params.access_list,
            max_fee_per_gas: params.max_fee_per_gas,
            max_priority_fee_per_gas: params.max_priority_fee_per_gas,
            condition: None,
        };

        let tx_hash = self.web3
            .eth()
            .send_transaction(tx_request)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to send transaction: {}", e)))?;

        Ok(tx_hash.to_string())
    }

    async fn build_transaction_params(
        &self,
        to_address: &H160,
        amount: U256,
    ) -> Result<TransactionParameters, DbErr> {
        // 获取 gas 价格
        let gas_price = self.web3.eth()
            .gas_price()
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get gas price: {}", e)))?;

        // 获取链 ID
        let chain_id = self.web3.eth()
            .chain_id()
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get chain id: {}", e)))?;

        // 获取发送方地址的 nonce
        let from_address = self.owner_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("Invalid owner address".to_string()))?;

        let nonce = self.web3.eth()
            .transaction_count(from_address, None)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get nonce: {}", e)))?;

        Ok(TransactionParameters {
            to: Some(*to_address),
            value: amount,
            gas: U256::from(21000),
            gas_price: Some(gas_price),
            nonce: Some(nonce),
            data: Bytes::default(),
            transaction_type: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            chain_id: Some(chain_id.as_u64()),
        })
    }

    // pub async fn contract_transfer(&self, account_address: &str, amount: U256) -> Result<String, ServiceError> {
    //     let web3_service = Web3Service::new(
    //         vec![
    //             //"http://localhost:7546".to_string(),
    //             "https://ethereum-rpc.publicnode.com".to_string(),
    //             // "https://eth.llamarpc.com".to_string(),
    //             // "https://rpc.ankr.com/eth".to_string(),
    //             // "https://ethereum.publicnode.com".to_string(),
    //             // "https://1rpc.io/eth".to_string(),
    //         ],
    //         self.contract_address.parse().unwrap()
    //     );
    // 
    //     let mut last_error = None;
    //     
    //     // 尝试所有 RPC 端点
    //     for rpc_url in web3_service.rpc_urls.iter() {
    //         log::info!("Trying to send transaction via RPC endpoint: {}", rpc_url);
    //         
    //         match self.try_contract_transfer(rpc_url, account_address, amount).await {
    //             Ok(tx_hash) => {
    //                 log::info!("Transaction sent successfully via {}: {}", rpc_url, tx_hash);
    //                 return Ok(tx_hash);
    //             }
    //             Err(e) => {
    //                 log::warn!("Failed to send transaction via {}: {}", rpc_url, e);
    //                 last_error = Some(e);
    //                 continue;
    //             }
    //         }
    //     }
    //     
    //     Err(last_error.unwrap_or_else(|| 
    //         ServiceError::InternalServerError("All RPC endpoints failed".to_string())
    //     ))
    // }

    pub async fn contract_transfer(&self, to_address: &str, amount: U256) -> Result<String, DbErr> {
        let to_address = to_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("Invalid recipient address".to_string()))?;

        let from_address = self.contract_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom("Invalid contract address".to_string()))?;

        // Use the owner address from config (should be 0xA2eA314813331737f17cC75559c859806f415415)
        let owner_address = self.owner_address
            .parse::<H160>()
            .map_err(|_| DbErr::Custom(format!("Invalid owner address: {}", self.owner_address)))?;

        info!("Contract address: {}", self.contract_address);
        info!("Owner address: {}", self.owner_address);
        info!("Sender address: {}", from_address);
        info!("Recipient address: {}", to_address);

        // Create contract instance
        info!("Creating contract instance");
        let contract = Contract::from_json(
            self.web3.eth(),
            from_address,
            include_bytes!("../../contracts/build/contracts/LuckGame.abi")
        ).map_err(|e| DbErr::Custom(format!("Failed to create contract: {}", e)))?;

        // 估算 gas 消耗
        let gas_estimate = contract.estimate_gas(
            "transferFunds",
            (to_address, amount),
            owner_address,
            Options::default(),
        )
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to estimate gas: {}", e)))?;

        info!("Estimated gas consumption: {} wei", gas_estimate);

        // Call transferFunds function with estimated gas
        info!("Calling transferFunds function");
        let tx_hash = contract.call(
            "transferFunds",
            (to_address, amount),
            owner_address,
            Options::with(|opt| {
                opt.gas = Some(gas_estimate);
            })
        )
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to call contract: {}", e)))?;

        // 获取交易收据以查看实际 gas 使用量
        let receipt = self.web3.eth().transaction_receipt(tx_hash)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get transaction receipt: {}", e)))?;

        if let Some(receipt) = receipt {
            info!("Actual gas used: {} wei", receipt.gas_used.unwrap_or_default());
            info!("Gas price: {} wei", receipt.effective_gas_price.unwrap_or_default());
            if let (Some(gas_used), Some(gas_price)) = (receipt.gas_used, receipt.effective_gas_price) {
                let total_cost = gas_used.saturating_mul(gas_price);
                info!("Total transaction cost: {} wei", total_cost);
                // 记录到platform_transaction_gas数据库表中
                self.record_platform_transaction_gas(to_address, from_address, total_cost, tx_hash.to_string()).await?;
            }
        }

        info!("TransferFunds function called");
        Ok(tx_hash.to_string())
    }


    async fn try_contract_transfer(&self, rpc_url: &str, account_address: &str, amount: U256) 
        -> Result<String, ServiceError> {
        let transport = web3::transports::Http::new(rpc_url)
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to create transport: {}", e)))?;
        
        let web3 = Web3::new(transport);
        
        // 设置超时
        let timeout_duration = std::time::Duration::from_secs(180);
        
        let tx_result = tokio::time::timeout(
            timeout_duration,
            self.execute_contract_transfer(&web3, account_address, amount)
        ).await
        .map_err(|e| ServiceError::InternalServerError(format!("Transaction timeout: {}", e)))??;
        
        Ok(tx_result)
    }

    async fn execute_contract_transfer(&self, web3: &Web3<Http>, account_address: &str, amount: U256) 
        -> Result<String, ServiceError> {
        // 原有的合约调用逻辑
        let to_address = account_address.parse::<H160>()
            .map_err(|_| ServiceError::BadRequest("Invalid recipient address".to_string()))?;

        let from_address = self.contract_address
            .parse::<H160>()
            .map_err(|_| ServiceError::BadRequest("Invalid contract address".to_string()))?;

        // Use the owner address from config (should be 0xA2eA314813331737f17cC75559c859806f415415)
        let owner_address = self.owner_address
            .parse::<H160>()
            .map_err(|_| ServiceError::BadRequest(format!("Invalid owner address: {}", self.owner_address)))?;

        info!("Contract address: {}", self.contract_address);
        info!("Owner address: {}", self.owner_address);
        info!("Sender address: {}", from_address);
        info!("Recipient address: {}", to_address);

        // Create contract instance
        info!("Creating contract instance");
        let contract = Contract::from_json(
            web3.eth(),
            from_address,
            include_bytes!("../../contracts/build/contracts/LuckGame.abi")
        ).map_err(|e| DbErr::Custom(format!("Failed to create contract: {}", e)))?;
        
        // 获取 nonce, gas_price 等
        let nonce = web3.eth().transaction_count(
            self.owner_address.parse().unwrap(),
            None
        ).await
        .map_err(|e| ServiceError::InternalServerError(format!("Failed to get nonce: {}", e)))?;
        
        let gas_price = web3.eth().gas_price().await
            .map_err(|e| ServiceError::InternalServerError(format!("Failed to get gas price: {}", e)))?;

        // 估算 gas 消耗
        let gas_estimate = contract.estimate_gas(
            "transferFunds",
            (to_address, amount),
            owner_address,
            Options::default(),
        )
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to estimate gas: {}", e)))?;
        info!("Estimated gas consumption: {} wei", gas_estimate);

        // Call transferFunds function with estimated gas
        info!("Calling transferFunds function");
        let tx_hash = contract.call(
            "transferFunds",
            (to_address, amount),
            owner_address,
            Options::with(|opt| {
                opt.gas = Some(gas_estimate);
            })
        )
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to call contract: {}", e)))?;

        // 获取交易收据以查看实际 gas 使用量
        let receipt = web3.eth().transaction_receipt(tx_hash)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get transaction receipt: {}", e)))?;

        if let Some(receipt) = receipt {
            info!("Actual gas used: {} wei", receipt.gas_used.unwrap_or_default());
            info!("Gas price: {} wei", receipt.effective_gas_price.unwrap_or_default());
            if let (Some(gas_used), Some(gas_price)) = (receipt.gas_used, receipt.effective_gas_price) {
                let total_cost = gas_used.saturating_mul(gas_price);
                info!("Total transaction cost: {} wei", total_cost);
                // 记录到platform_transaction_gas数据库表中
                self.record_platform_transaction_gas(to_address, from_address, total_cost, tx_hash.to_string()).await?;
            }
        }
        info!("TransferFunds function called");
        Ok(tx_hash.to_string())
        // 构建交易参数
        // let tx_params = TransactionParameters {
        //     to: Some(to_address),
        //     value: amount,
        //     gas: U256::from(21000),
        //     gas_price: Some(gas_price),
        //     nonce: Some(nonce),
        //     // ... 其他参数
        // };
        // 
        // // 签名并发送交易
        // let private_key = self.get_private_key().await?;
        // let key = create_key_from_private_key(&private_key)?;
        // 
        // let signed_tx = web3.accounts()
        //     .sign_transaction(tx_params, &key)
        //     .await
        //     .map_err(|e| ServiceError::InternalServerError(format!("Failed to sign transaction: {}", e)))?;
        // 
        // let tx_hash = web3.eth()
        //     .send_raw_transaction(signed_tx.raw_transaction)
        //     .await
        //     .map_err(|e| ServiceError::InternalServerError(format!("Failed to send transaction: {}", e)))?;
        // 
        // Ok(tx_hash.to_string())
    }

    // 记录到platform_transaction_gas数据库表中
    pub async fn record_platform_transaction_gas(
        &self,
        user_address: H160,
        from_address: H160,
        amount: U256,
        transaction_hash: String,
    ) -> Result<(), DbErr> {
        info!(
            "Recording platform transaction gas: user={}, from={}, amount={}, tx={}",
            user_address, from_address, amount, transaction_hash
        );

        let now = Utc::now();

        // 直接将 U256 转换为 Decimal，保持 Wei 值
        let decimal_amount = Decimal::from_str(&amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert amount to Decimal: {}", e)))?;

        // Create new platform transaction gas record
        let platform_transaction_gas = PlatformTransactionGasActiveModel {
            id: NotSet,
            user_address: Set(format!("{:?}", user_address)),
            from_address: Set(format!("{:?}", from_address)),
            amount_wei: Set(decimal_amount),  // 直接存储 Wei 值
            transaction_hash: Set(transaction_hash),
            created_at: Set(now),
            updated_at: Set(now),
        };

        // Insert the record into the database
        platform_transaction_gas.insert(&*self.db).await?;

        info!("Successfully recorded platform transaction gas");
        Ok(())
    }

    pub async fn record_commission(
        &self,
        user_address: String,
        from_address: String,
        commission: Decimal,
        transaction_hash: String,
    ) -> Result<(), DbErr> {
        info!(
            "Recording commission: user={}, from={}, amount={}, tx={}",
            user_address, from_address, commission, transaction_hash
        );

        let now = Utc::now();

        // Directly insert new record
        let commission = CommissionActiveModel {
            id: NotSet,
            user_address: Set(user_address),
            from_address: Set(from_address),
            commission: Set(commission),
            transaction_hash: Set(transaction_hash),
            created_at: Set(now),
            updated_at: Set(now),
        };

        commission.insert(&*self.db).await?;

        Ok(())
    }

    pub async fn get_total_prize_pool_from_platform_prize_pool(&self) -> Result<Decimal, DbErr> {
        info!("Getting total prize pool from platform_prize_pool");
        
        // Use the model to build and execute the query
        let total_amount: Option<Decimal> = PlatformPrizePool::find()
            .filter(platform_prize_pool_model::Column::UserAddress.eq(self.contract_address.clone()))
            .select_only()
            .column_as(
                sea_orm::sea_query::Expr::col(platform_prize_pool_model::Column::Amount).sum(),
                "total_amount"
            )
            .into_tuple()
            .one(&*self.db)
            .await?;

        let total_amount = total_amount.unwrap_or_else(|| Decimal::from(0));
        
        info!("Total prize pool amount: {} ETH", total_amount);

        Ok(total_amount)
    }

    pub async fn record_platform_funds_flow(
        &self,
        user_address: String,
        from_address: String,
        amount: Decimal,
        transaction_hash: String,
    ) -> Result<(), DbErr> {
        info!(
            "Recording platform funds flow: user={}, from={}, amount={}, tx={}",
            user_address, from_address, amount, transaction_hash
        );

        let now = Utc::now();

        // Create new platform funds flow record
        let platform_funds_flow = PlatformFundsFlowActiveModel {
            id: NotSet, // Auto-increment will handle this
            user_address: Set(user_address),
            from_address: Set(from_address),
            amount: Set(amount),
            transaction_hash: Set(transaction_hash),
            created_at: Set(now),
            updated_at: Set(now),
        };

        // Insert the record into the database
        platform_funds_flow.insert(&*self.db).await?;

        info!("Successfully recorded platform funds flow");
        Ok(())
    }

    pub async fn record_platform_prize_pool(
        &self,
        from_address: String,
        amount: Decimal,
        transaction_hash: String,
    ) -> Result<(), DbErr> {
        info!(
            "Recording platform prize pool: contract={}, from={}, amount={}, tx={}",
            self.contract_address, from_address, amount, transaction_hash
        );

        let now = Utc::now();

        // Create new platform prize pool record using contract address as user_address
        let platform_prize_pool = PlatformPrizePoolActiveModel {
            id: NotSet, // Auto-increment will handle this
            user_address: Set(self.contract_address.clone()), // Using contract address as user_address
            from_address: Set(from_address),                  // Using provided user_address as from_address
            amount: Set(amount),
            transaction_hash: Set(transaction_hash),
            created_at: Set(now),
            updated_at: Set(now),
        };

        // Insert the record into the database
        platform_prize_pool.insert(&*self.db).await?;

        info!("Successfully recorded platform prize pool");
        Ok(())
    }

    pub async fn record_winners(&self, winners: Vec<(String, String, Decimal, String)>) -> Result<(), DbErr> {
        info!("Recording lottery winners: {:?}", winners);
        let now = Utc::now();

        // Process each winner and create records
        for (user_address, luck_number, prize_amount, prize_grade) in winners {
            // Convert f64 to Decimal for database storage
            // let decimal_amount = Decimal::from_str(&prize_amount.to_string())
            //     .map_err(|e| DbErr::Custom(format!("Failed to convert prize amount to Decimal: {}", e)))?;

            let lottery_detail = LotteryDistributionDetailActiveModel {
                id: NotSet,
                user_address: Set(user_address.clone()),
                prize_amount: Set(prize_amount),
                luck_number: Set(luck_number),
                prize_grade: Set(prize_grade.clone()),
                created_at: Set(now),
                updated_at: Set(now),
            };

            // Insert the record into the database
            lottery_detail.insert(&*self.db).await?;

            info!(
                "Recorded lottery winner - Address: {}, Amount: {}, Grade: {}",
                user_address, prize_amount, prize_grade
            );
        }

        info!("Successfully recorded all lottery winners");
        Ok(())
    }

    pub async fn distribute_prize_to_winners(&self, winners: Vec<(String, String, Decimal, String)>) -> Result<(), DbErr> {
        info!("Starting prize distribution to winners");
        
        for (user_address, luck_number, prize_amount, prize_grade) in winners.iter() {
            info!(
                "Processing winner - Address: {}, Amount: {}, Grade: {}",
                user_address, prize_amount, prize_grade
            );
            
            // 移除可能的 "0x" 前缀并验证地址格式
            let clean_address = user_address.trim_start_matches("0x");
            if clean_address.len() != 40 || !clean_address.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(DbErr::Custom(format!("Invalid ethereum address format: {}", user_address)));
            }
            
            // 重新添加 "0x" 前缀
            let formatted_address = format!("0x{}", clean_address);
            
            // Convert prize amount from ETH to Wei (multiply by 10^18)
            let amount_wei = prize_amount * Decimal::from(10_u64.pow(18));
            info!("amount_wei: {}", amount_wei);
            
            info!("Formatted address: {}, amount_wei: {}", formatted_address.clone(), amount_wei);
            let fstr_address = formatted_address.as_str();
            match self.send_transaction2(
                fstr_address,
                U256::from_dec_str(&amount_wei.to_string().split('.').next().unwrap())
                    .map_err(|e| DbErr::Custom(format!("Failed to convert amount to U256: {}", e)))?)
            .await {
                Ok(tx_hash) => {
                    info!(
                        "Successfully sent prize to winner {} with transaction hash: {}",
                        formatted_address, tx_hash
                    );
                },
                Err(e) => {
                    error!(
                        "Failed to send prize to winner {}: {}",
                        formatted_address, e
                    );
                    return Err(DbErr::Custom(e.to_string()));
                }
            }
        }

        info!("Successfully completed prize distribution to all winners");
        sleep(Duration::from_secs(20)).await;
        Ok(())
    }

    async fn wait_for_transaction_receipt(
        &self,
        tx_hash: H256,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<Option<TransactionReceipt>, ServiceError> {
        let start_time = Instant::now();
        
        while start_time.elapsed() < timeout {
            match self.web3.eth().transaction_receipt(tx_hash).await {
                Ok(Some(receipt)) => {
                    return Ok(Some(receipt));
                }
                Ok(None) => {
                    info!("Transaction {} still pending, waiting...", tx_hash);
                    sleep(poll_interval).await;
                }
                Err(e) => {
                    error!("Error getting transaction receipt: {}", e);
                    return Err(ServiceError::InternalServerError(
                        format!("Failed to get transaction receipt: {}", e)
                    ));
                }
            }
        }
        
        Err(ServiceError::TransactionTimeout(format!(
            "Transaction {} not confirmed after {} seconds",
            tx_hash,
            timeout.as_secs()
        )))
    }
}

// 修改 create_key_from_private_key 函数以返回 Key 实例
fn create_key_from_private_key(private_key_hex: &str) -> Result<SecretKey, DbErr> {
    let private_key_bytes = hex::decode(private_key_hex)
        .map_err(|_| DbErr::Custom("无效的十六进制字符串".to_string()))?;

    SecretKey::from_slice(&private_key_bytes)
        .map_err(|_| DbErr::Custom("无效的私钥".to_string()))
}

