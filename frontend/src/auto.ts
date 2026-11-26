import { invokeConnection } from './connectionInvoke';
export type Device = { vid: number; pid: number; serial: string; baud: number };
export type Port = { name: string; manufacturer: string | null; description: string | null; vid: number; pid: number; serial_number: string | null; access: string };
export const autoKey = 'firescene.auto.v1', lastKey = 'firescene.last.v1';
export const matches = (p: Port, d: Device) => p.vid === d.vid && p.pid === d.pid && p.serial_number === d.serial;
export function candidate(ports: Port[], device: Device) { const found = ports.filter(p => matches(p, device)); return { port: found.length === 1 ? found[0] : null, ambiguous: found.length > 1 }; }
export const shouldScanPorts = (phase: string) => phase !== 'CONNECTED';
export const retryDelay = (attempts: number) => [5000, 10000, 20000][attempts] ?? null;
export function load(key: string): Device | null { try { const d = JSON.parse(localStorage.getItem(key) || 'null'); return d && Number.isInteger(d.vid) && Number.isInteger(d.pid) && typeof d.serial === 'string' && (key === lastKey || d.serial.trim()) && [57600,115200,230400,460800,921600].includes(d.baud) ? d : null; } catch { return null; } }
export function save(key: string, d: Device | null) { try { if (d) localStorage.setItem(key, JSON.stringify(d)); else localStorage.removeItem(key); } catch { /* Storage may be unavailable. */ } }
let pending: Promise<unknown> = Promise.resolve();
export function ordered<T>(fn: () => Promise<T>): Promise<T> { const next = pending.then(fn, fn); pending = next.then(() => undefined, () => undefined); return next; }
export async function setPermission(device: Device | null) { await ordered(() => invokeConnection('set_auto_device', { device })); }
