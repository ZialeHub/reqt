use reqwest::{header::HeaderMap, Method};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use crate::{
    filter::{Filter, FilterRule},
    pagination::{Pagination, PaginationRule, RequestPagination},
    query::Query,
    range::{Range, RangeRule},
    rate_limiter::RateLimiter,
    request::Request,
    request_url::RequestUrl,
    sort::{Sort, SortRule},
};

/// Builder to create a request
#[derive(Debug, Clone)]
pub struct RequestBuilder<
    X: Deserialize<'static> = (),
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
    pub(crate) rate_limiter: Arc<RwLock<RateLimiter>>,
    pub(crate) force_limit: Option<u8>,
    pub(crate) _phantom: std::marker::PhantomData<X>,
}

impl<
        X: Deserialize<'static>,
        B: Serialize + Clone,
        P: Pagination,
        F: Filter,
        S: Sort,
        R: Range,
    > RequestBuilder<X, B, P, F, S, R>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    /// Create a new request builder
    ///
    /// # Attributes
    /// * method - Method::GET
    /// * request_url - The URL to request
    /// * headers - None
    /// * body - None
    /// * pagination - P::default()
    /// * filter - F::default()
    /// * sort - S::default()
    /// * range - R::default()
    /// * rate_limiter - The rate limiter to use
    /// * force_limit - None
    pub fn new(request_url: RequestUrl, rate_limiter: Arc<RwLock<RateLimiter>>) -> Self {
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
            force_limit: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the method of the request
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set the headers of the request
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Set the body of the request
    pub fn body(mut self, body: B) -> Self {
        self.body = Some(body);
        self
    }

    /// Set the pagination of the request (Overrides the pagination from the connector)
    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.set_pagination(pagination);
        self
    }

    /// Set the filter of the request
    pub fn filter(mut self, filter: F) -> Self {
        self.filter = filter;
        self
    }

    /// Set the sort of the request
    pub fn sort(mut self, sort: S) -> Self {
        self.sort = sort;
        self
    }

    /// Set the range of the request
    pub fn range(mut self, range: R) -> Self {
        self.range = range;
        self
    }

    /// Set the number of retry attempts on 429 responses
    pub fn force_limit(mut self, limit: Option<u8>) -> Self {
        self.force_limit = limit;
        self
    }

    pub fn build(self) -> Request<X, B, P, F, S, R> {
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
            force_limit: self.force_limit,
            _phantom: self._phantom,
        }
    }
}
