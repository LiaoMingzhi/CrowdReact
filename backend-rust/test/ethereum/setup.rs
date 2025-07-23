use std::process::Command;
use std::sync::Once;
use web3::Web3;
use web3::transports::Http;
use std::time::Duration;
use tokio::time::sleep;

static INIT: Once = Once::new();

pub struct TestEnv {
    pub web3: Web3<Http>,
    pub contract_address: String,
    pub owner_address: String,
    pub owner_private_key: String,
}

impl TestEnv {
    pub async fn new() -> Self {
        // 确保只初始化一次本地节点
        INIT.call_once(|| {
            // 启动本地 hardhat 节点
            Command::new("npx")
                .args(&["hardhat", "node"])
                .spawn()
                .expect("Failed to start hardhat node");

            // 等待节点启动
            std::thread::sleep(Duration::from_secs(2));

            // 部署合约
            Command::new("npx")
                .args(&["hardhat", "run", "scripts/deploy.ts", "--network", "localhost"])
                .output()
                .expect("Failed to deploy contract");
        });

        // 连接到本地节点
        let transport = Http::new("http://localhost:8545")
            .expect("Failed to connect to local node");
        let web3 = Web3::new(transport);

        // 读取部署信息
        let deploy_info = std::fs::read_to_string("deploy-info.json")
            .expect("Failed to read deploy info");
        let deploy_info: serde_json::Value = serde_json::from_str(&deploy_info)
            .expect("Failed to parse deploy info");

        Self {
            web3,
            contract_address: deploy_info["contract"].as_str().unwrap().to_string(),
            owner_address: deploy_info["owner"].as_str().unwrap().to_string(),
            owner_private_key: deploy_info["privateKey"].as_str().unwrap().to_string(),
        }
    }
}