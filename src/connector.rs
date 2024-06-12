use std::{fmt::Display, future::Future};

use reqwest::{header::HeaderMap, Method};
use serde::Serialize;

use crate::{
    error::Result,
    filter::{Filter, FilterRule},
    pagination::{Pagination, PaginationRule, RequestPagination},
    prelude::ApiBuilder,
    query::Query,
    range::{Range, RangeRule},
    request::Request,
    request_url::RequestUrl,
    sort::{Sort, SortRule},
};

/// Authorization type to be used in the API
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AuthorizationType {
    #[default]
    None,
    // `username:password` into request headers
    // username and password are Base64 encoded
    // `Authorization: Basic bG9sOnNlY3VyZQ==`
    Basic(String),
    // `token` into request headers
    // `Authorization: Bearer <token>`
    Bearer(String),
    // `api_key` into request headers
    // `X-API-Key: 1234567890abcdef`
    ApiKey(String),
    // `access_token` into request headers
    // `Authorization: Bearer <access_token>`
    // `refresh_token` into request headers
    // `Authorization: Bearer <refresh_token>`
    OAuth2(String),
}

impl AuthorizationType {
    /// Set the Authorization header value for the request
    ///
    /// # Arguments
    /// * `headers` - A mutable reference to the request headers
    pub fn header_value(&self, headers: &mut HeaderMap) -> Result<()> {
        match self {
            AuthorizationType::None => {}
            AuthorizationType::ApiKey(_) => {
                headers.insert(
                    "X-API-Key",
                    reqwest::header::HeaderValue::from_str(&self.to_string())?,
                );
            }
            _ => {
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&self.to_string())?,
                );
            }
        }

        Ok(())
    }
}

impl Display for AuthorizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorizationType::Basic(token) => write!(f, "Basic {}", token),
            AuthorizationType::ApiKey(token) => write!(f, "{}", token),
            AuthorizationType::Bearer(token) | AuthorizationType::OAuth2(token) => {
                write!(f, "Bearer {}", token)
            }
            _ => panic!("TokenType::None is not allowed"),
        }
    }
}

/// Structure to build requests to the API
///
/// # Parameters
/// * `T` - Pagination type to be used in the request
///
/// # Attributes
/// * authorization - Authorization type to be used in the request
/// * endpoint - API endpoint to be used in the request
/// * pagination - Pagination type to be used in the request
#[derive(Debug, Clone)]
pub struct Api<
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
}

impl<P: Pagination, F: Filter, S: Sort, R: Range> Api<P, F, S, R>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.set_pagination(pagination);
        self
    }

    pub fn token(&self) -> String {
        self.authorization.to_string()
    }

    pub fn pattern_filter(mut self, pattern: impl ToString) -> Self {
        self.filter = self.filter.pattern(pattern);
        self
    }

    pub fn filter<T: IntoIterator>(mut self, property: impl ToString, value: T) -> Self
    where
        T::Item: ToString,
    {
        self.filter = self.filter.filter(property, value);
        self
    }

    pub fn filter_with<T: IntoIterator>(
        mut self,
        property: impl ToString,
        filter: impl ToString,
        value: T,
    ) -> Self
    where
        T::Item: ToString,
    {
        self.filter = self.filter.filter_with(property, filter, value);
        self
    }

    pub fn pattern_sort(mut self, pattern: impl ToString) -> Self {
        self.sort = self.sort.pattern(pattern);
        self
    }

    pub fn sort(mut self, property: impl ToString) -> Self {
        self.sort = self.sort.sort(property);
        self
    }

    pub fn sort_with(mut self, property: impl ToString, order: crate::sort::SortOrder) -> Self {
        self.sort = self.sort.sort_with(property, order);
        self
    }

    pub fn pattern_range(mut self, pattern: impl ToString) -> Self {
        self.range = self.range.pattern(pattern);
        self
    }

    pub fn range(
        mut self,
        property: impl ToString,
        min: impl ToString,
        max: impl ToString,
    ) -> Self {
        self.range = self.range.range(property, min, max);
        self
    }
}

impl<P: Pagination, F: Filter, S: Sort, R: Range> Connector<P, F, S, R> for Api<P, F, S, R>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    fn get(&self, route: impl ToString, query: Query) -> Result<Request<(), P, F, S, R>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::GET)
            .query(query);

        let request = Request::<(), P, F, S, R>::new(url.method.clone(), url, Some(headers), None)
            .pagination(self.pagination.pagination().clone())
            .set_filter(self.filter.clone())
            .set_sort(self.sort.clone())
            .set_range(self.range.clone());

        Ok(request)
    }

    fn post<B: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        body: Option<B>,
    ) -> Result<Request<B, P, F, S, R>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::POST)
            .query(query);

        let request = Request::<B, P, F, S, R>::new(url.method.clone(), url, Some(headers), body)
            .pagination(self.pagination.pagination().clone())
            .set_filter(self.filter.clone())
            .set_sort(self.sort.clone())
            .set_range(self.range.clone());

        Ok(request)
    }

    fn put<B: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        body: Option<B>,
    ) -> Result<Request<B, P, F, S, R>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::PUT)
            .query(query);

        let request = Request::<B, P, F, S, R>::new(url.method.clone(), url, Some(headers), body)
            .pagination(self.pagination.pagination().clone())
            .set_filter(self.filter.clone())
            .set_sort(self.sort.clone())
            .set_range(self.range.clone());

        Ok(request)
    }

    fn patch<B: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        body: Option<B>,
    ) -> Result<Request<B, P, F, S, R>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::PATCH)
            .query(query);

        let request = Request::<B, P, F, S, R>::new(url.method.clone(), url, Some(headers), body)
            .pagination(self.pagination.pagination().clone())
            .set_filter(self.filter.clone())
            .set_sort(self.sort.clone())
            .set_range(self.range.clone());

        Ok(request)
    }

    fn delete(&self, route: impl ToString, query: Query) -> Result<Request<(), P, F, S, R>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::DELETE)
            .query(query);

        let request = Request::<(), P, F, S, R>::new(url.method.clone(), url, Some(headers), None)
            .pagination(self.pagination.pagination().clone())
            .set_filter(self.filter.clone())
            .set_sort(self.sort.clone())
            .set_range(self.range.clone());

        Ok(request)
    }
}

/// Trait to implement on your connector structure
/// to allow the use of the `connect` method
pub trait Authorization<
    P: Pagination + Send = RequestPagination,
    F: Filter + Send = FilterRule,
    S: Sort + Send = SortRule,
    R: Range + Send = RangeRule,
> where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    fn connect(&self, url: &str) -> impl Future<Output = Result<Api<P, F, S, R>>> + Send {
        async move { Ok(ApiBuilder::new(url).build()) }
    }
}

/// Trait to implement on your connector structure
/// to allow the use of the `get`, `post`, `put`, `patch`, and `delete` methods
pub trait Connector<P: Pagination, F: Filter, S: Sort, R: Range>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    fn get(&self, route: impl ToString, query: Query) -> Result<Request<(), P, F, S, R>>;
    fn post<B: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        body: Option<B>,
    ) -> Result<Request<B, P, F, S, R>>;
    fn put<B: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        body: Option<B>,
    ) -> Result<Request<B, P, F, S, R>>;
    fn patch<B: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        body: Option<B>,
    ) -> Result<Request<B, P, F, S, R>>;
    fn delete(&self, route: impl ToString, query: Query) -> Result<Request<(), P, F, S, R>>;
}
