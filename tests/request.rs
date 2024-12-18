use fake::{Dummy, Fake, Faker};
use httpmock::MockServer;
use reqt::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
struct User {
    id: u64,
    name: String,
}

#[derive(Debug, Clone)]
struct PaginationTest {
    pub size: usize,
    pub current_page: usize,
    pub pagination: PaginationRule,
}
impl Default for PaginationTest {
    fn default() -> Self {
        Self {
            size: 100,
            current_page: 1,
            pagination: PaginationRule::default(),
        }
    }
}
impl Pagination for PaginationTest {
    fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    fn set_pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = pagination;
        self
    }

    fn pagination(&self) -> &PaginationRule {
        &self.pagination
    }

    fn current_page(&self) -> usize {
        self.current_page
    }

    fn get_current_page(&self) -> Query {
        Query::new()
            .add("page[number]", self.current_page)
            .add("page[size]", self.size)
    }

    fn get_size(&self) -> Query {
        Query::new().add("page[size]", self.size)
    }

    fn next(&mut self) {
        self.current_page += 1;
    }

    fn get_next_page(&mut self) -> Query {
        self.current_page += 1;
        Query::new()
            .add("page[number]", self.current_page)
            .add("page[size]", self.size)
    }

    fn reset(&mut self) {
        self.current_page = 1;
    }
}

fn mock_pages<'de, T: Clone + Serialize + Deserialize<'de>>(
    server: &MockServer,
    array: Vec<T>,
    split_by: usize,
) {
    for i in 0..(array.len() / split_by) {
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("page[number]", (i + 1).to_string())
                .query_param("page[size]", "100");
            then.status(200)
                .header("Content-Type", "application/json")
                .header("X-Total", array.len().to_string())
                .header("X-Per-Page", split_by.to_string())
                .json_body_obj(&array[(i * split_by)..((i + 1) * split_by)].to_vec());
        });
    }
}

fn mock_server_users() -> MockServer {
    let server = MockServer::start();

    let mut users = Vec::<User>::new();
    for _ in 0..1000 {
        users.push(Faker.fake());
    }

    server.mock(|when, then| {
        when.method("GET").path("/users/full");
        then.status(200)
            .header("Content-Type", "application/json")
            .json_body_obj(&users);
    });
    mock_pages::<User>(&server, users, 100);
    server
}

#[cfg(test)]
mod request_tests {
    use reqt::prelude::*;

    use super::*;

    #[tokio::test]
    async fn request_1000_users() -> Result<()> {
        let server = mock_server_users();
        let api: Api = ApiBuilder::new(server.base_url()).build();
        let users: Vec<User> = api.get("/users/full")?.await?;
        assert_eq!(users.len(), 1000);
        Ok(())
    }

    #[tokio::test]
    async fn request_users_page_1() -> Result<()> {
        let server = mock_server_users();
        let api: Api<PaginationTest> = ApiBuilder::new(server.base_url()).build();
        let users: Vec<User> = api.get("/users")?.await?;
        assert_eq!(users.len(), 100);
        Ok(())
    }

    #[tokio::test]
    async fn request_users_full_pages() -> Result<()> {
        let server = mock_server_users();
        let api: Api<PaginationTest> = ApiBuilder::new(server.base_url()).build();
        let users: Vec<User> = api
            .get("/users")?
            .pagination(PaginationRule::OneShot)
            .await?;
        assert_eq!(users.len(), 1000);
        Ok(())
    }
}
