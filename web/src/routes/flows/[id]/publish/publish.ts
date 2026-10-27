// URLs and copy-paste snippets for a published AI app. The slug matches il-publish's.

/** "Larkspur Support" → "larkspur-support". */
export function slug(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

export interface Endpoints {
  base: string;
  model: string;
  mcp: string;
  platformMcp: string;
  widget: string;
}

/** Where an app answers, for a server at `origin`. */
export function endpoints(origin: string, name: string): Endpoints {
  const model = slug(name);
  return {
    base: `${origin}/v1`,
    model,
    mcp: `${origin}/mcp/${model}`,
    platformMcp: `${origin}/mcp`,
    widget: `<script src="${origin}/w/${model}.js" async></script>`
  };
}

/** Commands and config that add the app's MCP server to a coding tool. */
export function mcpInstall(e: Endpoints) {
  const json = JSON.stringify({ mcpServers: { [e.model]: { url: e.mcp, headers: { Authorization: 'Bearer ${INTERLOCK_KEY}' } } } }, null, 2);
  return [
    { id: 'claude', label: 'Add to Claude Code', text: `claude mcp add --transport http ${e.model} ${e.mcp} --header "Authorization: Bearer $INTERLOCK_KEY"` },
    { id: 'cursor', label: 'Add to Cursor', text: json },
    { id: 'vscode', label: 'Add to VS Code', text: JSON.stringify({ servers: { [e.model]: { type: 'http', url: e.mcp } } }, null, 2) }
  ];
}

/** Quick-start code for each tab. */
export function quickStart(e: Endpoints, question: string): { value: string; label: string; code: string }[] {
  const q = JSON.stringify(question);
  return [
    {
      value: 'curl',
      label: 'cURL',
      code: `curl ${e.base}/chat/completions \\
  -H "Authorization: Bearer $INTERLOCK_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"model": "${e.model}", "messages": [{"role": "user", "content": ${q}}]}'`
    },
    {
      value: 'js',
      label: 'JavaScript',
      code: `import OpenAI from "openai";
const client = new OpenAI({ baseURL: "${e.base}", apiKey: process.env.INTERLOCK_KEY });
const res = await client.chat.completions.create({
  model: "${e.model}",
  messages: [{ role: "user", content: ${q} }],
});`
    },
    {
      value: 'python',
      label: 'Python',
      code: `from openai import OpenAI
import os
client = OpenAI(base_url="${e.base}", api_key=os.environ["INTERLOCK_KEY"])
res = client.chat.completions.create(
    model="${e.model}",
    messages=[{"role": "user", "content": ${q}}],
)`
    },
    {
      value: 'rust',
      label: 'Rust',
      code: `let res = reqwest::Client::new()
    .post("${e.base}/chat/completions")
    .bearer_auth(std::env::var("INTERLOCK_KEY")?)
    .json(&serde_json::json!({
        "model": "${e.model}",
        "messages": [{ "role": "user", "content": ${q} }],
    }))
    .send()
    .await?;`
    },
    {
      value: 'claude',
      label: 'Claude Code',
      code: `claude mcp add --transport http interlock ${e.platformMcp}
# then ask: "Add the ${e.model} assistant as a chat panel in this app."`
    }
  ];
}
