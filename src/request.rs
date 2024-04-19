use reqwest::{header::HeaderMap, Client, Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{
    error::ApiError,
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

    pub async fn send<T>(&self) -> Result<T, ApiError>
    where
        T: DeserializeOwned + Serialize,
        P: DeserializeOwned + Serialize,
    {
        let request = self.request::<P>(self.payload.clone())?;
        println!("Sending request: '{}'", request.url());
        let response = Self::execute(&request).await?;
        let status = response.status().as_u16();
        let number_of_elements = Self::get_number_of_elements(response.headers());
        let mut page_count = Self::get_page_count(response.headers());
        page_count = match self.pagination.get_pagination() {
            PaginationRule::None => 1,
            PaginationRule::Fixed(limit) => std::cmp::min(page_count, limit.to_owned()),
            PaginationRule::OneShot => page_count,
        };
        let parsed_response = self.parse_response_array(request, page_count).await;
        println!(
            "Received response: code {}, {} elements, expected type: {} -> successfully parsed: {}",
            status,
            number_of_elements,
            std::any::type_name::<T>(),
            parsed_response.is_ok()
        );
        parsed_response
    }

    fn request<T>(&self, payload: Option<T>) -> Result<reqwest::Request, ApiError>
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
        let url = self.request_url.as_url()?;
        let mut request_builder = client.request(self.method.clone(), url).body(body);
        if let Some(headers) = &self.headers {
            request_builder = request_builder.headers(headers.clone());
        }
        match request_builder.build() {
            Ok(request) => Ok(request),
            Err(e) => Err(ApiError::ReqwestBuilder(e)),
        }
    }

    async fn execute(request: &reqwest::Request) -> Result<reqwest::Response, ApiError> {
        println!("\nrequest = {:?}\n", request.url());
        let client = Client::new();
        let mut response = client
            .execute(request.try_clone().ok_or(ApiError::ReqwestClone)?)
            .await
            .map_err(|e| ApiError::ReqwestExecute(e))?;
        println!("response = {:?}", response);

        let remaining_secondly_calls = response
            .headers()
            .get("X-Secondly-RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u8>().ok());
        if remaining_secondly_calls == Some(0) {
            let time = std::time::Duration::from_millis(1100);
            tokio::time::sleep(time).await;
            println!("reaching limit, pausing requests for 1.1sec to avoid 429");
        }
        if response.status().as_u16() == 429 {
            println!(
                "Reached 429 despite rate-limit protection: {}",
                request.url()
            );
            let time = std::time::Duration::from_millis(1100);
            tokio::time::sleep(time).await;

            response = client
                .execute(request.try_clone().ok_or(ApiError::ReqwestClone)?)
                .await
                .map_err(|e| ApiError::ReqwestExecute(e))?;
        }
        println!(
            "Request response status : {} -> {:?}",
            response.status().as_u16(),
            request
        );
        match response.status() {
            StatusCode::NOT_FOUND => Err(ApiError::NotFound),
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            StatusCode::TOO_MANY_REQUESTS => Err(ApiError::TooManyRequests),
            StatusCode::INTERNAL_SERVER_ERROR => Err(ApiError::InternalServerError),
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::ACCEPTED
            | StatusCode::NO_CONTENT => Ok(response),
            _ => Err(ApiError::BadRequest),
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

    fn get_page_count(headers: &HeaderMap) -> u32 {
        match headers
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
        }
    }

    async fn parse_response<T>(response: reqwest::Response) -> Result<T, ApiError>
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
        mut page_count: u32,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned + Serialize,
    {
        let mut request_url = self.request_url.clone();
        let mut json_values = Value::Array(Vec::new());
        let client = Client::new();
        while page_count > 0 {
            page_count -= 1;
            request_url.request_next_page();

            let Ok(new_url) = request_url.as_url() else {
                return Err(ApiError::InternalServerError);
            };
            let Ok(request) = Self::build_request(&client, &request, new_url) else {
                return Err(ApiError::InternalServerError);
            };
            let next_page_response = client
                .execute(request)
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

    fn build_request(
        client: &Client,
        old_request: &reqwest::Request,
        url: Url,
    ) -> Result<reqwest::Request, ApiError> {
        let req = reqwest::Request::new(old_request.method().clone(), url);
        let mut request = reqwest::RequestBuilder::from_parts(client.clone(), req)
            .headers(old_request.headers().to_owned());
        if old_request.body().is_some() {
            let body: Vec<u8> = match old_request.body() {
                Some(p) => p.as_bytes().unwrap().to_owned(),
                None => Vec::new(),
            };
            request = request.body(body);
        }

        match request.build() {
            Ok(request) => Ok(request),
            Err(e) => Err(ApiError::ReqwestBuilder(e)),
        }
    }

    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.pagination(pagination);
        self
    }
}
