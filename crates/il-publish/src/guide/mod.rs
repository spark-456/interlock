//! `INTERLOCK.md` integration guide generator.

use il_core::Flow;

/// Generates a copyable integration guide for a published AI app.
pub fn generate(flow: &Flow, base_url: &str) -> String {
    let slug = flow
        .name
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let endpoint = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
    format!(
        r#"# {name} integration

Model: `{slug}`

Endpoint: `{endpoint}`

Authentication: `Authorization: Bearer $INTERLOCK_API_KEY`

The API accepts OpenAI chat completion messages and supports `stream: true` for server-sent events. List published models at `{base}/v1/models`.

## cURL

```sh
curl {endpoint} -H "Authorization: Bearer $INTERLOCK_API_KEY" -H "Content-Type: application/json" -d '{{"model":"{slug}","messages":[{{"role":"user","content":"Hello"}}]}}'
```

## JavaScript

```js
import OpenAI from 'openai';
const client = new OpenAI({{ apiKey: process.env.INTERLOCK_API_KEY, baseURL: '{base}/v1' }});
const reply = await client.chat.completions.create({{ model: '{slug}', messages: [{{ role: 'user', content: 'Hello' }}] }});
console.log(reply.choices[0].message.content);
```

## Python

```python
from openai import OpenAI
client = OpenAI(base_url="{base}/v1", api_key=__import__("os").environ["INTERLOCK_API_KEY"])
reply = client.chat.completions.create(model="{slug}", messages=[{{"role": "user", "content": "Hello"}}])
print(reply.choices[0].message.content)
```

## Rust

```rust
let client = reqwest::Client::new();
let response = client.post("{endpoint}")
    .bearer_auth(std::env::var("INTERLOCK_API_KEY")?)
    .json(&serde_json::json!({{"model":"{slug}","messages":[{{"role":"user","content":"Hello"}}]}}))
    .send().await?.error_for_status()?;
let result: serde_json::Value = response.json().await?;
println!("{{}}", result["choices"][0]["message"]["content"]);
```
"#,
        name = flow.name,
        base = base_url.trim_end_matches('/')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p01_guide_has_four_languages_and_no_raw_key() {
        let flow: Flow = serde_json::from_value(serde_json::json!({"id":"flow_1","workspaceId":"ws_a","version":1,
            "name":"Support Agent","kind":"app","nodes":[],"edges":[]}))
        .unwrap();
        let guide = generate(&flow, "https://interlock.example/");
        for name in ["## cURL", "## JavaScript", "## Python", "## Rust", "support-agent"] {
            assert!(guide.contains(name));
        }
        assert!(!guide.contains("ilk_"));
    }
}
