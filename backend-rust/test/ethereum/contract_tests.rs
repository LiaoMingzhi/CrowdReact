use super::setup::TestEnv;
use web3::types::{H160, U256};
use web3::contract::{Contract, Options};
use std::str::FromStr;

#[tokio::test]
async fn test_place_bet() {
    let env = TestEnv::new().await;
    let contract = Contract::from_json(
        env.web3.eth(),
        H160::from_str(&env.contract_address).unwrap(),
        include_bytes!("../../contracts/artifacts/LuckGame.json")
    ).unwrap();

    // 测试下注
    let result = contract.call(
        "placeBet",
        (U256::from(42),),
        H160::from_str(&env.owner_address).unwrap(),
        Options::with(|opt| {
            opt.value = Some(U256::exp10(16)); // 0.01 ETH
            opt.gas = Some(U256::from(100000));
        })
    ).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_commission() {
    let env = TestEnv::new().await;
    let contract = Contract::from_json(
        env.web3.eth(),
        H160::from_str(&env.contract_address).unwrap(),
        include_bytes!("../../contracts/artifacts/LuckGame.json")
    ).unwrap();

    // 添加代理
    let result = contract.call(
        "addAgent",
        (H160::from_str("0x1234...").unwrap(),),
        H160::from_str(&env.owner_address).unwrap(),
        Options::with(|opt| {
            opt.gas = Some(U256::from(100000));
        })
    ).await;

    assert!(result.is_ok());
}