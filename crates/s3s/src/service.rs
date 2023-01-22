use crate::auth::S3Auth;
use crate::error::{S3Error, S3Result};
use crate::http;
use crate::ops::S3;

use std::convert::Infallible;
use std::future::{ready, Ready};
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use hyper::service::Service;
use tracing::{debug, error};

pub struct S3Service {
    s3: Box<dyn S3>,
    auth: Option<Box<dyn S3Auth>>,
    full_body_limit: u64,
    base_domain: Option<String>,
}

impl S3Service {
    #[must_use]
    pub fn new(s3: Box<dyn S3>) -> Self {
        Self {
            s3,
            auth: None,
            full_body_limit: crate::http::DEFAULT_LENGTH_LIMIT,
            base_domain: None,
        }
    }

    pub fn set_auth(&mut self, auth: Box<dyn S3Auth>) {
        self.auth = Some(auth);
    }

    pub fn set_full_body_limit(&mut self, length_limit: u64) {
        self.full_body_limit = length_limit;
    }

    pub fn set_base_domain(&mut self, base_domain: impl Into<String>) {
        self.base_domain = Some(base_domain.into());
    }

    #[tracing::instrument(
        level = "debug",
        skip(self, req),
        fields(start_time=?time::OffsetDateTime::now_utc())
    )]
    pub async fn call(&self, mut req: http::Request) -> S3Result<http::Response> {
        debug!(?req);

        if self.full_body_limit > 0 {
            req.extensions_mut().insert(crate::http::LengthLimit(self.full_body_limit));
        }

        let s3 = &*self.s3;
        let auth = self.auth.as_deref();
        let base_domain = self.base_domain.as_deref();
        let result = crate::ops::call(&mut req, s3, auth, base_domain).await;

        match result {
            Ok(ref res) => debug!(?res),
            Err(ref err) => error!(?err),
        }

        result
    }

    #[must_use]
    pub fn into_shared(self) -> SharedS3Service {
        SharedS3Service(Arc::new(self))
    }
}

#[derive(Clone)]
pub struct SharedS3Service(Arc<S3Service>);

impl SharedS3Service {
    #[must_use]
    pub fn into_make_service(self) -> MakeService<Self> {
        MakeService(self)
    }
}

// TODO(blocking): GAT?
// See https://github.com/tower-rs/tower/issues/636
impl Service<http::Request> for SharedS3Service {
    type Response = http::Response;

    type Error = S3Error;

    type Future = BoxFuture<'static, S3Result<http::Response>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(())) // ASK: back pressure?
    }

    fn call(&mut self, req: http::Request) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move { service.call(req).await })
    }
}

#[derive(Clone)]
pub struct MakeService<S>(S);

impl<T, S: Clone> Service<T> for MakeService<S> {
    type Response = S;

    type Error = Infallible;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        ready(Ok(self.0.clone()))
    }
}