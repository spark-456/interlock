import { describe, expect, it } from 'vitest';
import { markdown } from './markdown';

describe('markdown', () => {
  it('renders what agents write', () => {
    expect(markdown('The **Team plan** costs *$48*.\nSee [Plans](https://help.example/plans).')).toBe(
      '<p>The <strong>Team plan</strong> costs <em>$48</em>.<br>See <a href="https://help.example/plans" target="_blank" rel="noopener noreferrer">Plans</a>.</p>'
    );
    expect(markdown('### CLI\n- one\n- `two`\n\n1. first\n2. second')).toBe(
      '<h4>CLI</h4><ul><li>one</li><li><code>two</code></li></ul><ol><li>first</li><li>second</li></ol>'
    );
    expect(markdown('```bash\ntvly crawl --limit 10\n```')).toBe('<pre><code>tvly crawl --limit 10</code></pre>');
  });

  it('renders tables', () => {
    expect(markdown('| Parameter | Does |\n|---|:--|\n| `limit` | caps pages |')).toBe(
      '<table><thead><tr><th>Parameter</th><th>Does</th></tr></thead><tbody><tr><td><code>limit</code></td><td>caps pages</td></tr></tbody></table>'
    );
  });

  it('never lets model text add markup', () => {
    expect(markdown('<img src=x onerror=alert(1)> **<b>hi</b>**')).toBe(
      '<p>&lt;img src=x onerror=alert(1)&gt; <strong>&lt;b&gt;hi&lt;/b&gt;</strong></p>'
    );
    expect(markdown('[x](javascript:alert(1))')).toBe('<p>[x](javascript:alert(1))</p>');
  });

  it('keeps asterisks that are not emphasis', () => {
    expect(markdown('2 * 3 * 4')).toBe('<p>2 * 3 * 4</p>');
  });
});
