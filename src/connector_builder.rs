use std::sync::{Arc, RwLock};

use crate::{
    connector::{Api, AuthorizationType},
    filter::{Filter, FilterRule},
    pagination::{Pagination, PaginationRule, RequestPagination},
    query::Query,
    range::{Range, RangeRule},
    rate_limiter::{RateLimiter, TimePeriod},
    sort::{Sort, SortRule},
};

/// Builder to create an API connector
///
/// # Parameters
/// * `P` - Pagination rule to be used in the API
/// * `F` - Filter rule to be used in the API
/// * `S` - Sort rule to be used in the API
/// * `R` - Range rule to be used in the API
///
/// # Example
/// ```rust,ignore
/// let api = ApiBuilder::<MyRequestPagination, MyFilterRule>::new("https://api.example.com")
///    .bearer("token")
///    .pagination(PaginationRule::Fixed(10))
///    .filter(MyFilterRule::default().pattern("property[filter]"))
///    .build();
/// ```
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
    pub(crate) force_limit: Option<u8>,
}

impl<P: Pagination, F: Filter, S: Sort, R: Range> ApiBuilder<P, F, S, R>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    /// Create a new API builder
    ///
    /// # Attributes
    /// * authorization - AuthorizationType::None
    /// * endpoint - The API endpoint
    /// * pagination - P::default()
    /// * filter - F::default()
    /// * sort - S::default()
    /// * range - R::default()
    /// * rate_limiter - RateLimiter::new(1, TimePeriod::Second)
    /// * force_limit - None
    pub fn new(endpoint: impl ToString) -> Self {
        Self {
            authorization: AuthorizationType::None,
            endpoint: endpoint.to_string(),
            pagination: P::default(),
            filter: F::default(),
            sort: S::default(),
            range: R::default(),
            rate_limiter: RateLimiter::new(1, TimePeriod::Second),
            force_limit: None,
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

    pub fn oidc(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::OIDC(token.to_string());
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

    pub fn force_limit(mut self, limit: u8) -> Self {
        self.force_limit = Some(limit);
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
            force_limit: self.force_limit,
        }
    }
}
