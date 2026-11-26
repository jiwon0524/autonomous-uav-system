export function recentFrameRate(rate:number|null|undefined, sampledAt:number|null|undefined, now:number, active:boolean):string {
  if (!active || rate == null || !Number.isFinite(rate) || rate < 0 || sampledAt == null || now < sampledAt || now - sampledAt > 4000) return '확인 불가';
  return `${rate.toFixed(1)} 프레임/초`;
}
export function heartbeatInterval(ms:number|null|undefined, active:boolean):string {
  return active && ms != null && Number.isFinite(ms) && ms >= 0 ? `${(ms/1000).toFixed(2)}초` : '확인 불가';
}
