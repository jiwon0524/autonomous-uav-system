export async function withConnectionDeadline<T>(request:Promise<T>, timeoutMs:number):Promise<T> {
 let timer:ReturnType<typeof setTimeout>|undefined;
 try {return await Promise.race([request,new Promise<never>((_,reject)=>{timer=setTimeout(()=>reject(new Error('연결 상태 조회 시간 초과')),timeoutMs);})]);}
 finally {if(timer!==undefined)clearTimeout(timer);}
}
