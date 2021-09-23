use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use std::convert::TryInto;
use unicode_segmentation::UnicodeSegmentation;

use crate::domain::{NewSubscriber, SubscriberName, SubscriberEmail};
use crate::email_client::EmailClient;

#[derive(serde::Deserialize)]
pub struct FormData {

    email: String,
    name: String,

}

impl TryInto<NewSubscriber> for FormData {

    type Error = String;
    
    fn try_into(self) -> Result<NewSubscriber, Self::Error> {
    
        let name = SubscriberName::parse(self.name)?;
        let email = SubscriberEmail::parse(self.email)?;
    
        Ok(NewSubscriber { email, name })
    }
    
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        email = %form.email,
        name = %form.name
    )
)]
pub async fn subscribe(
    
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    
) -> Result<HttpResponse, HttpResponse> {

    let new_subscriber = form
        .0
        .try_into()
        .map_err(|_| HttpResponse::BadRequest().finish())?;

    insert_subscriber(&pool, &new_subscriber)
        .await
        .map_err(|_|HttpResponse::InternalServerError().finish())?;

    let confirmation_link =
        "https://my-api.com/subscriptions/confirm"; //

    let _ = send_confirmation_email(&email_client, new_subscriber).await;

    Ok(HttpResponse::Ok().finish())

}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())

}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(

    email_client: &EmailClient,
    new_subscriber: NewSubscriber,

) -> Result<(), reqwest::Error> {

    let confirmation_link = "https://my-api.com/subscriptions/confirm";

    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &html_body,
            &plain_body,
        )
        .await

}

pub fn is_valid_name(s: &str) -> bool {

    let is_empty_or_whitespace = s.trim().is_empty();

    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)

}