use reqwest::{header::HeaderMap, Client, Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{
    error::{ApiError, Result},
    pagination::{Pagination, PaginationRule, RequestPagination},
    request_url::RequestUrl,
};

#[derive(Debug, Clone)]
pub struct Request<P: Serialize + Clone = (), U: Pagination + Clone = RequestPagination> {
    pub(crate) method: Method,
    pub(crate) request_url: RequestUrl,
    pub(crate) headers: Option<HeaderMap>,
    pub(crate) payload: Option<P>,
    pub(crate) pagination: U,
}

impl<P: Serialize + Clone, U: Pagination + Clone> Request<P, U> {
    pub fn new(
        method: Method,
        request_url: RequestUrl,
        headers: Option<HeaderMap>,
        payload: Option<P>,
        pagination: U,
    ) -> Self {
        Self {
            method,
            request_url,
            headers,
            payload,
            pagination,
        }
    }

    pub async fn send<T>(&self) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
        P: DeserializeOwned + Serialize,
    {
        let request = self.build_reqwest::<P>(self.payload.clone())?;
        let first_response = Self::execute_reqwest(&request).await?;
        let parsed_response = self.parse_response_array(request, first_response).await;
        parsed_response
    }

    fn build_reqwest<T>(&self, payload: Option<T>) -> Result<reqwest::Request>
    where
        T: DeserializeOwned + Serialize,
    {
        let body: Vec<u8> = match payload {
            Some(p) => match serde_json::to_string(&p) {
                Ok(s) => s.as_bytes().to_owned(),
                Err(e) => return Err(ApiError::PayloadSerialization(e)),
            },
            None => Vec::new(),
        };

        let client = Client::new();
        let url = self.request_url.as_url(&self.pagination)?;
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
        let response = client
            .execute(request.try_clone().ok_or(ApiError::ReqwestClone)?)
            .await
            .map_err(|e| ApiError::ReqwestExecute(e))?;

        let remaining_secondly_calls = response
            .headers()
            .get("X-Secondly-RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u8>().ok());

        if remaining_secondly_calls == Some(0) {
            let time = std::time::Duration::from_millis(1100);
            tokio::time::sleep(time).await;
        }

        match response.status() {
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::ACCEPTED
            | StatusCode::NO_CONTENT => Ok(response),
            status => Err(status.into()),
        }
    }

    fn get_page_count(headers: &HeaderMap, pagination: &PaginationRule) -> u32 {
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
                (total / per_page).ceil() as u32
            }
        };

        match pagination {
            PaginationRule::None => 1,
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
        serde_json::from_slice::<T>(text.as_bytes()).map_err(|e| ApiError::ResponseParse(e))
    }

    async fn parse_response_array<T>(
        &self,
        request: reqwest::Request,
        first_response: reqwest::Response,
    ) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let page_count =
            Self::get_page_count(first_response.headers(), self.pagination.get_pagination());
        let mut pagination = self.pagination.clone();
        pagination.next();
        let mut json_values = Value::Array(Self::parse_response(first_response).await?);
        let client = Client::new();

        for _ in 1..page_count {
            pagination.next();
            let next_url = self.request_url.as_url(&pagination)?;

            let next_request = Self::build_next_reqwest(&request, next_url)?;

            let next_page_response = client
                .execute(next_request)
                .await
                .map_err(|e| ApiError::ReqwestExecute(e))?;

            match &mut json_values {
                Value::Array(a) => {
                    let mut json_value: Vec<Value> =
                        Self::parse_response(next_page_response).await?;
                    a.append(&mut json_value)
                }
                _ => return Err(ApiError::JsonValueNotArray),
            }
        }
        serde_json::from_value::<T>(json_values).map_err(|e| ApiError::ResponseParse(e))
    }

    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.pagination(pagination);
        self
    }
}
