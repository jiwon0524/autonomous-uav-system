export function formatDateTime(value:string|null|undefined):string {
  return value && Number.isFinite(Date.parse(value))?new Date(value).toLocaleString('ko-KR'):'--';
}
