import type {ReplayRecord} from './Recordings';

// Holds only an actually recorded, recent and connected sample; never interpolates.
export function replaySample(records:ReplayRecord[],position:number):ReplayRecord|undefined {
  for(let i=records.length-1;i>=0;i--) {
    const sample=records[i];
    if(sample.type==='sample' && sample.elapsed_ms<=position) return position-sample.elapsed_ms<=2500 && sample.snapshot?.phase==='CONNECTED' ? sample : undefined;
  }
  return undefined;
}
export function advanceReplay(position:number,elapsed:number,speed:number,duration:number):number {
  if (![position,elapsed,speed,duration].every(Number.isFinite) || elapsed<0 || ![0.5,1,2,4].includes(speed) || duration<0) return position;
  return Math.max(0,Math.min(duration,position+elapsed*speed));
}
