import { MapView, MapSettings, type MapSelection } from './MapView';
import { FirstRun } from './FirstRun';
import { initialTheme, needsFirstRun, saveTheme, type Theme } from './setupPreferences';
import { Companion } from './Companion';
import { ManualPreview } from './ManualPreview';
import { RecordingBadge } from './RecordingStatus';
import { SafetyBanner, SafetyPage } from './Safety';
import { LiveStrip, OverviewTelemetry, OverviewFlightStrip, LiveEvents, LiveInspector, FlightInstruments } from './Telemetry';
import { ConnectionSettings, phaseText, sourceText, useConnection } from './Connection';
import { lazy, Suspense, useCallback, useState, type ReactNode } from 'react';
import { missionAfterMapChange, type SelectedMission } from './workflow';
const Spatial = lazy(() => import('./Spatial').then(m => ({default:m.Spatial})));
const Camera = lazy(() => import('./Camera').then(m => ({default:m.Camera})));
const MissionPage = lazy(() => import('./Missions').then(m => ({default:m.MissionPage})));
const Recordings = lazy(() => import('./Recordings').then(m => ({default:m.Recordings})));
const PageLoading = () => <div className="placeholder" role="status">화면 불러오는 중</div>;
import { version } from '../package.json';

const navigation = [
  ['overview', '종합 관제'], ['flight', '비행 제어'], ['mission', '임무 계획'],
  ['auto', '자율비행'], ['mapping', '3D 매핑'], ['camera', '카메라'],
  ['safety', '안전 관리'], ['logs', '비행 기록'], ['settings', '시스템 설정'],
] as const;
function Icon({ name, size = 18 }: { name: string; size?: number }) {
  const paths: Record<string, ReactNode> = {
    overview: <><path d="M4 5h16v14H4z"/><path d="M8 9h8M8 13h5"/></>,
    flight: <path d="m12 2 3 8 7 2-7 2-3 8-3-8-7-2 7-2 3-8Z"/>,
    mission: <><circle cx="6" cy="18" r="2"/><circle cx="18" cy="6" r="2"/><path d="M8 18c6 0 4-10 8-11"/></>,
    auto: <><rect x="4" y="5" width="16" height="14" rx="3"/><path d="M9 10h.01M15 10h.01M9 15h6M12 2v3"/></>,
    mapping: <><path d="m3 6 6-3 6 3 6-3v15l-6 3-6-3-6 3V6Z"/><path d="M9 3v15M15 6v15"/></>,
    camera: <><path d="M4 8h4l2-2h4l2 2h4v11H4z"/><circle cx="12" cy="13" r="3"/></>,
    safety: <><path d="M12 3 4 6v5c0 5 3.5 8.5 8 10 4.5-1.5 8-5 8-10V6l-8-3Z"/><path d="m9 12 2 2 4-4"/></>,
    logs: <><path d="M5 3h14v18H5z"/><path d="M8 8h8M8 12h8M8 16h5"/></>,
    settings: <><circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2"/></>,
    link: <><path d="m8.5 15.5-2 2a3 3 0 0 1-4-4l3.5-3.5a3 3 0 0 1 4 0M15.5 8.5l2-2a3 3 0 0 1 4 4L18 14a3 3 0 0 1-4 0M8 16l8-8"/></>,
    layers: <><path d="m12 3-9 5 9 5 9-5-9-5Z"/><path d="m3 12 9 5 9-5M3 16l9 5 9-5"/></>,
    search: <><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></>,
    alert: <><path d="M12 3 2 21h20L12 3Z"/><path d="M12 9v5M12 18h.01"/></>,
    chevron: <path d="m9 18 6-6-6-6"/>,
    panel: <><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M15 4v16"/></>,
    lock: <><rect x="5" y="10" width="14" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/></>,
    sun: <><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2"/></>,
    moon: <path d="M20 15.5A8 8 0 0 1 8.5 4 8.5 8.5 0 1 0 20 15.5Z"/>,
    more: <><circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/></>,
    locate: <><circle cx="12" cy="12" r="3"/><circle cx="12" cy="12" r="8"/><path d="M12 2V0M12 24v-2M2 12H0M24 12h-2"/></>,
  };
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden>{paths[name] || paths.overview}</svg>;
}

function Mark() {
  return <img className="brand-mark" src="/firescene-mark.png" alt=""/>;
}

function Dot({ tone = 'muted' }: { tone?: 'good' | 'warn' | 'danger' | 'muted' }) {
  return <span className={`dot ${tone}`} />;
}


const descriptions = [
  '', '기체 상태와 비행 제어 기능을 준비하고 있습니다.', '임무 계획 기능을 준비하고 있습니다.',
  '자율비행 기능을 준비하고 있습니다.', '3D 매핑 연동을 준비하고 있습니다.', '카메라 연동을 준비하고 있습니다.',
  '안전 관리 기능을 준비하고 있습니다.', '비행 기록 기능을 준비하고 있습니다.', '연결 및 프로그램 설정을 준비하고 있습니다.',
];

function App() {
  const [page, setPage] = useState(0);
  const [mapSelection,setMapSelection] = useState<MapSelection|null>(null);
  const [selectedMission,setSelectedMission] = useState<SelectedMission|null>(null);
  const selectMapLocation=useCallback((selection:MapSelection|null)=>{setMapSelection(selection);setSelectedMission(m=>missionAfterMapChange(m,selection));},[]);
  const [missionDirty,setMissionDirty] = useState(false);
  const control = useConnection();
  const connection = control.connection;
  const connected = connection.phase === 'CONNECTED' && !!connection.vehicle;
  const status = connected ? 'PX4 HEARTBEAT 수신' : connection.phase === 'CONNECTED' ? '기체 식별 확인 불가' : ['DISCONNECTED','PORT_DETECTED'].includes(connection.phase) ? '기체 미연결' : phaseText[connection.phase] || '확인 불가';
  const [details, setDetails] = useState(true);
  const [navExpanded, setNavExpanded] = useState(true);
  const [context, setContext] = useState<'현장'|'기체'|'안전'|'조종'>('현장');
  const [theme, setThemeState] = useState<Theme>(()=>typeof window === 'undefined' ? 'dark' : initialTheme());
  const [firstRun,setFirstRun] = useState(()=>typeof window !== 'undefined' && needsFirstRun());
  function setTheme(next:Theme) {setThemeState(next);saveTheme(next);}
  function navigate(next:number) { if (page===2 && next!==2 && missionDirty && !window.confirm('저장하지 않은 임무 변경사항이 있습니다. 이동하시겠습니까?')) return; setPage(next); }
  return <div className={`app theme-${theme}`}><div className="product" inert={firstRun || undefined}>
    <header className="header">
      <div className="product-brand"><Mark/><div><b>FireScene <em>GCS</em></b><span>Ground Control Station · {version}</span></div></div>
      <div className="header-context"><span>현재 임무</span><b>{selectedMission?.name || '선택한 임무 없음'}</b></div>
      <div className="header-status"><Dot tone={connected ? 'good' : 'muted'}/><div><b>{status}</b><span>{connection.phase==='UNKNOWN' ? 'UNKNOWN · 상태 조회 실패' : connection.source==='OFFLINE' ? 'OFFLINE · 텔레메트리 미연결' : sourceText(connection)}</span></div></div>
      <button className="theme-toggle" onClick={() => setTheme(theme === 'dark' ? 'day' : 'dark')} aria-label={theme === 'dark' ? 'Day 테마로 변경' : 'Dark 테마로 변경'}><Icon name={theme === 'dark' ? 'sun' : 'moon'} size={17}/><span>{theme === 'dark' ? 'Day' : 'Dark'}</span></button>
    </header>
    <div className="status-strip" aria-label="주요 비행 상태"><div><span>연결</span><b>{status}</b></div><LiveStrip/><SafetyBanner onSelect={()=>navigate(6)}/><RecordingBadge/><div className="strip-lock"><Icon name="lock" size={14}/> 명령 잠김</div></div>
    <div className={`body ${details ? '' : 'panel-closed'} ${page===0 ? 'overview-body' : ''} ${navExpanded ? 'nav-expanded' : 'nav-collapsed'}`}>
      <nav className="nav-rail" aria-label="주 메뉴"><button className="nav-toggle" aria-label={navExpanded ? '메뉴 접기' : '메뉴 펼치기'} aria-expanded={navExpanded} onClick={()=>setNavExpanded(!navExpanded)}><Icon name="panel"/><span>{navExpanded ? '메뉴 접기' : '메뉴 펼치기'}</span></button>{navigation.map(([icon,label],i)=><button key={icon} title={label} aria-label={label} aria-current={page === i ? 'page' : undefined} className={page === i ? 'active' : ''} onClick={()=>navigate(i)}><Icon name={icon}/><span>{label}</span></button>)}<div className="nav-foot">FireScene<br/><span>운용 화면</span></div></nav>
      <main className={`workspace ${page===0 ? 'overview-workspace' : ''}`}>
        {page === 0 ? <div className="overview-shell"><div className="overview-title"><div><span className="eyebrow">MISSION CONTROL / 공간 관제</span><h1>종합 관제</h1></div><span className="source-label">{connected ? '기체 데이터 수신 중' : '데이터 소스 없음'} · {sourceText(connection)}</span></div>
          <MapView source={connection.source} initialSelection={mapSelection} onSelection={selectMapLocation}/>
          <section className="overview-bottom" aria-label="핵심 비행 정보"><OverviewFlightStrip/><div className="overview-control"><span>조종 모드</span><button className="connection-button" onClick={()=>{setDetails(true);setContext('조종');}}>입력 미리보기</button><small>비행 명령 잠김</small></div></section>
        </div> : page === 1 ? <><div className="workspace-heading"><div><span className="eyebrow">기체 운용</span><h1>비행 제어</h1><p>자세 및 비행 상태</p></div><span className="source-label">{connected ? '기체 데이터 수신 중' : '데이터 소스 없음'}</span></div><FlightInstruments/></> : page === 2 ? <Suspense fallback={<PageLoading/>}><MissionPage source={connection.source} sharedSelection={mapSelection} selectedMission={selectedMission} onSelectedMission={setSelectedMission} onSharedSelection={setMapSelection} onDirtyChange={setMissionDirty}/></Suspense> : page === 4 ? <Suspense fallback={<PageLoading/>}><Spatial selection={mapSelection}/></Suspense> : page === 5 ? <Suspense fallback={<PageLoading/>}><Camera/></Suspense> : page === 6 ? <SafetyPage/> : page === 7 ? <Suspense fallback={<PageLoading/>}><Recordings selectedMissionId={selectedMission?.id ?? null}/></Suspense> : page === 8 ? <><ConnectionSettings control={control}/><Companion settings/><MapSettings/></> : page >= 3 && page <= 5 ? <><div className="workspace-heading"><div><span className="eyebrow">작업 공간</span><h1>{navigation[page][1]}</h1><p>{descriptions[page]}</p></div><span className="source-label">개발 예정</span></div><Companion page={navigation[page][1]}/></> : <div className="placeholder"><Icon name={navigation[page][0]} size={28}/><span className="eyebrow">작업 공간</span><h1>{navigation[page][1]}</h1><p>개발 예정</p><small>{descriptions[page]}</small></div>}
        {page===1 && <ManualPreview key={page}/>}
      </main>
      {details && (page===0 ? <aside className="inspector overview-context" aria-label="상황 패널"><div className="context-header"><div><span className="eyebrow">CONTEXT / 상황 인식</span><h2>상황 패널</h2></div><button onClick={()=>setDetails(false)} aria-label="기체 정보 패널 닫기">×</button></div><div className="context-tabs" role="tablist" aria-label="상황 패널 항목">{(['현장','기체','안전','조종'] as const).map(item=><button key={item} role="tab" aria-selected={context===item} onClick={()=>setContext(item)}>{item}</button>)}</div>
        {context==='현장' && <div className="context-content"><span className="context-kicker">INCIDENT / 현장</span><h3>{mapSelection?.name || '선택된 현장 없음'}</h3><p>{mapSelection?.address || '지도에서 건물을 선택하거나 임무 계획에서 저장된 임무를 선택하세요.'}</p>{mapSelection && <dl className="context-facts"><div><dt>조회 범위</dt><dd>{mapSelection.radius} m</dd></div><div><dt>건물 윤곽</dt><dd>{mapSelection.geometry ? '선택됨' : '미선택'}</dd></div></dl>}<div className="context-divider"/><span className="context-kicker">MISSION / 임무</span><h3>{selectedMission?.name || '선택한 임무가 없습니다'}</h3><p>{selectedMission ? `상태 ${selectedMission.status} · 진행 상태 미연동` : '저장된 임무를 선택해 현장과 연결하세요.'}</p><button className="connection-button" onClick={()=>navigate(2)}>임무 계획 열기</button><div className="context-divider"/><span className="context-kicker">최근 상태 메시지</span><LiveEvents/></div>}
        {context==='기체' && <div className="context-content"><span className="context-kicker">VEHICLE / 기체</span><h3>{status}</h3><p>{connected ? 'HEARTBEAT 수신 중 · 송신자 인증되지 않음' : '기체에서 수신된 데이터가 없습니다.'}</p><LiveInspector/><details className="context-graphs"><summary>상세 비행 정보 및 그래프</summary><OverviewTelemetry/></details></div>}
        {context==='안전' && <div className="context-content"><span className="context-kicker">SAFETY / 안전 감시</span><h3>안전 상태</h3><SafetyBanner onSelect={()=>navigate(6)}/><p>안전 감시는 보조 정보이며 PX4 failsafe와 별도입니다.</p><button className="connection-button" onClick={()=>navigate(6)}>안전 관리 열기</button><div className="command-lock"><Icon name="lock" size={16}/><div><b>비행 명령 잠김</b><span>모든 기체 명령을 사용할 수 없습니다</span></div></div></div>}
        {context==='조종' && <div className="context-content control-context"><ManualPreview key="overview"/></div>}
      </aside> : <aside className="inspector" aria-label="기체 정보"><div className="inspector-head"><div><span className="eyebrow">기체 정보</span><h2>기체 상태</h2><span>{connected ? 'PX4 HEARTBEAT 확인' : '연결된 기체 없음'}</span></div><button onClick={()=>setDetails(false)} aria-label="기체 정보 패널 닫기">×</button></div><div className="inspector-section"><div className="section-heading"><h3>연결 상태</h3><span className="state-badge"><Dot tone={connected ? 'good' : 'muted'}/>{status}</span></div><p>{connected ? 'HEARTBEAT 수신 중 · 송신자 인증되지 않음' : '기체에서 수신된 데이터가 없습니다.'}</p></div><div className="inspector-section"><div className="section-heading"><h3>비행 준비 상태</h3><span>확인 불가</span></div><LiveInspector/></div><div className="inspector-section"><div className="section-heading"><h3>안전 경고</h3></div><SafetyBanner onSelect={()=>navigate(6)}/></div><div className="command-lock"><Icon name="lock" size={16}/><div><b>비행 명령 잠김</b><span>모든 기체 명령을 사용할 수 없습니다</span></div></div><p className="development-note">읽기 전용 연결 · 비행 명령 비활성</p></aside>)}

      {!details && <button className="reopen-panel" onClick={()=>setDetails(true)}><Icon name="panel" size={17}/><span>기체 정보</span></button>}
    </div>
  </div>{firstRun && <FirstRun theme={theme} onTheme={setTheme} onClose={()=>setFirstRun(false)}/>}</div>;
}
export default App;
