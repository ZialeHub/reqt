use reqwest::{header::HeaderMap, Method};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use crate::{
    filter::{Filter, FilterRule},
    pagination::{Pagination, RequestPagination},
    prelude::{PaginationRule, Query},
    range::{Range, RangeRule},
    rate_limiter::RateLimiter,
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
    R: Range = RangeRule,
> where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    pub(crate) method: Method,
    pub(crate) request_url: RequestUrl,
    pub(crate) headers: Option<HeaderMap>,
    pub(crate) body: Option<B>,
    pub(crate) pagination: P,
    pub(crate) filter: F,
    pub(crate) sort: S,
    pub(crate) range: R,
    pub(crate) rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl<B: Serialize + Clone, P: Pagination, F: Filter, S: Sort, R: Range>
    RequestBuilder<B, P, F, S, R>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    pub fn new(request_url: RequestUrl, rate_limiter: Arc<Mutex<RateLimiter>>) -> Self {
        Self {
            method: Method::GET,
            request_url,
            headers: None,
            body: None,
            pagination: P::default(),
            filter: F::default(),
            sort: S::default(),
            range: R::default(),
            rate_limiter,
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
        self.pagination = self.pagination.set_pagination(pagination);
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

    pub fn range(mut self, range: R) -> Self {
        self.range = range;
        self
    }

    pub fn build(self) -> Request<B, P, F, S, R> {
        Request {
            method: self.method,
            request_url: self.request_url,
            headers: self.headers,
            body: self.body,
            pagination: self.pagination,
            filter: self.filter,
            sort: self.sort,
            range: self.range,
            rate_limiter: self.rate_limiter,
        }
    }
}
