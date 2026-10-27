// The embeddable chat widget: `<script src="https://host/w/<app>.js" async></script>`.
// Theming via data attributes: data-title, data-greeting, data-accent (any CSS colour), data-position="left".
import tokens from '../src/lib/design/tokens.css?inline';
import { chat, type Message, type Source } from './stream';

const STYLE = `
:host { all: initial; position: fixed; bottom: 20px; right: 20px; z-index: 2147483000; font: 14px/1.45 var(--font-sans); color: var(--text); }
:host([data-position="left"]) { right: auto; left: 20px; }
button { font: inherit; cursor: pointer; }
:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
.launch { width: 52px; height: 52px; border: 0; border-radius: 50%; background: var(--primary); color: var(--surface); font-size: 22px; box-shadow: var(--shadow-lg); }
.panel { position: absolute; bottom: 64px; right: 0; width: min(360px, calc(100vw - 40px)); height: min(520px, calc(100vh - 120px)); display: flex; flex-direction: column; background: var(--surface); border: 1px solid var(--border); border-radius: 14px; box-shadow: var(--shadow-lg); overflow: hidden; }
:host([data-position="left"]) .panel { right: auto; left: 0; }
.panel[hidden] { display: none; }
header { display: flex; align-items: center; justify-content: space-between; padding: 12px 14px; border-bottom: 1px solid var(--border); font-weight: 600; }
header button { border: 0; background: none; color: var(--text-3); font-size: 18px; }
ol { flex: 1; margin: 0; padding: 14px; list-style: none; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; }
li { max-width: 85%; padding: 8px 12px; border-radius: 12px; white-space: pre-wrap; overflow-wrap: anywhere; }
.assistant { align-self: flex-start; background: var(--surface-2); }
.user { align-self: flex-end; background: var(--primary); color: var(--surface); }
.error { align-self: stretch; max-width: none; background: var(--error-soft); color: var(--error-text); }
.sources { margin-top: 6px; font-size: 12px; color: var(--text-3); }
.sources a { color: var(--primary); margin-right: 8px; }
form { display: flex; gap: 8px; padding: 10px; border-top: 1px solid var(--border); }
input { flex: 1; font: inherit; color: inherit; padding: 8px 10px; border: 1px solid var(--border-strong); border-radius: 8px; background: var(--surface); }
form button { border: 0; border-radius: 8px; padding: 0 14px; background: var(--primary); color: var(--surface); font-weight: 600; }
form button:disabled { opacity: .5; cursor: default; }
`;

function element<K extends keyof HTMLElementTagNameMap>(tag: K, props: object = {}): HTMLElementTagNameMap[K] {
  return Object.assign(document.createElement(tag), props);
}

function showSources(bubble: HTMLElement, sources: Source[]) {
  const links = sources.filter((s) => s.url && /^https?:\/\//.test(s.url));
  if (!links.length) return;
  bubble.querySelector('.sources')?.remove();
  const list = element('div', { className: 'sources', textContent: 'Sources: ' });
  for (const s of links) list.append(element('a', { href: s.url, textContent: s.title || s.url, target: '_blank', rel: 'noopener noreferrer' }));
  bubble.append(list);
}

function mount(script: HTMLScriptElement) {
  const src = new URL(script.src);
  const app = decodeURIComponent(src.pathname.split('/').pop() ?? '').replace(/\.js$/, '');
  const endpoint = new URL(`/w/${encodeURIComponent(app)}/chat`, src).href;
  const { title = 'Ask us', greeting, accent, position } = script.dataset;

  const host = element('div');
  if (position === 'left') host.dataset.position = 'left';
  const root = host.attachShadow({ mode: 'open' });
  const style = element('style', { textContent: tokens.replaceAll(':root', ':host') + STYLE });
  if (accent) host.style.setProperty('--primary', accent);

  const launch = element('button', { className: 'launch', type: 'button', textContent: '💬' });
  launch.setAttribute('aria-label', `Open ${title}`);
  launch.setAttribute('aria-expanded', 'false');
  const panel = element('section', { className: 'panel', hidden: true });
  panel.setAttribute('aria-label', title);
  const close = element('button', { type: 'button', textContent: '×' });
  close.setAttribute('aria-label', 'Close');
  const log = element('ol');
  log.setAttribute('aria-live', 'polite');
  const input = element('input', { type: 'text', placeholder: 'Type a message…', autocomplete: 'off' });
  input.setAttribute('aria-label', 'Message');
  const send = element('button', { type: 'submit', textContent: 'Send' });
  const form = element('form');
  form.append(input, send);
  panel.append(element('header', { textContent: title }), log, form);
  panel.querySelector('header')?.append(close);
  root.append(style, launch, panel);
  document.body.append(host);

  const history: Message[] = [];
  const bubble = (className: string, text: string) => {
    const li = element('li', { className, textContent: text });
    log.append(li);
    log.scrollTop = log.scrollHeight;
    return li;
  };
  if (greeting) bubble('assistant', greeting);

  const toggle = (open: boolean) => {
    panel.hidden = !open;
    launch.setAttribute('aria-expanded', String(open));
    if (open) input.focus();
  };
  launch.addEventListener('click', () => toggle(panel.hidden));
  close.addEventListener('click', () => {
    toggle(false);
    launch.focus();
  });

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    const text = input.value.trim();
    if (!text || send.disabled) return;
    input.value = '';
    send.disabled = true;
    bubble('user', text);
    history.push({ role: 'user', content: text });
    const answer = bubble('assistant', '');
    const body = answer.appendChild(document.createTextNode(''));
    try {
      await chat(endpoint, app, history, ({ text, sources }) => {
        body.data += text;
        showSources(answer, sources);
        log.scrollTop = log.scrollHeight;
      });
      history.push({ role: 'assistant', content: body.data });
    } catch (error) {
      answer.remove();
      bubble('error', error instanceof Error ? error.message : 'Something went wrong.');
    } finally {
      send.disabled = false;
      input.focus();
    }
  });
}

const script = document.currentScript;
if (script instanceof HTMLScriptElement) {
  if (document.body) mount(script);
  else document.addEventListener('DOMContentLoaded', () => mount(script));
}
