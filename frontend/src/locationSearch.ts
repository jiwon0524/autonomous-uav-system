export type SearchKind = 'road'|'parcel'|'place';
export type SearchItem = { title?: string; address?: {road?: string; parcel?: string; bldnm?: string}; point: {x:string; y:string}; category?: string; kind:SearchKind; candidate?:boolean };
export type SearchResponse = {status?:string; error?:{code?:string; text?:string}; result?:{items?:Omit<SearchItem,'kind'>[]}};
export const normalizeQuery = (s:string) => s.normalize('NFC').trim().replace(/\s+/g,' ');
const compact = (s:string) => normalizeQuery(s).replace(/\s/g,'').toLocaleLowerCase();
export function alternateQuery(query:string):string|null {
 const q=normalizeQuery(query), c=compact(q);
 if (c.endsWith('대학고')) return c.slice(0,-3)+'대학교';
 if (c.endsWith('대')) return c+'학교';
 if (q.includes(' ')) return q.replace(/\s/g,'');
 return null;
}
const valid = (i:SearchItem) => i.point && i.point.x !== '' && i.point.y !== '' && Number.isFinite(Number(i.point.x)) && Number.isFinite(Number(i.point.y)) && Math.abs(Number(i.point.x))<=180 && Math.abs(Number(i.point.y))<=85;
export function rank(item:SearchItem, query:string):number {
 if(item.candidate) return 4;
 const values=[item.title,item.address?.bldnm,item.address?.road,item.address?.parcel].filter((v):v is string=>!!v);
 if(values.some(v=>normalizeQuery(v).toLocaleLowerCase()===normalizeQuery(query).toLocaleLowerCase())) return 0;
 if(values.some(v=>compact(v)===compact(query))) return 1;
 if(values.some(v=>compact(v).includes(compact(query)))) return 2;
 return 3;
}
export function mergeResults(items:SearchItem[],query:string):SearchItem[] {
 const result:SearchItem[]=[];
 for(const item of items.filter(valid).sort((a,b)=>rank(a,query)-rank(b,query))) {
  const existing=result.find(x=>Number(x.point.x)===Number(item.point.x) && Number(x.point.y)===Number(item.point.y) && ((!!x.title && !!item.title && compact(x.title)===compact(item.title)) || (!!x.address?.road && compact(x.address.road)===compact(item.address?.road||'')) || (!!x.address?.parcel && compact(x.address.parcel)===compact(item.address?.parcel||''))));
  if(!existing) result.push(item);
 }
 return result;
}
export function searchError(code:string|undefined):string {
 const c=(code||'').toUpperCase();
 if(c.includes('LIMIT')||c.includes('QUOTA')||c==='429') return '검색 호출 제한 · 잠시 후 다시 시도하세요';
 if(c.includes('AUTH')||c.includes('KEY')||c==='401'||c==='403') return '인증 실패 · 키와 도메인을 확인하세요';
 return '검색 서버 오류';
}

// A cancelled or superseded response cannot publish into the current view.
export function createSearchGate() {
 let generation=0;
 return {start:()=>++generation, cancel:()=>{generation++;}, isCurrent:(id:number)=>id===generation};
}
