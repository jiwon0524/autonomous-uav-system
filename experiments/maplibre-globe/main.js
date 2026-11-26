import * as maplibregl from 'maplibre-gl';
import workerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url';
import 'maplibre-gl/dist/maplibre-gl.css';
import './style.css';
maplibregl.setWorkerUrl(workerUrl);
const app=document.querySelector('#app');
app.innerHTML='<header><strong>MAPLIBRE · 격리 실험</strong><span>OpenFreeMap / OpenStreetMap · 실험</span><button id="theme" disabled>Dark</button><button id="flat" disabled>평면</button><button id="globe" disabled>지구본</button><button id="site" disabled>부산 현장</button></header><main id="map"></main><p id="status" role="status">초기화 중</p><pre id="diagnostics"></pre>';
const status=document.querySelector('#status'), diagnostics=document.querySelector('#diagnostics');
const local=new URLSearchParams(location.search).get('source')==='local';
const source=local?{version:8,sources:{},layers:[{id:'background',type:'background',paint:{'background-color':'#263d4a'}}]}:'https://tiles.openfreemap.org/styles/liberty';
const log=[]; let map,loaded=false, dark=false; const original=new Map(); const started=performance.now();
function event(name,detail='') {log.push(`${Math.round(performance.now()-started)}ms ${name} ${detail}`); diagnostics.textContent=log.slice(-10).join('\n');}
const timer=setTimeout(()=>{if(!loaded){status.textContent='지도 초기화 시간 초과 · 네트워크, 워커와 WebGL 확인';event('timeout');}},20000);
try {
 map=new maplibregl.Map({container:'map',center:[129.059,35.154],zoom:12,style:source,attributionControl:true});
 for(const name of ['style.load','load','idle','webglcontextlost','webglcontextrestored']) map.on(name,()=>{event(name); if(name==='load'){loaded=true;clearTimeout(timer);document.querySelectorAll('header button').forEach(button=>button.disabled=false);status.textContent=local?'평면 진단 완료 · 단색 실험':'지도 로딩 완료 · OpenFreeMap 데이터';} if(name==='webglcontextlost')status.textContent='WebGL 연결 손실';});
 map.on('error',e=>{event('error',e.error?.message||'unknown');status.textContent='지도 리소스 오류 · 진단 내용 확인';});
 document.querySelector('#theme').onclick=()=>{
  if(!loaded || local) return;
  dark=!dark;
  for(const layer of map.getStyle().layers){
   let property, color; const id=layer.id;
   if(layer.type==='background'){property='background-color';color='#182027';}
   else if(layer.type==='fill'){property='fill-color'; color=/water/.test(id)?'#243e50':/building/.test(id)?'#46515b':/park|wood|grass/.test(id)?'#253630':'#29333a';}
   else if(layer.type==='line' && /road|bridge|tunnel/.test(id)){property='line-color';color=/motorway|primary|trunk/.test(id)?'#ae926b':'#64717b';}
   else if(layer.type==='symbol' && layer.layout?.['text-field'] && !/road|shield|motorway|route/.test(id)){property='text-color';color='#dde4e9';}
   if(!property)continue;
   const key=id+':'+property;
   if(!original.has(key))original.set(key,map.getPaintProperty(id,property));
   try{map.setPaintProperty(id,property,dark?color:original.get(key));}catch(e){event('style error',id);}
  }
  document.querySelector('#theme').textContent=dark?'Day':'Dark';event('theme',dark?'dark':'day');
 };
 document.querySelector('#flat').onclick=()=>{map.setProjection({type:'mercator'});event('projection','mercator');};
 document.querySelector('#globe').onclick=()=>{map.setProjection({type:'globe'});map.flyTo({center:[127,36],zoom:1.3});event('projection','globe');};
 document.querySelector('#site').onclick=()=>map.flyTo({center:[129.059,35.154],zoom:14});
 window.addEventListener('pagehide',()=>{clearTimeout(timer);map.remove();},{once:true});
 event('canvas',`${map.getCanvas().width}x${map.getCanvas().height}`);
} catch(e){clearTimeout(timer);status.textContent='WebGL 초기화 실패';event('exception',String(e));}
