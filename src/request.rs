use reqwest::{Client, Method, StatusCode, Url, header::HeaderMap};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    sync::{Arc, RwLock},
};

use crate::{
    error::{ApiError, Result},
    filter::{Filter, FilterRule},
    pagination::{Pagination, PaginationRule, RequestPagination},
    query::Query,
    range::{Range, RangeRule},
    rate_limiter::RateLimiter,
    request_url::RequestUrl,
    sort::{Sort, SortOrder, SortRule},
};

/// Structure to send requests to the API
///
/// # Parameters
/// * `P` - body type to be used in the request
/// * `U` - Pagination type to be used in the request
///
/// # Attributes
/// * method - HTTP method to be used in the request
/// * request_url - URL to be used in the request
/// * headers - Headers to be used in the request
/// * body - body to be used in the request
/// * pagination - Pagination type to be used in the request
#[derive(Debug, Clone)]
pub struct Request<
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
    X: for<'de> Deserialize<'de> + Serialize + Send + 'static,
    B: Serialize + DeserializeOwned + Clone + Sync + Send + 'static + Unpin,
    P: Pagination + Sync + Send + 'static + Unpin,
    F: Filter + Sync + Send + 'static + Unpin,
    S: Sort + Sync + Send + 'static + Unpin,
    R: Range + Sync + Send + 'static + Unpin,
> IntoFuture for Request<X, B, P, F, S, R>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    type Output = Result<X>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move { self.send::<X>().await })
    }
}

impl<X: Deserialize<'static>, B: Serialize + Clone, P: Pagination, F: Filter, S: Sort, R: Range>
    Request<X, B, P, F, S, R>
where
    Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
{
    /// Create a new request
    pub fn new(
        method: Method,
        request_url: RequestUrl,
        headers: Option<HeaderMap>,
        body: Option<B>,
    ) -> Self {
        Self {
            method,
            request_url,
            headers,
            body,
            pagination: P::default(),
            filter: F::default(),
            sort: S::default(),
            range: R::default(),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::default())),
            force_limit: None,
            _phantom: std::marker::PhantomData,
        }
    }

    fn get_number_of_elements(headers: &HeaderMap) -> u32 {
        match headers
            .get("X-Total")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f32>().ok())
        {
            Some(v) => v as u32,
            None => 1,
        }
    }

    /// Send the request and parse the response into type 'T'
    pub async fn send<T>(&mut self) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
        B: DeserializeOwned + Serialize,
    {
        match self.rate_limiter.write() {
            Ok(mut rate) => rate.request(),
            Err(e) => log::error!("Rate limiter error: {e:?}"),
        }
        let request = self.build_reqwest::<B>(self.body.clone())?;
        log::info!("{request:?}");
        let first_response = Self::execute_reqwest(&request, self.force_limit).await?;
        match self.rate_limiter.write() {
            Ok(mut rate) => rate.update(first_response.headers()),
            Err(e) => log::error!("Rate limiter error: {e:?}"),
        }
        let number_of_elements = Self::get_number_of_elements(first_response.headers());
        match number_of_elements {
            1 => Self::parse_response(first_response).await,
            _ => {
                self.parse_response_array::<T>(request, first_response)
                    .await
            }
        }
    }

    fn build_reqwest<T>(&self, body: Option<T>) -> Result<reqwest::Request>
    where
        T: DeserializeOwned + Serialize,
    {
        let body: Vec<u8> = match body {
            Some(p) => match serde_json::to_string(&p) {
                Ok(s) => s.as_bytes().to_owned(),
                Err(e) => return Err(ApiError::BodySerialization(e)),
            },
            None => Vec::new(),
        };

        let client = Client::new();
        let url =
            self.request_url
                .as_url(&self.pagination, &self.filter, &self.sort, &self.range)?;
        let mut request_builder = client.request(self.method.clone(), url).body(body);
        if let Some(headers) = &self.headers {
            request_builder = request_builder.headers(headers.clone());
        }
        match request_builder.build() {
            Ok(request) => Ok(request),
            Err(e) => Err(ApiError::ReqwestBuilder(e)),
        }
    }

    fn build_next_reqwest(
        previous_request: &reqwest::Request,
        url: Url,
    ) -> Result<reqwest::Request> {
        let request = reqwest::Request::new(previous_request.method().clone(), url);
        let client = Client::new();
        let mut request = reqwest::RequestBuilder::from_parts(client, request)
            .headers(previous_request.headers().to_owned());
        let body: Vec<u8> = match previous_request.body() {
            Some(p) => p.as_bytes().unwrap().to_owned(),
            None => Vec::new(),
        };
        request = request.body(body);

        match request.build() {
            Ok(request) => Ok(request),
            Err(e) => Err(ApiError::ReqwestBuilder(e)),
        }
    }

    async fn execute_reqwest(
        request: &reqwest::Request,
        retries_limit: Option<u8>,
    ) -> Result<reqwest::Response> {
        let client = Client::new();
        let response = client
            .execute(request.try_clone().ok_or(ApiError::ReqwestClone)?)
            .await
            .map_err(ApiError::ReqwestExecute)?;

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let Some(mut limit) = retries_limit else {
                return Err(ApiError::TooManyRequests);
            };
            while limit > 0 {
                limit -= 1;
                let response = client
                    .execute(request.try_clone().ok_or(ApiError::ReqwestClone)?)
                    .await
                    .map_err(ApiError::ReqwestExecute)?;
                if response.status() != StatusCode::TOO_MANY_REQUESTS {
                    break;
                }
            }
        }
        match response.status() {
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::ACCEPTED
            | StatusCode::NO_CONTENT => Ok(response),
            status => Err(status.into()),
        }
    }

    fn get_page_count(headers: &HeaderMap, pagination: &PaginationRule) -> usize {
        let page_count = match headers
            .get("X-Total")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f32>().ok())
        {
            None => 1,
            Some(total) => {
                let per_page = headers
                    .get("X-Per-Page")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(1.);
                (total / per_page).ceil() as usize
            }
        };

        match pagination {
            PaginationRule::Fixed(limit) => std::cmp::min(page_count, limit.to_owned()),
            PaginationRule::OneShot => page_count,
        }
    }

    async fn parse_response<T>(response: reqwest::Response) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let text = match response.text().await {
            Ok(text) => text,
            Err(e) => return Err(ApiError::ResponseToText(e)),
        };
        serde_json::from_slice::<T>(text.as_bytes()).map_err(ApiError::ResponseParse)
    }

    async fn parse_response_array<T>(
        &mut self,
        request: reqwest::Request,
        first_response: reqwest::Response,
    ) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let page_count =
            Self::get_page_count(first_response.headers(), self.pagination.pagination());
        self.pagination.next();
        let mut json_values = Value::Array(Self::parse_response(first_response).await?);

        for _ in 1..page_count {
            let next_url =
                self.request_url
                    .as_url(&self.pagination, &self.filter, &self.sort, &self.range)?;

            let next_request = Self::build_next_reqwest(&request, next_url)?;
            log::info!("{next_request:?}");

            let next_page_response = Self::execute_reqwest(&next_request, self.force_limit).await?;
            match self.rate_limiter.write() {
                Ok(mut rate) => rate.update(next_page_response.headers()),
                Err(e) => log::error!("Rate limiter error: {e:?}"),
            }

            match &mut json_values {
                Value::Array(a) => {
                    let mut json_value: Vec<Value> =
                        Self::parse_response(next_page_response).await?;
                    a.append(&mut json_value)
                }
                _ => return Err(ApiError::JsonValueNotArray),
            }
            self.pagination.next();
        }
        serde_json::from_value::<T>(json_values).map_err(ApiError::ResponseParse)
    }

    pub fn reset_pagination(&mut self) {
        self.pagination.reset();
    }

    /// Pagination setter to override the Api pagination
    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.set_pagination(pagination);
        self
    }

    pub fn set_filter(mut self, filter: F) -> Self {
        self.filter = filter;
        self
    }

    pub fn set_sort(mut self, sort: S) -> Self {
        self.sort = sort;
        self
    }

    pub fn set_range(mut self, range: R) -> Self {
        self.range = range;
        self
    }

    /// Set the pattern filter
    pub fn pattern_filter(mut self, pattern: impl ToString) -> Self {
        self.filter = self.filter.pattern(pattern);
        self
    }

    /// Add a filter to the list
    pub fn filter<T: IntoIterator>(mut self, property: impl ToString, value: T) -> Self
    where
        T::Item: ToString,
    {
        self.filter = self.filter.filter(property, value);
        self
    }

    /// Add a specific filter to a property without the pattern
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

    /// Add a sort on a pattern to the list
    pub fn pattern_sort(mut self, pattern: impl ToString) -> Self {
        self.sort = self.sort.pattern(pattern);
        self
    }

    /// Add a sort on a property to the list
    pub fn sort(mut self, property: impl ToString) -> Self {
        self.sort = self.sort.sort(property);
        self
    }

    /// Add a sort with order on a property to the list
    pub fn sort_with(mut self, property: impl ToString, order: SortOrder) -> Self {
        self.sort = self.sort.sort_with(property, order);
        self
    }

    /// Set the pattern to match the range
    pub fn pattern_range(mut self, pattern: impl ToString) -> Self {
        self.range = self.range.pattern(pattern);
        self
    }

    /// Add a range to the list
    pub fn range(
        mut self,
        property: impl ToString,
        min: impl ToString,
        max: impl ToString,
    ) -> Self {
        self.range = self.range.range(property, min, max);
        self
    }

    /// Set the number of retry attempts on 429 responses
    ///
    /// None means no retry
    pub fn force_limite(mut self, limit: Option<u8>) -> Self {
        self.force_limit = limit;
        self
    }

    /// Add a body to the request
    ///
    /// Do nothing if the request method is not POST, PUT or PATCH
    pub fn body(mut self, body: &B) -> Self {
        match self.method {
            Method::POST | Method::PUT | Method::PATCH => {
                self.body = Some(body.clone());
            }
            _ => {}
        }
        self
    }

    /// Add a query to the request
    pub fn query(mut self, query: impl Into<Query>) -> Self {
        self.request_url = self.request_url.join_query(query.into());
        self
    }
}
