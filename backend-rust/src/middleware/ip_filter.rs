use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, body::{BoxBody, EitherBody},
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use crate::services::geoip_service::GeoIpService;
use log::{info, warn, error};
use serde_json::json;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct IpFilter {
    geoip_service: GeoIpService,
    blocked_countries: Vec<String>,
}

impl IpFilter {
    pub fn new(db_path: &str, blocked_countries: Vec<String>) -> Self {
        let geoip_service = GeoIpService::new(db_path)
            .expect("Failed to initialize GeoIP service");
            
        Self {
            geoip_service,
            blocked_countries,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for IpFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Transform = IpFilterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(IpFilterMiddleware {
            service,
            blocked_countries: self.blocked_countries.clone(),
            geoip_service: self.geoip_service.clone(),
        }))
    }
}

#[derive(Clone)]
pub struct IpFilterMiddleware<S> {
    service: S,
    geoip_service: GeoIpService,
    blocked_countries: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for IpFilterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let blocked_countries = self.blocked_countries.clone();
        let geoip_service = self.geoip_service.clone();

        Box::pin(async move {
            let ip = req.connection_info().realip_remote_addr().map(String::from);
            
            if let Some(ip) = ip {
                match geoip_service.get_country_code(&ip) {
                    Ok(Some(country_code)) if blocked_countries.contains(&country_code) => {
                        warn!("Blocking request from restricted country: {}", country_code);
                        let res = HttpResponse::Forbidden()
                            .json(json!({
                                "error": "Access denied from your country"
                            }));
                        
                        let (request, _) = req.into_parts();
                        return Ok(ServiceResponse::new(request, res).map_into_left_body());
                    }
                    Err(e) => {
                        error!("Failed to check IP location: {}", e);
                    }
                    _ => {}
                }
            }

            let res = service.call(req).await?;
            Ok(res.map_into_right_body())
        })
    }
} 