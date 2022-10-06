use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>, // We don't want to log this by accident
}
// Structure desired by the postman service.
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}
impl EmailClient {
    pub fn new(
        sender: SubscriberEmail,
        base_url: String, // We don't want to log this by accident
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            sender,
            base_url,
            http_client,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        text_content: &str,
        html_content: &str,
    ) -> Result<(), reqwest::Error> {
        // You can do better using `reqwest::Url::join` if you change
        // `base_url`'s type from `String` to `reqwest::Url`.
        // I'll leave it as an exercise for the reader!
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        self.http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use fake::Fake;
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Faker,
    };
    use secrecy::Secret;
    use tokio_test::{assert_err, assert_ok};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use crate::domain::SubscriberEmail;

    use super::EmailClient;

    struct SendEmailBodyMatcher;
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body as a JSON value
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                // Check that all the mandatory fields are populated
                // without inspecting the field values
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // If parsing failed, do not match the request
                false
            }
        }
    }

    // Helper functions for tests
    fn random_email_data() -> (SubscriberEmail, String, String) {
        let subscriber_email = email_data();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();
        (subscriber_email, subject, content)
    }

    fn email_data() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        // Let's fail fast in tests; Very important for development loop speed
        let timeout = std::time::Duration::from_millis(200);
        EmailClient::new(email_data(), base_url, Secret::new(Faker.fake()), timeout)
    }

    #[tokio::test]
    async fn send_email_sends_a_request_to_the_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let (subscriber_email, subject, content) = random_email_data();

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            // Use our custom matcher!
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let (subscriber_email, subject, content) = random_email_data();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_ok!(outcome);
    }
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let (subscriber_email, subject, content) = random_email_data();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let (subscriber_email, subject, content) = random_email_data();

        let response = ResponseTemplate::new(200)
            // 3 minutes!
            .set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(outcome);
    }
}
