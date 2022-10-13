use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use base_proj::{
    configuration::{get_configuration, DatabaseConfiguration},
    email_client::EmailClient,
    startup::run,
    telemetry::{create_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use wiremock::MockServer;

// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test]` attribute.
//

// when testing, you can enable or disable the `tracing` subscriber by
// setting the `TEST_LOG` environment variable. For example:
// TEST_LOG=true cargo test health_check_works | bunyan
static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = create_subscriber("test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = create_subscriber("test".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }
    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        // We don't care about the exact Argon2 parameters here
        // given that it's for testing purposes!
        let password_hash = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash)
    VALUES ($1, $2, $3)",
            self.user_id,
            self.username,
            password_hash,
        )
        .execute(pool)
        .await
        .expect("Failed to store test user.");
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Extract the confirmation links embedded in the request to the email API.
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Port 0 tells the OS to give us any available port.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Launch a mock server to stand in for Postmark's API
    let email_server = MockServer::start().await;

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    configuration.email_client.base_url = email_server.uri();
    let connection_pool = configure_database(&configuration.database).await;

    // let connection_string = configuration.database.connection_string();
    // let connection_pool = PgPool::connect(&connection_string)
    //     .await
    //     .expect("Failed to connect to Postgres.");

    // Build a new email client
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        sender_email,
        configuration.email_client.base_url,
        configuration.email_client.authorization_token,
        timeout,
    );

    let server = run(
        listener,
        connection_pool.clone(),
        email_client,
        configuration.application.base_url,
    )
    .expect("Failed to bind address");

    // let application_port = application.port();

    let _ = tokio::spawn(server);
    let test_app = TestApp {
        address,
        db_pool: connection_pool,
        email_server,
        port,
        test_user: TestUser::generate(),
    };
    test_app.test_user.store(&test_app.db_pool).await;
    test_app
}

async fn configure_database(config: &DatabaseConfiguration) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
