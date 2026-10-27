// Node tile tone + icon from PRD catalog data (see design/interlock/MAPPING.md, Node categories).
// The design's tones are roles in an AI app; PRD categories map onto them here.

export type Tone =
  | 'source' | 'kb' | 'router' | 'agent' | 'tool' | 'guard' | 'tests' | 'output'
  | 'trigger' | 'media' | 'logic' | 'app';

export const TONES: readonly Tone[] = [
  'source', 'kb', 'router', 'agent', 'tool', 'guard', 'tests', 'output', 'trigger', 'media', 'logic', 'app'
];

export function isTone(s: string | undefined | null): s is Tone {
  return !!s && (TONES as readonly string[]).includes(s);
}

/** CSS variables for a tone's tile background, icon colour and tile border. */
export function toneVars(tone: Tone): { tile: string; icon: string; edge: string } {
  return { tile: `var(--cat-${tone}-tile)`, icon: `var(--cat-${tone}-icon)`, edge: `var(--cat-${tone}-edge)` };
}

const ROUTERS = new Set(['ai.decide', 'ai.classify', 'logic.switch', 'logic.if']);

/** Default tone for a node type id. */
export function toneFor(typeId: string): Tone {
  if (typeId === 'build.tests') return 'tests';
  if (ROUTERS.has(typeId)) return 'router';
  if (typeId === 'knowledge.add' || typeId === 'data.parse_document') return 'source';
  if (typeId === 'data.openapi' || typeId === 'data.http') return 'tool';
  if (typeId.startsWith('app.')) return 'app';
  const prefix = typeId.split('.')[0];
  switch (prefix) {
    case 'knowledge': return 'kb';
    case 'ai': return 'agent';
    case 'trigger': return 'trigger';
    case 'media': return 'media';
    case 'logic': return 'logic';
    case 'data': return 'source';
    case 'human': return 'guard';
    case 'output': return 'output';
    default: return 'logic';
  }
}

const ICONS: Record<string, string> = {
  'knowledge.base': 'db', 'knowledge.search': 'search', 'knowledge.add': 'docs', 'data.parse_document': 'docs',
  'data.openapi': 'wrench', 'data.http': 'globe', 'ai.agent': 'agent', 'ai.decide': 'split', 'ai.classify': 'split',
  'ai.write': 'pen', 'ai.generate_video': 'film', 'ai.generate_image': 'image', 'ai.generate_voice': 'mic',
  'ai.transcribe': 'mic', 'logic.switch': 'split', 'logic.if': 'split', 'logic.loop': 'repeat', 'logic.wait': 'timer',
  'human.approval': 'shield', 'output.respond': 'chat', 'output.notify': 'bell', 'trigger.schedule': 'clock',
  'trigger.webhook': 'webhook', 'trigger.manual': 'hand', 'trigger.chat': 'chat', 'trigger.api_call': 'api',
  'trigger.mcp_call': 'mcp', 'media.cut_clips': 'scissors', 'build.tests': 'tests'
};

/** Default icon key for a node type id (manifest icon keys are used when they exist in the set). */
export function iconFor(typeId: string, manifestIcon?: string): string {
  return ICONS[typeId] ?? manifestIcon ?? 'box';
}
