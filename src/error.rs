use reqwest::header::InvalidHeaderValue;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Not Found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Too Many Requests")]
    TooManyRequests,
    #[error("Bad Request")]
    BadRequest,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Pagination Done")]
    PaginationDone,
    #[error("Page Limit Exceeded")]
    PageLimitExceeded,
    #[error("JsonValue is not an Array")]
    JsonValueNotArray,
    #[error("Response parse to T: {0}")]
    ResponseParse(#[source] serde_json::Error),
    #[error("Response to text: {0}")]
    ResponseToText(#[source] reqwest::Error),
    #[error("ExecuteReqwest: {0}")]
    ReqwestExecute(#[source] reqwest::Error),
    #[error("Clone reqwest")]
    ReqwestClone,
    #[error("ReqwestBuilder: {0}")]
    ReqwestBuilder(#[source] reqwest::Error),
    #[error("Wrong Body Format: {0}")]
    BodySerialization(#[from] serde_json::Error),
    #[error("Wrong Url Format: {0}")]
    WrongUrlFormat(#[from] url::ParseError),
    #[error("Invalid Header Value: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("{1} ➤  {0}")]
    Connector(#[source] Box<ApiError>, ConnectorError),
}

impl From<reqwest::StatusCode> for ApiError {
    fn from(status: reqwest::StatusCode) -> Self {
        match status {
            reqwest::StatusCode::NOT_FOUND => ApiError::NotFound,
            reqwest::StatusCode::UNAUTHORIZED => ApiError::Unauthorized,
            reqwest::StatusCode::TOO_MANY_REQUESTS => ApiError::TooManyRequests,
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => ApiError::InternalServerError,
            _ => ApiError::BadRequest,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Connector")]
pub struct ConnectorError;

pub trait ErrorContext<T, E> {
    fn err_ctx(self, context: E) -> Result<T>;
}

impl<T> ErrorContext<T, ConnectorError> for Result<T> {
    fn err_ctx(self, context: ConnectorError) -> Result<T> {
        self.map_err(|e| ApiError::Connector(Box::new(e), context))
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
