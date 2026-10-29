import { dev } from '$app/environment';
import { error } from '@sveltejs/kit';

// The component gallery is a development tool; production builds don't serve it.
export function load() {
  if (!dev) error(404, 'Not found');
}
