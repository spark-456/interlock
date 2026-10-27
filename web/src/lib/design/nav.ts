// Primary navigation. PRD §7.3 routes; design styling (see design/interlock/MAPPING.md, App shell).

export interface NavItem {
  href: string;
  label: string;
  icon: string;
}

export const sidebarNav: NavItem[] = [
  { href: '/flows', label: 'Flows', icon: 'layers' },
  { href: '/templates', label: 'Templates', icon: 'grid' },
  { href: '/approvals', label: 'Approvals', icon: 'inbox' },
  { href: '/connections', label: 'Connections', icon: 'link' },
  { href: '/settings', label: 'Settings', icon: 'sliders' }
];

export const tabNav: NavItem[] = [
  { href: '/flows', label: 'Flows', icon: 'layers' },
  { href: '/new', label: 'New', icon: 'plus' },
  { href: '/approvals', label: 'Approvals', icon: 'inbox' },
  { href: '/settings', label: 'Settings', icon: 'sliders' }
];

/** Whether `href` is the active section for `path`. `/new/x/building` counts as Flows. */
export function isActive(href: string, path: string): boolean {
  if (href === '/new') return path === '/new';
  if (href === '/flows') return path.startsWith('/flows') || /^\/new\/[^/]+\/building/.test(path);
  return path === href || path.startsWith(`${href}/`);
}

/** Canvas screens collapse the sidebar to 64 px (design S3, S6). */
export function isCanvasRoute(routeId: string | null): boolean {
  return routeId === '/new/[id]/building' || routeId === '/flows/[id]';
}
