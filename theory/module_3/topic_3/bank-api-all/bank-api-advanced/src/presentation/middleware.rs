use std::future::{ready, Ready};
use std::task::{Context, Poll};
use std::time::Instant;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdService { service }))
    }
}

pub struct RequestIdService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        req.extensions_mut().insert(RequestId(request_id.clone()));

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            res.response_mut().headers_mut().insert(
                HeaderName::from_static("x-request-id"),
                HeaderValue::from_str(&request_id).unwrap(),
            );
            Ok(res)
        })
    }
}

pub struct TimingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TimingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TimingService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TimingService { service }))
    }
}

pub struct TimingService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TimingService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_owned();
        let request_id = req
            .extensions()
            .get::<RequestId>()
            .map(|rid| rid.0.clone());

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut response = fut.await?;
            let duration = start.elapsed();
            if let Some(ref rid) = request_id {
                info!(
                    request_id = %rid,
                    method = %method,
                    path = %path,
                    status = response.status().as_u16(),
                    duration_ms = duration.as_millis(),
                    "request completed"
                );
            } else {
                info!(
                    method = %method,
                    path = %path,
                    status = response.status().as_u16(),
                    duration_ms = duration.as_millis(),
                    "request completed"
                );
            }

            let timing_header_value =
                format!("app;dur={}", duration.as_millis());
            if let Ok(value) = HeaderValue::from_str(&timing_header_value) {
                response
                    .response_mut()
                    .headers_mut()
                    .insert(HeaderName::from_static("server-timing"), value);
            }

            Ok(response)
        })
    }
}

