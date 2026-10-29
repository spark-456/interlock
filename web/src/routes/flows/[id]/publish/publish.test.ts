import { describe, expect, it } from 'vitest';
import { endpoints, mcpInstall, platformMcpInstall, quickStart, slug } from './publish';

describe('publish', () => {
  it('slugs names the way il-publish does', () => {
    expect(slug('Larkspur Support')).toBe('larkspur-support');
    expect(slug('  FAQ bot — v2!! ')).toBe('faq-bot-v2');
  });

  it('builds the URLs and snippets for an app', () => {
    const e = endpoints('https://il.example', 'Larkspur Support');
    expect(e).toEqual({
      base: 'https://il.example/v1',
      model: 'larkspur-support',
      mcp: 'https://il.example/mcp/larkspur-support',
      platformMcp: 'https://il.example/mcp',
      widget: '<script src="https://il.example/w/larkspur-support.js" async></script>'
    });
    const tabs = quickStart(e, 'Can I get a refund?');
    expect(tabs.map((t) => t.label)).toEqual(['cURL', 'JavaScript', 'Python', 'Rust', 'Claude Code']);
    for (const t of tabs.slice(0, 4)) expect(t.code).toContain('larkspur-support');
    expect(tabs[0]?.code).toContain('"Can I get a refund?"');
    expect(mcpInstall(e)[0]?.text).toContain('claude mcp add --transport http larkspur-support https://il.example/mcp/larkspur-support');
    expect(JSON.parse(mcpInstall(e)[1]!.text).mcpServers['larkspur-support'].url).toBe(e.mcp);
  });

  it('P02 includes the bearer header in the platform MCP command', () => {
    const e = endpoints('https://il.example', 'Larkspur Support');
    const command = platformMcpInstall(e);
    expect(command).toContain('https://il.example/mcp --header "Authorization: Bearer $INTERLOCK_KEY"');
    expect(quickStart(e, 'Hello').find((tab) => tab.value === 'claude')?.code).toContain(command);
  });
});
