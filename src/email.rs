use std::error::Error;

use lettre::{
    message::{Body, Mailbox, MessageBuilder},
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use tokio::{spawn, task::JoinHandle};

use crate::config::EmailConfig;

pub struct EmailHandler<'a> {
    connection_url: &'a String,
    from: &'a Mailbox,
    to: &'a Mailbox,
}

impl<'a> EmailHandler<'a> {
    pub fn new(config: &'a EmailConfig) -> Self {
        return Self {
            connection_url: &config.connection_url,
            from: &config.from,
            to: &config.to,
        };
    }

    pub async fn send_email(
        &self,
        msg: MessageBuilder,
        body: Body,
    ) -> Result<JoinHandle<()>, Box<dyn Error + Send + Sync>> {
        let email = msg.from(self.from.clone()).to(self.to.clone()).body(body)?;
        let conn_url = self.connection_url.clone();
        return Ok(spawn(async move {
            let client = AsyncSmtpTransport::<Tokio1Executor>::from_url(&conn_url);
            match client {
                Ok(value) => {
                    match value.build().send(email).await {
                        Ok(_) => (),
                        Err(err) => {
                            eprintln!("Failed to send email: {err}");
                        }
                    };
                }
                Err(err) => {
                    eprintln!("Failed to create email transport: {err}");
                }
            };
        }));
    }
}
