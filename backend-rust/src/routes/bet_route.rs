use actix_web::web;
use crate::handlers::bet_handler;
use log::{info, debug, trace};

pub fn bet_config(cfg: &mut web::ServiceConfig) {
    info!("Configuring bet routes");
    cfg.service(
        web::scope("/bet")
            .route("/transaction-params", web::get().to(bet_handler::get_transaction_params))
            .route("/place_bet", web::post().to(bet_handler::place_bet))
            .route("/commission", web::get().to(bet_handler::get_commission_details))
            .route("/record", web::post().to(bet_handler::record_bet))
            .route("/agents", web::get().to(bet_handler::get_agents))
            .route("/agent_details", web::get().to(bet_handler::get_agent_details))
            .route("/history", web::get().to(bet_handler::get_bet_history))
            .route("/datetime_week", web::get().to(bet_handler::get_datetime_week))
            
    ).service(
        web::scope("/prize")
            .route("/pool", web::get().to(bet_handler::get_prize_pool_expectation))
            .route("/competition_lottery", web::get().to(bet_handler::get_competition_lottery_info))
    );
    info!("Bet routes configured successfully");
    debug!("Registered routes:");
    debug!(" - GET /bet/transaction-params");
    debug!(" - POST /bet/place_bet");
    debug!(" - GET /bet/info");
    debug!(" - GET /bet/commission");
    debug!(" - POST /bet/record");
    debug!(" - GET /bet/agents");
    debug!(" - GET /bet/history");
    trace!("Route configuration details:");
    trace!(" - Base path: /bet");
    trace!(" - Handler: get_transaction_params, place_bet, get_bet_info, get_commission_income, record_bet, get_agents");
    trace!(" - Methods: GET, POST");
    trace!(" - Full paths: /bet/transaction-params, /bet/place_bet, /bet/info, /bet/commission, /bet/record, /bet/agents");
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::config::config::Config as AppConfig;
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_transaction_params() {
        let config = Arc::new(
            AppConfig::from_env().expect("Failed to load config")
        );
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .configure(bet_config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/bet/transaction-params?from=0x123&amount=0.001")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
