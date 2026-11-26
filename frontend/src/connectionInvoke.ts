import { invoke } from '@tauri-apps/api/core';

// Only development builds can install a QA interceptor for ordering tests.
export function invokeConnection<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (import.meta.env?.DEV) {
    const hook = (window as Window & { __firesceneQaInvoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown> }).__firesceneQaInvoke;
    if (hook) return hook(command, args) as Promise<T>;
  }
  return invoke<T>(command, args);
}
