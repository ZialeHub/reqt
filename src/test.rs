#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use reqwest::{Client, StatusCode};
    use serde::{Deserialize, Serialize};

    use crate::prelude::*;

    fn get_credentials() -> TestApiConnector {
        let connector: TestApiConnector = toml::from_str(include_str!("../.env")).unwrap();
        connector
    }

    #[derive(Debug, Clone, Deserialize)]
    struct TokenResponse {
        pub access_token: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct TestApiConnector {
        uid: String,
        secret: String,
        auth_endpoint: String,
        scopes: Vec<String>,
    }

    impl Authentication for TestApiConnector {
        async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
            let pagination = RequestPagination::default();
            let connector = ApiBuilder::new(url, pagination);
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
                        StatusCode::OK
                        | StatusCode::CREATED
                        | StatusCode::ACCEPTED
                        | StatusCode::NO_CONTENT => {}
                        status => return Err(status.into()),
                    }
                    match response.text().await {
                        Ok(response_text) => {
                            let token: TokenResponse =
                                serde_json::from_str(&response_text).unwrap();
                            Ok(connector.bearer(token.access_token).build())
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
    async fn get_method_default() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector.get("users", Query::new())?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default());
        let mut request =
            connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size);
        Ok(())
    }

    #[tokio::test]
    async fn connector_fixed_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::Fixed(3));
        let mut request =
            connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size * 3);
        Ok(())
    }

    #[tokio::test]
    async fn connector_one_shot_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::OneShot);
        let mut request =
            connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 761);
        Ok(())
    }

    #[tokio::test]
    async fn request_none_pagination_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::OneShot);
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::default());
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size);
        Ok(())
    }

    #[tokio::test]
    async fn request_fixed_pagination_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::Fixed(3));
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size * 3);
        Ok(())
    }

    #[tokio::test]
    async fn request_one_shot_pagination_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::OneShot);
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 761);
        Ok(())
    }

    #[tokio::test]
    async fn request_consistency() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::Fixed(2));
        let first_response = request.send::<Vec<User>>().await?;
        assert_eq!(first_response.len(), connector.pagination.size * 2);
        let second_response = request.send::<Vec<User>>().await?;
        assert_eq!(second_response.len(), connector.pagination.size * 2);
        assert_eq!(request.pagination.current_page(), 5);
        assert!(first_response
            .iter()
            .zip(second_response.iter())
            .all(|(a, b)| a.login != b.login));
        Ok(())
    }

    #[tokio::test]
    async fn request_consistency_after_reset() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::Fixed(2));
        let first_response = request.send::<Vec<User>>().await?;
        assert_eq!(first_response.len(), connector.pagination.size * 2);
        request.pagination.reset();
        let second_response = request.send::<Vec<User>>().await?;
        assert_eq!(second_response.len(), connector.pagination.size * 2);
        assert_eq!(request.pagination.current_page(), 3);
        assert!(first_response
            .iter()
            .zip(second_response.iter())
            .all(|(a, b)| a.login == b.login));
        Ok(())
    }
}
