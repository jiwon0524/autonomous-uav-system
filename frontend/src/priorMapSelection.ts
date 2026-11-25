export type PriorSelection = {center:number[];radius:number};
export function priorTarget(selection:PriorSelection|null):{lon:number;lat:number;radius:number}|null {
 if (!selection || selection.center.length!==2) return null;
 const [lon,lat]=selection.center, radius=selection.radius;
 return Number.isFinite(lon)&&Number.isFinite(lat)&&Number.isFinite(radius)&&Math.abs(lon)<=180&&Math.abs(lat)<=85&&radius>=1&&radius<=2000 ? {lon,lat,radius} : null;
}
