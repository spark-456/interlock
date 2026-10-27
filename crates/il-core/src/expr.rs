//! Expression rendering (minijinja, sandboxed: no IO, no filesystem loader).
//!
//! Scope variables available to settings expressions (the engine and `NodeHarness` both use [`Scope`]):
//! - `item`  — the current item's JSON (per-item nodes) or the first item (batch nodes)
//! - `items` — all input items' JSON
//! - `nodes` — `{ "<nodeId>": { "json": <first output item json>, "items": [...] } }` for upstream nodes
//! - `run`   — `{ "id", "flowId", "trigger" }`
//! - `now`   — RFC 3339 timestamp
//!
//! Helpers: `json` (serialize), `first`, `join`, `default` (minijinja built-ins), `tojson`.
//!
//! A string that is exactly one `{{ expr }}` evaluates to the expression's JSON value (keeping
//! arrays, numbers and objects); any other string with `{{`/`{%` renders as text.

use std::sync::OnceLock;

use minijinja::{Environment, UndefinedBehavior, Value as MjValue};
use serde_json::Map;

use crate::{CoreError, Json};

/// Variables available to expressions.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    vars: Map<String, Json>,
}

impl Scope {
    /// An empty scope.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets `item`.
    pub fn item(self, item: Json) -> Self {
        self.var("item", item)
    }
    /// Sets `items`.
    pub fn items(self, items: Vec<Json>) -> Self {
        self.var("items", Json::Array(items))
    }
    /// Sets any variable.
    pub fn var(mut self, name: &str, value: Json) -> Self {
        self.vars.insert(name.into(), value);
        self
    }
    fn as_json(&self) -> Json {
        Json::Object(self.vars.clone())
    }
}

fn env() -> &'static Environment<'static> {
    static ENV: OnceLock<Environment<'static>> = OnceLock::new();
    ENV.get_or_init(|| {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Chainable);
        env.set_recursion_limit(32);
        env.add_filter("json", |v: MjValue| -> Result<String, minijinja::Error> {
            serde_json::to_string(&v)
                .map_err(|e| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))
        });
        env
    })
}

/// True when the string contains template syntax.
pub fn is_expression(s: &str) -> bool {
    s.contains("{{") || s.contains("{%")
}

/// Renders one string. See module docs for the single-expression rule.
pub fn render_str(template: &str, scope: &Scope) -> Result<Json, CoreError> {
    if !is_expression(template) {
        return Ok(Json::String(template.to_string()));
    }
    let ctx = MjValue::from_serialize(scope.as_json());
    let trimmed = template.trim();
    if let Some(inner) = single_expression(trimmed) {
        let expr = env().compile_expression(inner).map_err(err)?;
        let value = expr.eval(ctx).map_err(err)?;
        if value.is_undefined() || value.is_none() {
            return Ok(Json::Null);
        }
        return serde_json::to_value(&value).map_err(|e| CoreError::Expression(e.to_string()));
    }
    env().render_str(template, ctx).map(Json::String).map_err(err)
}

/// Renders every string inside a JSON value (settings), recursively.
pub fn render_value(value: &Json, scope: &Scope) -> Result<Json, CoreError> {
    Ok(match value {
        Json::String(s) => render_str(s, scope)?,
        Json::Array(list) => Json::Array(list.iter().map(|v| render_value(v, scope)).collect::<Result<_, _>>()?),
        Json::Object(map) => {
            let mut out = Map::with_capacity(map.len());
            for (k, v) in map {
                out.insert(k.clone(), render_value(v, scope)?);
            }
            Json::Object(out)
        }
        other => other.clone(),
    })
}

/// Checks that an expression string compiles, without evaluating it.
pub fn validate(template: &str) -> Result<(), CoreError> {
    if let Some(inner) = single_expression(template.trim()) {
        env().compile_expression(inner).map(|_| ()).map_err(err)
    } else {
        env().template_from_str(template).map(|_| ()).map_err(err)
    }
}

fn single_expression(s: &str) -> Option<&str> {
    let inner = s.strip_prefix("{{")?.strip_suffix("}}")?;
    if inner.contains("{{") || inner.contains("}}") || inner.contains("{%") {
        return None;
    }
    Some(inner.trim())
}

fn err(e: minijinja::Error) -> CoreError {
    CoreError::Expression(e.to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn scope() -> Scope {
        Scope::new()
            .item(json!({"title": "Larkspur Mug", "price": 24, "tags": ["new", "ceramic"]}))
            .items(vec![json!({"title": "a"}), json!({"title": "b"})])
    }

    #[test]
    fn plain_strings_pass_through() {
        assert_eq!(render_str("hello", &scope()).unwrap(), json!("hello"));
    }

    #[test]
    fn single_expression_keeps_type() {
        assert_eq!(render_str("{{ item.price }}", &scope()).unwrap(), json!(24));
        assert_eq!(render_str("{{ item.tags }}", &scope()).unwrap(), json!(["new", "ceramic"]));
        assert_eq!(render_str("{{ item.missing }}", &scope()).unwrap(), Json::Null);
    }

    #[test]
    fn mixed_text_renders_string() {
        let out = render_str("New: {{ item.title }} for ${{ item.price }}", &scope()).unwrap();
        assert_eq!(out, json!("New: Larkspur Mug for $24"));
    }

    #[test]
    fn helpers_work() {
        assert_eq!(render_str("{{ item.tags | join(', ') }}", &scope()).unwrap(), json!("new, ceramic"));
        assert_eq!(render_str("{{ (items | first).title }}", &scope()).unwrap(), json!("a"));
        assert_eq!(render_str("{{ item.nope | default('x') }}", &scope()).unwrap(), json!("x"));
        assert_eq!(render_str("{{ item.tags | json }}", &scope()).unwrap(), json!("[\"new\",\"ceramic\"]"));
    }

    #[test]
    fn render_value_recurses() {
        let settings = json!({"input": "{{ item.title }}", "n": 3, "list": ["{{ item.price }}", "x"]});
        let out = render_value(&settings, &scope()).unwrap();
        assert_eq!(out, json!({"input": "Larkspur Mug", "n": 3, "list": [24, "x"]}));
    }

    #[test]
    fn validate_catches_syntax_errors() {
        assert!(validate("{{ item.title }}").is_ok());
        assert!(validate("{{ item.title ").is_err());
    }
}
