import {useEffect, useRef, useState} from 'react';
import * as maplibregl from 'maplibre-gl';
import workerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url';
import 'maplibre-gl/dist/maplibre-gl.css';
import type {MapSelection} from './MapView';
import GeoJSON from 'ol/format/GeoJSON';
import {incidentArea} from './mapGeometry';
import {headingRotation} from './tacticalMap';
import {useGpsTelemetry} from './Telemetry';
maplibregl.setWorkerUrl(workerUrl);
type Props={selection:MapSelection|null;source:string;track:number[][];onClose:()=>void};
export default function ExperimentalGlobe({selection,source,track,onClose}:Props){
 const host=useRef<HTMLDivElement>(null), mapRef=useRef<maplibregl.Map|null>(null);
 const [status,setStatus]=useState('실험 지도 초기화 중'); const [ready,setReady]=useState(false); const gps=useGpsTelemetry();
 const lat=gps.gps_lat,lon=gps.gps_lon;
 const live=lat?.status==='LIVE' && lon?.status==='LIVE' && lat.value!==null && lon.value!==null && Math.abs(lat.timestamp_ms-lon.timestamp_ms)<100 && lat.system_id===lon.system_id && lat.component_id===lon.component_id && Math.abs(lat.value)<=85 && Math.abs(lon.value)<=180;
 useEffect(()=>{if(!host.current)return;let map:maplibregl.Map;let timer:number|undefined;let observer:ResizeObserver|undefined;
 try{map=new maplibregl.Map({container:host.current,style:'https://tiles.openfreemap.org/styles/liberty',center:selection ? [selection.center[0],selection.center[1]] : [127,36],zoom:1.4});mapRef.current=map;
 timer=window.setTimeout(()=>setStatus('지도 로딩 시간 초과 · 기본 지도로 돌아가세요'),20000);
 map.on('load',()=>{window.clearTimeout(timer);map.setProjection({type:'globe'});setReady(true);setStatus('Global 실험 · OpenFreeMap / OpenStreetMap');});
 map.on('error',()=>setStatus('지도 리소스 오류 · 기본 지도로 돌아갈 수 있습니다'));
 map.on('webglcontextlost',()=>{setReady(false);setStatus('WebGL 연결 손실 · 기본 지도로 돌아가세요');});
 observer=new ResizeObserver(()=>map.resize());observer.observe(host.current);
 }catch{setStatus('WebGL 초기화 실패 · 기본 지도로 돌아가세요');return;}
 return ()=>{window.clearTimeout(timer);observer?.disconnect();map.remove();mapRef.current=null;};
 },[]);
 useEffect(()=>{const map=mapRef.current, app=host.current?.closest('.app');if(!map||!ready||!app)return;
  const original=new Map<string,unknown>();
  const apply=()=>{const dark=app.classList.contains('theme-dark');
   for(const layer of map.getStyle().layers){let property: 'background-color'|'fill-color'|'line-color'|'text-color'|''='',color:unknown;const id=layer.id;
    if(id.startsWith('firescene-'))continue;
    if(layer.type==='background'){property='background-color';color='#182027';}
    else if(layer.type==='fill'){property='fill-color';color=/water/.test(id)?'#243e50':/building/.test(id)?'#46515b':/park|wood|grass/.test(id)?'#253630':'#29333a';}
    else if(layer.type==='line' && /road|bridge|tunnel/.test(id)){property='line-color';color=/motorway|primary|trunk/.test(id)?'#ae926b':'#64717b';}
    else if(layer.type==='symbol' && layer.layout?.['text-field'] && !/road|shield|motorway|route/.test(id)){property='text-color';color='#dde4e9';}
    if(!property)continue;const key=id+':'+property;if(!original.has(key))original.set(key,map.getPaintProperty(id,property));
    try{map.setPaintProperty(id,property,(dark?color:original.get(key)) as never);if(property==='text-color'){const halo=id+':text-halo-color';if(!original.has(halo))original.set(halo,map.getPaintProperty(id,'text-halo-color'));map.setPaintProperty(id,'text-halo-color',(dark?'#182027':original.get(halo)) as never);}}catch{/* preserve provider layer if incompatible */}
   }
  };apply();const observer=new MutationObserver(apply);observer.observe(app,{attributes:true,attributeFilter:['class']});return ()=>observer.disconnect();
 },[ready]);
 useEffect(()=>{const map=mapRef.current;if(!map||!ready)return;
  const features: import('geojson').Feature[]=[];
  if(selection){
   try{if(selection.geometry) features.push({type:'Feature',properties:{kind:'building'},geometry:selection.geometry as import('geojson').Geometry});
    const area=incidentArea(selection.center,selection.radius);
    if(area) features.push({type:'Feature',properties:{kind:'radius'},geometry:new GeoJSON().writeGeometryObject(area,{featureProjection:'EPSG:3857',dataProjection:'EPSG:4326'}) as import('geojson').Geometry});
   }catch{setStatus('현장 외곽선 또는 범위를 표시할 수 없습니다');}
  }
  if(live && track.length>1) features.push({type:'Feature',properties:{kind:'track'},geometry:{type:'LineString',coordinates:track}});
  const data:import('geojson').FeatureCollection={type:'FeatureCollection',features};
  const source=map.getSource('firescene-mission') as maplibregl.GeoJSONSource|undefined;
  if(source){source.setData(data);return;}
  map.addSource('firescene-mission',{type:'geojson',data});
  map.addLayer({id:'firescene-mission-fill',type:'fill',source:'firescene-mission',filter:['!=',['get','kind'],'track'],paint:{'fill-color':'#ffba69','fill-opacity':0.12}});
  map.addLayer({id:'firescene-mission-line',type:'line',source:'firescene-mission',paint:{'line-color':['case',['==',['get','kind'],'track'],'#45b5e8','#ffba69'],'line-width':2}});
 },[selection,ready,live,track]);
 useEffect(()=>{const map=mapRef.current;if(!map||!ready)return;const markers:maplibregl.Marker[]=[];
 if(selection && Number.isFinite(selection.center[0]) && Number.isFinite(selection.center[1])){const el=document.createElement('div');el.className='globe-mission-marker';el.title=`임무 현장 · ${selection.name}`;markers.push(new maplibregl.Marker({element:el}).setLngLat([selection.center[0],selection.center[1]]).addTo(map));}
 if(live){const el=document.createElement('div');const rotation=headingRotation(gps.heading,lat);const icon=document.createElement('div');icon.className=rotation===null?'globe-aircraft-marker':'globe-aircraft-heading';el.appendChild(icon);el.title=source==='SIMULATION'?'시뮬레이션 기체 위치':'기체 위치 · 출처 미검증';markers.push(new maplibregl.Marker({element:el,rotationAlignment:'map',rotation:rotation===null?0:rotation*180/Math.PI}).setLngLat([lon.value!,lat.value!]).addTo(map));}
 return ()=>markers.forEach(m=>m.remove());
 },[selection,ready,live,lat?.value,lon?.value,source,gps.heading]);
 return <section className="globe-experiment" aria-label="Global 지도 실험"><div className="globe-toolbar"><strong>GLOBAL · 개발 실험</strong><button onClick={onClose}>기본 지도 복귀</button><button disabled={!ready||!selection} onClick={()=>selection&&mapRef.current?.flyTo({center:[selection.center[0],selection.center[1]],zoom:13})}>임무 위치 확대</button><button disabled={!ready} onClick={()=>mapRef.current?.setProjection({type:'mercator'})}>평면</button><button disabled={!ready} onClick={()=>{mapRef.current?.setProjection({type:'globe'});mapRef.current?.flyTo({center:[127,36],zoom:1.4});}}>지구본</button></div><div ref={host} className="globe-map"/><p role="status">{status} · {live ? source==='SIMULATION'?'시뮬레이션 기체':'기체 위치 · 출처 미검증':'기체 위치 미수신'} · 지도 조작은 비행 명령이 아닙니다.</p></section>;
}
