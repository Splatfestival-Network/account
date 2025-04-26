use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use std::fs;

pub async fn send_verification_email(to: &str, code: i32, username: &str) -> Result<(), String> {
    let smtp_user = env::var("SMTP_USER").map_err(|_| "SMTP_USER not set".to_string())?;
    let smtp_pass = env::var("SMTP_PASS").map_err(|_| "SMTP_PASS not set".to_string())?;
    let smtp_server = env::var("SMTP_SERVER").map_err(|_| "SMTP_SERVER not set".to_string())?;

    // Load template
    let template = fs::read_to_string("res/email/confirmationTemplate.html")
        .map_err(|e| format!("Failed to read email template: {}", e))?;

    // Replace placeholders
    let body = template
        .replace("{{username}}", username)
        .replace("{{confirmation-code}}", &format!("{:06}", code));

    let email = Message::builder()
        .from(smtp_user.parse().unwrap())
        .to(to.parse().unwrap())
        .subject("Your Verification Code")
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(body)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(smtp_user, smtp_pass);

    let mailer = SmtpTransport::relay(&smtp_server)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();

    mailer.send(&email).map_err(|e| e.to_string())?;

    Ok(())
}
