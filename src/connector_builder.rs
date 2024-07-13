use std::sync::{Arc, RwLock};

use crate::{
    connector::{Api, AuthorizationType},
    filter::{Filter, FilterRule},
    pagination::{Pagination, PaginationRule, RequestPagination},
    prelude::Query,
    range::{Range, RangeRule},
    rate_limiter::{RateLimiter, TimePeriod},
    sort::{Sort, SortRule},
};

#[derive(Debug, Clone)]
pub struct ApiBuilder<
    P: Pagination = RequestPagination,
    F: Filter = FilterRule,
    S: Sort = SortRule,
    R: Range = RangeRule,
> where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    pub(crate) authorization: AuthorizationType,
    pub(crate) endpoint: String,
    pub(crate) pagination: P,
    pub(crate) filter: F,
    pub(crate) sort: S,
    pub(crate) range: R,
    pub(crate) rate_limiter: RateLimiter,
}

impl<P: Pagination, F: Filter, S: Sort, R: Range> ApiBuilder<P, F, S, R>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    pub fn new(endpoint: impl ToString) -> Self {
        Self {
            authorization: AuthorizationType::None,
            endpoint: endpoint.to_string(),
            pagination: P::default(),
            filter: F::default(),
            sort: S::default(),
            range: R::default(),
            rate_limiter: RateLimiter::new(1, TimePeriod::Second),
        }
    }

    pub fn endpoint(mut self, endpoint: impl ToString) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    pub fn bearer(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::Bearer(token.to_string());
        self
    }

    pub fn oauth2(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::OAuth2(token.to_string());
        self
    }

    pub fn basic(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::Basic(token.to_string());
        self
    }

    pub fn api_key(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::ApiKey(token.to_string());
        self
    }

    pub fn keycloak(mut self, auth_type: AuthorizationType) -> Self {
        self.authorization = AuthorizationType::Keycloak(Box::new(auth_type));
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

    pub fn limit(mut self, limit: u32) -> Self {
        self.rate_limiter.limit = limit;
        self
    }

    pub fn limit_period(mut self, period: TimePeriod) -> Self {
        self.rate_limiter.period = period;
        self
    }

    pub fn build(self) -> Api<P, F, S, R> {
        Api {
            authorization: self.authorization,
            endpoint: self.endpoint,
            pagination: self.pagination,
            filter: self.filter,
            sort: self.sort,
            range: self.range,
            rate_limit: Arc::new(RwLock::new(self.rate_limiter)),
        }
    }
}
