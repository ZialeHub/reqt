use reqwest::{header::HeaderMap, Method};
use serde::Serialize;

use crate::{
    pagination::{Pagination, RequestPagination},
    request::Request,
    request_url::RequestUrl,
};

/// Builder to create a request
#[derive(Debug, Clone)]
pub struct RequestBuilder<P: Serialize + Clone = (), U: Pagination = RequestPagination> {
    pub(crate) method: Method,
    pub(crate) request_url: RequestUrl,
    pub(crate) headers: Option<HeaderMap>,
    pub(crate) payload: Option<P>,
    pub(crate) pagination: U,
}

impl<P: Serialize + Clone, U: Pagination> RequestBuilder<P, U> {
    pub fn new(request_url: RequestUrl, pagination: U) -> Self {
        Self {
            method: Method::GET,
            request_url,
            headers: None,
            payload: None,
            pagination,
        }
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn payload(mut self, payload: P) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn build(self) -> Request<P, U> {
        Request {
            method: self.method,
            request_url: self.request_url,
            headers: self.headers,
            payload: self.payload,
            pagination: self.pagination,
        }
    }
}
