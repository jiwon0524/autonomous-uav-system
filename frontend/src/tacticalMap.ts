/** Only draw a heading when it belongs to the live position's vehicle. */
export function headingRotation(heading: {status:string; value:number|null; system_id:number; component_id:number}|undefined, position: {system_id:number; component_id:number}): number|null {
  if (!heading || heading.status !== 'LIVE' || heading.value === null || !Number.isFinite(heading.value) || heading.system_id !== position.system_id || heading.component_id !== position.component_id) return null;
  return heading.value * Math.PI / 180;
}
