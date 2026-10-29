//! Model providers. One module per provider; each self-registers a `ProviderFactory`.

pub mod fake;
mod groq;
pub mod openai;
mod openrouter;
mod tokenfactory;
