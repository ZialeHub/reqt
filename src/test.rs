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
            let connector = ApiConnectorBuilder::new(url, pagination);
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
        let request = connector.get("users", Query::new())?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 30);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::None);
        let request = connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 30);
        Ok(())
    }

    #[tokio::test]
    async fn connector_fixed_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::Fixed(3));
        let request = connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 90);
        Ok(())
    }

    #[tokio::test]
    async fn connector_one_shot_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::OneShot);
        let request = connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
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
        let request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::None);
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 30);
        Ok(())
    }

    #[tokio::test]
    async fn request_fixed_pagination_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::Fixed(3));
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 90);
        Ok(())
    }

    #[tokio::test]
    async fn request_one_shot_pagination_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::OneShot);
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 761);
        Ok(())
    }
}
