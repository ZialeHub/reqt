#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use reqwest::{Client, StatusCode};
    use serde::{Deserialize, Serialize};

    use crate::prelude::*;

    // TODO TO start tests you must provide your own UID and SECRET
    const UID: &'static str = "your uid";
    const SECRET: &'static str = "your secret";

    #[derive(Debug, Clone, Deserialize)]
    struct TokenResponse {
        pub access_token: String,
    }

    struct TestApiConnector {
        uid: String,
        secret: String,
        auth_endpoint: String,
        scopes: Vec<String>,
    }
    impl Authentification for TestApiConnector {
        async fn connect(&self, url: &str) -> Result<ApiConnector, ApiError> {
            let connector = ApiConnectorBuilder::new(url).token_type(TokenType::Bearer);
            let client = Client::new();

            let scopes = self
                .scopes
                .iter()
                .fold(String::new(), |acc, scope| format!("{} {}", acc, scope));
            let mut params = HashMap::new();
            params.insert("grant_type", "client_credentials");
            params.insert("client_id", &self.uid);
            params.insert("client_secret", &self.secret);
            params.insert("scope", &scopes);
            match client
                .post(&self.auth_endpoint)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .form(&params)
                .send()
                .await
            {
                Ok(response) => {
                    match response.status() {
                        StatusCode::NOT_FOUND => return Err(ApiError::NotFound),
                        StatusCode::UNAUTHORIZED => return Err(ApiError::Unauthorized),
                        StatusCode::TOO_MANY_REQUESTS => return Err(ApiError::TooManyRequests),
                        StatusCode::INTERNAL_SERVER_ERROR => {
                            return Err(ApiError::InternalServerError)
                        }
                        StatusCode::OK
                        | StatusCode::CREATED
                        | StatusCode::ACCEPTED
                        | StatusCode::NO_CONTENT => {}
                        _ => return Err(ApiError::BadRequest),
                    }
                    match response.text().await {
                        Ok(response_text) => {
                            let token: TokenResponse =
                                serde_json::from_str(&response_text).unwrap();
                            eprintln!("token = {}", token.access_token);
                            Ok(connector.token(&token.access_token).build())
                        }
                        Err(e) => Err(ApiError::ResponseToText(e)),
                    }
                }
                Err(e) => Err(ApiError::ReqwestExecute(e)),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct User {
        pub login: String,
    }

    #[tokio::test]
    async fn test_get_method_default() -> Result<(), ApiError> {
        let data_connector = TestApiConnector {
            uid: UID.to_string(),
            secret: SECRET.to_string(),
            auth_endpoint: "https://api.intra.42.fr/oauth/token".to_string(),
            scopes: vec!["public".to_string()],
        };
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        eprintln!("{:?}", connector);
        let request = connector.get("users", Query::build())?;
        eprintln!("{:?}", request);
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        assert_eq!(response.len(), 30);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_method_with_pagination_limit_from_connector() -> Result<(), ApiError> {
        let data_connector = TestApiConnector {
            uid: UID.to_string(),
            secret: SECRET.to_string(),
            auth_endpoint: "https://api.intra.42.fr/oauth/token".to_string(),
            scopes: vec!["public".to_string()],
        };
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::Fixed(3));
        eprintln!("{:?}", connector);
        let request =
            connector.get("users", Query::build().add("filter[primary_campus_id]", 31))?;
        eprintln!("{:?}", request);
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        eprintln!("\n\nLEN = {:?}\n\n", response.len());
        assert_eq!(response.len(), 90);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_method_with_pagination_one_shot_from_connector() -> Result<(), ApiError> {
        let data_connector = TestApiConnector {
            uid: UID.to_string(),
            secret: SECRET.to_string(),
            auth_endpoint: "https://api.intra.42.fr/oauth/token".to_string(),
            scopes: vec!["public".to_string()],
        };
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::OneShot);
        eprintln!("{:?}", connector);
        let request =
            connector.get("users", Query::build().add("filter[primary_campus_id]", 31))?;
        eprintln!("{:?}", request);
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        eprintln!("\n\nLEN = {:?}\n\n", response.len());
        assert_eq!(response.len(), 721);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_method_without_pagination_from_request() -> Result<(), ApiError> {
        let data_connector = TestApiConnector {
            uid: UID.to_string(),
            secret: SECRET.to_string(),
            auth_endpoint: "https://api.intra.42.fr/oauth/token".to_string(),
            scopes: vec!["public".to_string()],
        };
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::OneShot);
        eprintln!("{:?}", connector);
        let request = connector
            .get("users", Query::build().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::None);
        eprintln!("{:?}", request);
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        eprintln!("\n\nLEN = {:?}\n\n", response.len());
        assert_eq!(response.len(), 30);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_method_with_pagination_limit_from_request() -> Result<(), ApiError> {
        let data_connector = TestApiConnector {
            uid: UID.to_string(),
            secret: SECRET.to_string(),
            auth_endpoint: "https://api.intra.42.fr/oauth/token".to_string(),
            scopes: vec!["public".to_string()],
        };
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        eprintln!("{:?}", connector);
        let request = connector
            .get("users", Query::build().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::Fixed(3));
        eprintln!("{:?}", request);
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        eprintln!("\n\nLEN = {:?}\n\n", response.len());
        assert_eq!(response.len(), 90);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_method_with_pagination_one_shot_from_request() -> Result<(), ApiError> {
        let data_connector = TestApiConnector {
            uid: UID.to_string(),
            secret: SECRET.to_string(),
            auth_endpoint: "https://api.intra.42.fr/oauth/token".to_string(),
            scopes: vec!["public".to_string()],
        };
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        eprintln!("{:?}", connector);
        let request = connector
            .get("users", Query::build().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::OneShot);
        eprintln!("{:?}", request);
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        eprintln!("\n\nLEN = {:?}\n\n", response.len());
        assert_eq!(response.len(), 721);
        Ok(())
    }
}
