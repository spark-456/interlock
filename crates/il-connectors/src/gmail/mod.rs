//! Gmail: new-email trigger, reply, send, label.
//!
//! Owned by task C01. Add one file per node and list it here.

mod common;
pub mod label;
pub mod new_email;
pub mod reply;
pub mod send;

fn google_provider() -> crate::oauth::Provider {
    crate::oauth::Provider {
        id: "google",
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
        token_url: "https://oauth2.googleapis.com/token".into(),
        scopes: vec![
            "https://www.googleapis.com/auth/gmail.modify".into(),
            "https://www.googleapis.com/auth/gmail.send".into(),
            "https://www.googleapis.com/auth/spreadsheets".into(),
            "https://www.googleapis.com/auth/youtube.upload".into(),
        ],
        client_id_env: "GOOGLE_CLIENT_ID",
        client_secret_env: "GOOGLE_CLIENT_SECRET",
        redirect_uri_env: "GOOGLE_REDIRECT_URI",
    }
}

inventory::submit! { crate::oauth::ProviderFactory(google_provider) }
