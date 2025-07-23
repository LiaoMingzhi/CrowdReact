use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorTooManyRequests,
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};
use lazy_static::lazy_static;
use crate::utils::error::ServiceError;

// 定义限流配置
const WINDOW_SIZE: Duration = Duration::from_secs(60); // 1分钟时间窗口
const MAX_REQUESTS: usize = 100; // 每个时间窗口允许的最大请求数

lazy_static! {
    static ref RATE_LIMITER: Mutex<HashMap<String, (Instant, usize)>> = Mutex::new(HashMap::new());
}

pub struct RateLimiter;

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitMiddleware<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Box::pin(async move { Ok(RateLimitMiddleware { service }) })
    }
}

pub struct RateLimitMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req.peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut rate_limiter = RATE_LIMITER.lock().unwrap();
        let now = Instant::now();

        let is_allowed = if let Some((last_reset, count)) = rate_limiter.get_mut(&ip) {
            if now.duration_since(*last_reset) > WINDOW_SIZE {
                *last_reset = now;
                *count = 1;
                true
            } else if *count < MAX_REQUESTS {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            rate_limiter.insert(ip, (now, 1));
            true
        };

        if !is_allowed {
            return Box::pin(async move {
                Err(Error::from(ServiceError::TooManyRequests))
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
} 