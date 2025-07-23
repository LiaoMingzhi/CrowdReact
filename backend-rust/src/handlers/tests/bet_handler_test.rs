use super::*;
use actix_web::{test, App};
use serde_json::json;
use std::env;
use web3::signing::SecretKey;

// 创建测试用的数据库连接
async fn create_test_db() -> DatabaseConnection {
    Database::connect("postgres://tests:tests@localhost:5432/test_db")
        .await
        .expect("Failed to create tests database connection")
}

// 创建测试应用
async fn create_test_app() -> impl actix_web::dev::Service<actix_http::Request> {
    // 设置测试环境变量
    let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    env::set_var("ETHEREUM_PRIVATE_KEY", private_key);
    env::set_var("CONTRACT_ADDRESS", "0xContractAddress");
    env::set_var("OWNER_ADDRESS", "0xOwnerAddress");
    
    let db = Arc::new(create_test_db().await);
    let web3 = create_mock_web3();
    
    let amount_service = Arc::new(AmountService::new(
        Arc::clone(&db),
        web3,
        env::var("CONTRACT_ADDRESS").unwrap(),
        env::var("OWNER_ADDRESS").unwrap(),
    ));
    
    let buy_luck_number_service = Arc::new(BuyLuckNumberService::new(
        Arc::clone(&db),
        Arc::clone(&amount_service)
    ));
    
    let week_action_service = Arc::new(WeekActionService::new(
        Arc::clone(&db),
        buy_luck_number_service,
        amount_service
    ));

    App::new()
        .app_data(web::Data::new(db))
        .app_data(web::Data::new(week_action_service))
        .service(web::resource("/bet/place_bet").route(web::post().to(place_bet)))
}

#[actix_web::test]
async fn test_get_user_info() {
    let app = create_test_app().await;
    let req = test::TestRequest::get()
        .uri("/bet/user_info?account_address=0x123")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: UserInfoResponse = test::read_body_json(resp).await;
    assert_eq!(body.account_address, "0x123");
}

#[actix_web::test]
async fn test_get_bet_info() {
    let app = create_test_app().await;
    let req = test::TestRequest::get()
        .uri("/bet/bet_info?account_address=0x123")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: BetInfoResponse = test::read_body_json(resp).await;
    assert_eq!(body.account_address, "0x123");
    assert!(!body.purchase_numbers.is_empty());
}

#[actix_web::test]
async fn test_get_agent_info() {
    let app = create_test_app().await;
    let req = test::TestRequest::get()
        .uri("/bet/agent_info?account_address=0x123")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: AgentInfoResponse = test::read_body_json(resp).await;
    assert!(!body.lower_agents.is_empty());
}

#[actix_web::test]
async fn test_get_commission_income() {
    let app = create_test_app().await;
    let req = test::TestRequest::get()
        .uri("/bet/commission_income?account_address=0x123")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: CommissionIncomeResponse = test::read_body_json(resp).await;
    assert_eq!(body.address, "0x123");
    assert!(body.amount > 0.0);
}

#[actix_web::test]
async fn test_place_bet() {
    let app = create_test_app().await;
    let bet_request = json!({
        "account_address": "0x123",
        "amount": "1.0",
        "transaction_hash": "0x71d0d5d269f1089f2d5ab686ac4d1735314db8c5eee9623b368484014c546a1f",
        "block_number": 12345,
        "block_timestamp": 1634567890
    });

    let req = test::TestRequest::post()
        .uri("/bet/place_bet")
        .set_json(&bet_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_current_time() {
    let app = create_test_app().await;
    let req = test::TestRequest::get()
        .uri("/bet/current_time")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Current time:"));
}

// 错误情况测试
#[actix_web::test]
async fn test_place_bet_invalid_amount() {
    let app = create_test_app().await;
    let bet_request = json!({
        "account_address": "0x123",
        "mnemonic": "test_mnemonic",
        "signature": "test_signature",
        "amount": -100.0  // 无效金额
    });

    let req = test::TestRequest::post()
        .uri("/bet/place_bet")
        .set_json(&bet_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_get_user_info_invalid_address() {
    let app = create_test_app().await;
    let req = test::TestRequest::get()
        .uri("/bet/user_info?account_address=invalid")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

// 辅助函数测试
#[test]
fn test_response_structs() {
    let user_info = UserInfoResponse {
        account_address: "0x123".to_string(),
        mnemonic: "tests".to_string(),
        signature: "tests".to_string(),
        role_type: "user".to_string(),
    };
    assert_eq!(user_info.account_address, "0x123");

    let bet_info = BetInfoResponse {
        purchase_amount: 100.0,
        account_address: "0x123".to_string(),
        purchase_numbers: vec!["123".to_string()],
        purchase_time: "2023-10-17T12:00:00Z".to_string(),
    };
    assert_eq!(bet_info.purchase_amount, 100.0);
}

#[tokio::test]
async fn test_bet_status_update() {
    let app = create_test_app().await;
    let db = Arc::new(create_test_db().await);
    
    // 创建测试投注记录
    let tx_hash = "0x123456789";
    let bet_record = bet_records::ActiveModel {
        account_address: Set("0xtest".to_string()),
        amount: Set("1.0".to_string()),
        transaction_hash: Set(tx_hash.to_string()),
        status: Set(BetStatus::Pending.to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };
    
    let record = bet_record.insert(&*db).await.unwrap();
    
    // 测试状态更新
    let result = update_bet_status(&db, tx_hash, BetStatus::Confirmed).await;
    assert!(result.is_ok());
    
    // 验证状态已更新
    let updated_record = BetRecords::find_by_id(record.id)
        .one(&*db)
        .await
        .unwrap()
        .unwrap();
    
    assert_eq!(updated_record.status, BetStatus::Confirmed.to_string());
}