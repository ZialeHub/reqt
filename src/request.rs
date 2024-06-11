use reqwest::{header::HeaderMap, Client, Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{
    error::{ApiError, Result},
    filter::{Filter, FilterRule},
    pagination::{Pagination, PaginationRule, RequestPagination},
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

impl<B: Serialize + Clone, P: Pagination, F: Filter, S: Sort> Request<B, P, F, S> {
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
        }
    }

    /// Send the request and parse the response into type 'T'
    pub async fn send<T>(&mut self) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
        B: DeserializeOwned + Serialize,
    {
        let request = self.build_reqwest::<B>(self.body.clone())?;
        eprintln!("{:?}", request);
        let first_response = Self::execute_reqwest(&request).await?;
        self.parse_response_array(request, first_response).await
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
        let url = self
            .request_url
            .as_url(&self.pagination, &self.filter, &self.sort)?;
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

    async fn execute_reqwest(request: &reqwest::Request) -> Result<reqwest::Response> {
        let client = Client::new();
        let mut response = client
            .execute(request.try_clone().ok_or(ApiError::ReqwestClone)?)
            .await
            .map_err(ApiError::ReqwestExecute)?;

        let remaining_secondly_calls = response
            .headers()
            .get("X-Secondly-RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u8>().ok());

        if remaining_secondly_calls == Some(0) {
            let time = std::time::Duration::from_millis(1100);
            tokio::time::sleep(time).await;
        }
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let time = std::time::Duration::from_millis(1100);
            tokio::time::sleep(time).await;
            response = client
                .execute(request.try_clone().ok_or(ApiError::ReqwestClone)?)
                .await
                .map_err(ApiError::ReqwestExecute)?;
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
            Self::get_page_count(first_response.headers(), self.pagination.get_pagination());
        self.pagination.next();
        let mut json_values = Value::Array(Self::parse_response(first_response).await?);

        for _ in 1..page_count {
            let next_url = self
                .request_url
                .as_url(&self.pagination, &self.filter, &self.sort)?;

            let next_request = Self::build_next_reqwest(&request, next_url)?;

            let next_page_response = Self::execute_reqwest(&next_request).await?;

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

    /// Pagination setter to override the Api pagination
    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.pagination(pagination);
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

    pub fn sort_with(mut self, property: impl ToString, order: SortOrder) -> Self {
        self.sort = self.sort.sort_with(property, order);
        self
    }
}
