// Toast queue. `toast('Copied')` from anywhere; <Toast /> in the layout renders it.

export type ToastTone = 'info' | 'success' | 'error';
export interface ToastItem {
  id: number;
  message: string;
  tone: ToastTone;
}

export const toasts = $state<{ list: ToastItem[] }>({ list: [] });
let next = 1;

/** Shows a toast for `ms` milliseconds. */
export function toast(message: string, tone: ToastTone = 'info', ms = 4000): number {
  const id = next++;
  toasts.list.push({ id, message, tone });
  setTimeout(() => dismiss(id), ms);
  return id;
}

/** Removes a toast. */
export function dismiss(id: number): void {
  toasts.list = toasts.list.filter((t) => t.id !== id);
}
