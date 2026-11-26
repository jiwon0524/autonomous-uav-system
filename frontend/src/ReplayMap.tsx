import { useEffect, useRef, useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import Map from 'ol/Map';
import View from 'ol/View';
import TileLayer from 'ol/layer/Tile';
import VectorLayer from 'ol/layer/Vector';
import XYZ from 'ol/source/XYZ';
import VectorSource from 'ol/source/Vector';
import Feature from 'ol/Feature';
import Point from 'ol/geom/Point';
import LineString from 'ol/geom/LineString';
import {fromLonLat} from 'ol/proj';
import {Circle as CircleStyle, Fill, Stroke, Style} from 'ol/style';
import type {ReplayRecord} from './Recordings';

export function replayPoint(record:ReplayRecord):number[]|null {
  if(record.type!=='sample'||record.snapshot?.phase!=='CONNECTED') return null;
  const f=record.snapshot.telemetry?.fields, lat=f?.gps_lat, lon=f?.gps_lon;
  if(!lat||!lon||lat.status!=='LIVE'||lon.status!=='LIVE'||lat.value===null||lon.value===null||!Number.isFinite(lat.value)||!Number.isFinite(lon.value)||Math.abs(lat.value)>85||Math.abs(lon.value)>180||lat.system_id!==lon.system_id||lat.component_id!==lon.component_id||Math.abs(lat.timestamp_ms-lon.timestamp_ms)>=100) return null;
  return [lon.value,lat.value];
}
export function ReplayMap({records,position}:{records:ReplayRecord[];position:number}) {
  const target=useRef<HTMLDivElement>(null), map=useRef<Map|null>(null), features=useRef(new VectorSource()); const [key,setKey]=useState(''); const [mapStatus,setMapStatus]=useState('지도 준비');
  useEffect(()=>{let live=true;if(isTauri()) void invoke<string>('load_map_key').then(k=>{if(live)setKey(k);}).catch(()=>{});return()=>{live=false;};},[]);
  useEffect(()=>{if(!key||!target.current)return;const tiles=new XYZ({url:`https://api.vworld.kr/req/wmts/1.0.0/${encodeURIComponent(key)}/Base/{z}/{y}/{x}.png`,attributions:'© VWorld',maxZoom:19});let failed=false;tiles.on('tileloaderror',()=>{failed=true;setMapStatus('지도 타일 오류 · 연결 또는 인증 확인');});tiles.on('tileloadend',()=>{if(!failed)setMapStatus('지도 타일 수신');});const instance=new Map({target:target.current,layers:[new TileLayer({source:tiles}),new VectorLayer({source:features.current})],view:new View({center:fromLonLat([126.978,37.5665]),zoom:14})});map.current=instance;const observer=new ResizeObserver(()=>instance.updateSize());observer.observe(target.current);return()=>{observer.disconnect();instance.setTarget(undefined);instance.dispose();map.current=null;};},[key]);
  const positionBucket=Math.floor(position/500)*500;
  useEffect(()=>{const position=positionBucket;const vector=features.current;vector.clear();let segment:number[][]=[];let previous=-1;let last:number[]|null=null;let lastTime=-1;
    const draw=()=>{if(segment.length>1){const f=new Feature(new LineString(segment));f.setStyle(new Style({stroke:new Stroke({color:'#e3b977',width:3})}));vector.addFeature(f);}segment=[];};
    records.filter(r=>r.type==='sample'&&r.elapsed_ms<=position).slice(-500).forEach(r=>{const p=replayPoint(r);if(!p||previous>=0&&r.elapsed_ms-previous>2500){draw();last=null;}if(p){const projected=fromLonLat(p);segment.push(projected);last=projected;lastTime=r.elapsed_ms;}previous=r.elapsed_ms;});draw();
    if(last && position-lastTime<=2500){const f=new Feature(new Point(last));f.setStyle(new Style({image:new CircleStyle({radius:7,fill:new Fill({color:'#e3b977'}),stroke:new Stroke({color:'#171b19',width:2})})}));vector.addFeature(f);map.current?.getView().setCenter(last);}
  },[records,positionBucket,key]);
  return <div className="replay-map"><div className="replay-map-head">REPLAY · GPS 원시 위치 · 현재 10분 구간 · 지도 출처 VWorld · {mapStatus}</div><div className="replay-map-canvas" ref={target}>{!key&&<div className="map-empty">지도 키가 없거나 지도 설정을 불러올 수 없습니다.</div>}</div></div>;
}
