use reqwest::{header::HeaderMap, Method};
use serde::Serialize;

use crate::{
    filter::{Filter, FilterRule},
    pagination::{Pagination, RequestPagination},
    prelude::PaginationRule,
    request::Request,
    request_url::RequestUrl,
    sort::{Sort, SortRule},
};

/// Builder to create a request
#[derive(Debug, Clone)]
pub struct RequestBuilder<
    B: Serialize + Clone = (),
    P: Pagination = RequestPagination,
    F: Filter = FilterRule,
    S: Sort = SortRule,
> {
    pub(crate) method: Method,
    pub(crate) request_url: RequestUrl,
    pub(crate) headers: Option<HeaderMap>,
    pub(crate) body: Option<B>,
    pub(crate) pagination: P,
    pub(crate) filter: F,
    pub(crate) sort: S,
}

impl<B: Serialize + Clone, P: Pagination, F: Filter, S: Sort> RequestBuilder<B, P, F, S> {
    pub fn new(request_url: RequestUrl) -> Self {
        Self {
            method: Method::GET,
            request_url,
            headers: None,
            body: None,
            pagination: P::default(),
            filter: F::default(),
            sort: S::default(),
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

    pub fn body(mut self, body: B) -> Self {
        self.body = Some(body);
        self
    }

    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.pagination(pagination);
        self
    }

    pub fn filter(mut self, filter: F) -> Self {
        self.filter = filter;
        self
    }

    pub fn sort(mut self, sort: S) -> Self {
        self.sort = sort;
        self
    }

    pub fn build(self) -> Request<B, P, F, S> {
        Request {
            method: self.method,
            request_url: self.request_url,
            headers: self.headers,
            body: self.body,
            pagination: self.pagination,
            filter: self.filter,
            sort: self.sort,
        }
    }
}
