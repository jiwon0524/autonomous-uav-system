import { stationShell } from './platform/stationShell';
import { MissionHistory } from './history/MissionHistory';
import { updateMissionNotes } from './history/model';
import type { MissionRecord } from './history/model';
import { useSafety } from './safety/useSafety';
import { obstacleMarker } from './safety/model';
import type { MapMarker } from './navigation/model';
import { SafetyOverlay } from './safety/SafetyUI';
import { useSensors } from './sensors/store';
import { formatSensorEvent, targetDepthDistance } from './sensors/model';
import { SensorMonitor } from './sensors/SensorMonitor';
import { NavigationMap, MarkerDetail, ReturnDestinationDialog, SafePointSetup } from './navigation/NavigationUI';
import { initialMapState, mapReducer, emptyNavigationData, navigationMarkers } from './navigation/model';
import { TargetList, TargetDetail } from './targets/TargetManagement';
import { initialTargetState, targetReducer } from './targets/model';
import { useConnectionStatus } from './useConnectionStatus';
import { useReducer, useState } from 'react';
import { TopStatusBar, FlightStatus, FlightControls, ReturnSafety, EmergencyControl, AICamera, SensorStatus, SystemHealth, SystemConsole } from './components/DashboardPanels';
import { ActionModal, PositionControl } from './components/Interactions';
import type { Notice } from './components/Interactions';

export default function App() {
  const [screen,setScreen]=useState<'dashboard'|'history'>('dashboard');
  const [missions,setMissions]=useState<readonly MissionRecord[]>([]);
  const sensorView=stationShell.currentView()==='sensors';
  const {status,events}=useConnectionStatus(!sensorView);
  const {sensors,events:sensorEvents}=useSensors();
  const {state:safety,dispatch:dispatchSafety}=useSafety(status,sensors);
  const consoleEvents=[...events.map(event=>({...event,id:`bridge:${event.id}`})),...sensorEvents.map(event=>({id:`sensor:${event.id}`,text:formatSensorEvent(event)})),...safety.logs.map(event=>({id:`safety:${event.id}`,text:event.text}))];
  const [targetState,dispatchTarget]=useReducer(targetReducer,initialTargetState);
  const targetItems=Object.values(targetState.byId);
  const selectedTarget=targetState.selectedId?targetState.byId[targetState.selectedId]:undefined;
  const [mapState,dispatchMap]=useReducer(mapReducer,initialMapState);
  const [baseNavigationData]=useState(emptyNavigationData);
  const safetyMarkers=Object.values(safety.obstacles).map(obstacleMarker).filter((marker):marker is MapMarker=>marker!==null);
  const navigationData={...baseNavigationData,markers:[...baseNavigationData.markers,...safetyMarkers]};
  const [returnOpen,setReturnOpen]=useState(false);
  const selectedMarker=navigationMarkers(navigationData).find(marker=>marker.id===mapState.selectedMarkerId);
  const navigation={data:navigationData,state:mapState,dispatch:dispatchMap,onMove:()=>setNotice({title:'목표 위치',message:'실제 이동 시스템이 연결되어 있지 않습니다. 이동 명령은 전송되지 않았습니다.'})};
  const [notice,setNotice]=useState<Notice|null>(null);
  const onAction=(title:string,confirmation=false)=>{
    if(title==='SAFE POINT 설정'){
      if(!navigationData.sourceReady){setNotice({title,message:'현재 위치 데이터가 없어 SAFE POINT를 설정할 수 없습니다.'});return;}
      dispatchMap({type:'begin-safe-point'});return;
    }
    if(title==='안전 복귀'){setReturnOpen(true);return;}
    setNotice({title,confirmation,message:confirmation?`${title} 명령을 실행하시겠습니까?\n실제 비행 제어는 아직 연결되지 않았습니다.`:title==='자동 순찰'?'자동 순찰 기능은 향후 단계에서 구현됩니다.':'실제 비행 제어는 아직 연결되지 않았습니다.\n실제 비행 명령은 전송되지 않았습니다.'});
  };
  const openMonitor=()=>{
    if(!stationShell.openSensorMonitor())setNotice({title:'센서 모니터',message:'팝업이 차단되었습니다. 이 사이트의 팝업을 허용한 뒤 다시 열어 주세요.'});
  };
  if(sensorView)return <SensorMonitor/>;
  return <div className="app-shell"><SafetyOverlay state={safety} onDismiss={id=>dispatchSafety({type:'dismiss',id})}/><TopStatusBar onHistory={()=>setScreen('history')}/>{screen==='history'?<MissionHistory records={missions} onBack={()=>setScreen('dashboard')} onSaveNotes={(id,notes)=>setMissions(current=>updateMissionNotes(current,id,notes))}/>:<main><div className="workspace-heading"><div><span className="eyebrow">통합 관제 워크스페이스</span><h2>비행 관제</h2></div><div className="workspace-status"><i className="dot"/>비행 명령 비활성화<span className="divider"/>연결 상태 · PHASE 03-1</div></div><div className="dashboard-grid"><aside className="column left-column"><FlightStatus status={status}/><FlightControls onAction={onAction} onPosition={()=>dispatchMap({type:'panel',mode:'POSITION_CONTROL'})}/><ReturnSafety onAction={onAction} {...navigation} failsafe={safety.failsafe}/><EmergencyControl onAction={onAction}/></aside><div className="column center-column"><AICamera targetId={targetState.cameraTargetId} rgb={sensors.rgb}/><div className="target-grid"><TargetList variant="authorized" targets={targetItems} selectedId={targetState.selectedId} onSelect={id=>dispatchTarget({type:'select',id})}/><TargetList variant="unknown" targets={targetItems} selectedId={targetState.selectedId} onSelect={id=>dispatchTarget({type:'select',id})}/></div><div className="preview-note"><span className="note-dot"/>실제 명령은 전송되지 않으며 센서 및 지도는 연결되지 않았습니다.</div></div><aside className="column right-column">{mapState.panelMode==='NAVIGATION'?<NavigationMap {...navigation}/>:<PositionControl {...navigation} onBack={()=>dispatchMap({type:'panel',mode:'NAVIGATION'})} onAction={onAction}/>}<SensorStatus onOpen={openMonitor} sensors={sensors}/><SystemHealth health={safety.health}/></aside></div></main>}<SystemConsole events={consoleEvents}/>{returnOpen && !notice && <ReturnDestinationDialog data={navigationData} onClose={()=>setReturnOpen(false)} onConfirm={marker=>{setReturnOpen(false);setNotice({title:'안전 복귀',confirmation:true,message:`${marker.name}으로 안전 복귀하시겠습니까?\n실제 비행 제어는 연결되지 않았습니다.`});}}/>}{selectedMarker && !notice && !returnOpen && <MarkerDetail marker={selectedMarker} onClose={()=>dispatchMap({type:'select-marker',id:null})}/>}{mapState.safePointDraft.stage!=='idle' && !notice && <SafePointSetup {...navigation} onClose={()=>dispatchMap({type:'cancel-safe-point'})} onUnavailable={()=>setNotice({title:'SAFE POINT 설정',message:'실제 위치 저장 시스템이 연결되어 있지 않습니다. SAFE POINT를 생성하지 않았습니다.'})}/>}{selectedTarget && !notice && <TargetDetail distance={targetDepthDistance(sensors.depth,selectedTarget.id)} target={selectedTarget} onClose={()=>dispatchTarget({type:'select',id:null})} onCamera={()=>{dispatchTarget({type:'inspect-camera',id:selectedTarget.id});setNotice({title:'카메라에서 확인',message:'실제 카메라가 연결되어 있지 않습니다.'});}} onTrack={()=>{dispatchTarget({type:'request-tracking',id:selectedTarget.id});setNotice({title:'추적 시작',message:'실제 추적 시스템이 연결되어 있지 않습니다.'});}}/>}{notice && <ActionModal notice={notice} onClose={()=>setNotice(null)} onConfirm={()=>setNotice({title:notice.title,message:'실제 비행 제어는 아직 연결되지 않았습니다.\n실제 비행 명령은 전송되지 않았습니다.'})}/>}</div>;
}
