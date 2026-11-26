#!/usr/bin/env python3
"""Isolated real Tauri smoke QA via W3C WebDriver; no production dependencies."""
import argparse, base64, json, os, pathlib, shutil, socket, subprocess, sys, tempfile, time, urllib.request, threading

ROOT = pathlib.Path(__file__).resolve().parents[2]
PARSER = argparse.ArgumentParser()
PARSER.add_argument('--driver', required=True, help='tauri-driver executable')
PARSER.add_argument('--native-driver', required=True, help='WebKitWebDriver executable')
PARSER.add_argument('--app', default=str(ROOT/'src-tauri/target/debug/firescene-gcs'))
PARSER.add_argument('--results', default='/tmp/firescene-desktop-qa')
PARSER.add_argument('--xdotool', help='optional X11 xdotool to focus the test window')
PARSER.add_argument('--dev', action='store_true', help='start Vite and use debug simulation build')
PARSER.add_argument('--with-map-key', action='store_true', help='copy existing key into isolated private config')
PARSER.add_argument('--matrix', action='store_true', help='capture both themes at four requested window sizes')
PARSER.add_argument('--fixtures', action='store_true', help='use synthetic loopback camera and point cloud sources')
PARSER.add_argument('--serial-only', action='store_true', help='run only the PTY recovery scenario')
PARSER.add_argument('--partial-only', action='store_true', help='run only the partial telemetry interruption scenario')
PARSER.add_argument('--load-only', action='store_true', help='run GUI synthetic rate scenarios')
PARSER.add_argument('--recording-only', action='store_true', help='run connection recording scenarios')
PARSER.add_argument('--no-key-only', action='store_true', help='verify missing map key handling')
PARSER.add_argument('--race-only', action='store_true', help='run controlled async connection ordering')
PARSER.add_argument('--instruments-only', action='store_true', help='compare fixed synthetic instrument values')
PARSER.add_argument('--offline-only', action='store_true', help='run offline input practice scenario')
PARSER.add_argument('--startup-only', action='store_true', help='measure WebDriver session to navigation readiness')
PARSER.add_argument('--version-only', help='verify the displayed application version')
PARSER.add_argument('--preflight-only', action='store_true', help='verify read-only preflight screen')
PARSER.add_argument('--virtual-only', action='store_true', help='verify virtual control integration')
PARSER.add_argument('--journal-endurance-only', action='store_true', help='check repeated virtual event sessions and ordering')
PARSER.add_argument('--journal-endurance-batches', type=int, default=3, help='number of 30-session endurance batches')
PARSER.add_argument('--replay-only', action='store_true', help='verify recording and replay interlock')
PARSER.add_argument('--release-ipc-only', action='store_true', help='verify production virtual IPC is unregistered')
PARSER.add_argument('--journal-stall-only', action='store_true', help='verify watchdog while isolated event FIFO is blocked')
PARSER.add_argument('--journal-stall-stop-only', action='store_true', help='verify stop while isolated event FIFO is blocked')
PARSER.add_argument('--journal-failure-only', action='store_true', help='verify isolated event open failure')
PARSER.add_argument('--journal-write-failure-only', action='store_true', help='verify isolated event write failure via /dev/full')
PARSER.add_argument('--virtual-perf-only', action='store_true', help='sample idle and active virtual session process usage')
PARSER.add_argument('--focus-ipc-only', action='store_true', help='verify direct IPC after actual X11 focus loss')
PARSER.add_argument('--layout-diagnostic-only', action='store_true', help='capture window and overview geometry through resize sequence')
PARSER.add_argument('--other-tauri-window-only', action='store_true', help='verify untrusted Tauri webview IPC ACL')
PARSER.add_argument('--minimized-ipc-only', action='store_true', help='verify IPC during actual X11 minimize')
PARSER.add_argument('--queue-race-only', action='store_true', help='fill isolated journal queue via actual virtual IPC')
PARSER.add_argument('--queue-watchdog-only', action='store_true', help='verify watchdog alone releases after IPC queue saturation')
PARSER.add_argument('--virtual-session-races-only', action='store_true', help='verify fault/stop and stale-stop races through IPC')
PARSER.add_argument('--virtual-recovery-only', action='store_true', help='verify simulated recovery through dev-only QA fixture')
PARSER.add_argument('--write-stop-race-only', action='store_true', help='race event write failure and stop through IPC')
PARSER.add_argument('--serial-recovery', action='store_true', help='enable Linux development PTY discovery fixture')
args=PARSER.parse_args()
if args.serial_only: args.serial_recovery=True
out=pathlib.Path(args.results).resolve(); out.mkdir(parents=True,exist_ok=True)
with socket.socket() as probe:
 probe.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
 probe.bind(('127.0.0.1',4444))  # refuse to attach to an unrelated existing driver
# Refuse any persistent or user-owned data directory as a test profile.
profile=pathlib.Path(tempfile.mkdtemp(prefix='firescene-qa-',dir='/tmp'))
env=os.environ.copy()
if args.other_tauri_window_only:
 if not args.dev: raise ValueError('other window fixture requires dev simulation')
 env['FIRESCENE_QA_UNTRUSTED_WINDOW']='1'
if args.virtual_recovery_only:
 if not args.dev: raise ValueError('recovery fixture requires dev simulation')
 env['FIRESCENE_QA_RECOVERY']='1'
pty_master=None; pty_slave=None; pty_state={'enabled':False,'system':42,'stop':False}
if args.serial_recovery:
 if not args.dev: raise ValueError('--serial-recovery requires --dev and a simulation debug build')
 pty_master,pty_slave=os.openpty(); env['FIRESCENE_QA_PTY']=os.ttyname(pty_slave)
 os.close(pty_slave); pty_slave=None  # do not retain the exclusive slave across sessions
 def pty_feed():
  while not pty_state['stop']:
   if pty_state['enabled']:
    frame=bytearray([0xfe,9,1,pty_state['system'],1,0,0,0,0,0,2,12,0,0,3]); crc=0xffff
    for byte in list(frame[1:])+[50]:
     t=byte^(crc&255);t^=(t<<4)&255;crc=(crc>>8)^(t<<8)^(t<<3)^(t>>4)
    frame.extend([crc&255,crc>>8])
    try: os.write(pty_master,frame)
    except OSError: pass
   time.sleep(.5)
 pty_thread=threading.Thread(target=pty_feed,daemon=True);pty_thread.start()
for k,v in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:
 p=profile/v; p.mkdir(mode=0o700); env[k]=str(p)
if args.with_map_key:
 source=pathlib.Path.home()/'.config/org.firescene.gcs/vworld-api-key'
 dest=profile/'config/org.firescene.gcs/vworld-api-key'; dest.parent.mkdir(mode=0o700,parents=True); shutil.copyfile(source,dest); dest.chmod(0o600)
vite=None; feed=None; fixtures=[]
if args.dev:
 vite=subprocess.Popen(['npm','run','dev','--','--port','1420','--strictPort'],cwd=ROOT/'frontend',env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 time.sleep(2)
proc=subprocess.Popen([args.driver,'--native-driver',args.native_driver],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
sid=None; results=[]
def call(method,path,data=None):
 req=urllib.request.Request('http://127.0.0.1:4444'+path,data=None if data is None else json.dumps(data).encode(),method=method,headers={'Content-Type':'application/json'})
 with urllib.request.urlopen(req,timeout=30) as f: return json.load(f)['value']
def command(method,path,data=None):return call(method,'/session/'+sid+path,data)
def js(script,*arguments): return command('POST','/execute/sync',{'script':script,'args':list(arguments)})
def check(name,fn):
 try:
  detail=fn(); results.append({'name':name,'status':'PASS','detail':detail or ''}); print('PASS',name,flush=True)
 except Exception as e:
  results.append({'name':name,'status':'FAIL','detail':type(e).__name__+': '+str(e)[:250]}); print('FAIL',name,str(e)[:250],flush=True)
  if name == 'serial automatic recovery and manual disconnect':
   try: results[-1]['diagnostic']=js("return document.querySelector('.connection-page')?.innerText")
   except Exception: pass
  if name == 'controlled async connection ordering':
   try: results[-1]['diagnostic']=js("return {internal:Object.getOwnPropertyDescriptor(window,'__TAURI_INTERNALS__'),invoke:Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__,'invoke'),qa:window.__qaRace && {holdPoll:window.__qaRace.holdPoll,commands:window.__qaRace.commands}}")
   except Exception: pass
  if name == 'simulation recording and replay': screenshot('recording-failure')
  if name == 'synthetic camera display':
   try:
    screenshot('camera-failure'); command('POST','/window/rect',{'width':1200,'height':800})
   except Exception: pass
  if name == 'offline practice basic':
   try:
    results[-1]['diagnostic']=js("return {focus:document.hasFocus(),visibility:document.visibilityState,reason:document.querySelector('.manual-preview [role=status]')?.innerText||'',manual:document.querySelector('.manual-preview')?.innerText,buttons:[...document.querySelectorAll('button')].filter(e=>e.innerText.includes('오프라인 입력 연습')).map(e=>({text:e.innerText,disabled:e.disabled,rect:{x:e.getBoundingClientRect().x,y:e.getBoundingClientRect().y}}))}")
   except Exception: pass
  if name in ('overview workspace fits 1200x800','compact live source and safety'):
   try:
    results[-1]['diagnostic']=js("const e=document.querySelector('.overview-workspace'),m=document.querySelector('.overview-shell .map-real');const r=x=>x?{width:x.getBoundingClientRect().width,height:x.getBoundingClientRect().height,client:x.clientHeight,scroll:x.scrollHeight}:null;const bottom=e?.getBoundingClientRect().bottom||0;const overflow=[...(e?.querySelectorAll('*')||[])].map(x=>({tag:x.tagName,cls:typeof x.className==='string'?x.className:'',bottom:x.getBoundingClientRect().bottom,height:x.getBoundingClientRect().height})).filter(x=>x.bottom>bottom+2).sort((a,b)=>b.bottom-a.bottom).slice(0,12);return {innerWidth,innerHeight,workspace:r(e),map:r(m),overflow,bodyClass:document.querySelector('.body')?.className,focus:document.hasFocus(),fullscreen:!!document.fullscreenElement}")
    results[-1]['window_rect']=command('GET','/window/rect')
   except Exception: pass
def click_text(label):
 ok=js("const e=[...document.querySelectorAll('button')].find(x=>x.textContent.trim()===arguments[0]); if(!e||e.disabled)return false; e.click();return true;",label)
 if not ok: raise AssertionError('button unavailable: '+label)
def click_user(label):
 found=command('POST','/element',{'using':'xpath','value':"//button[normalize-space(.)="+json.dumps(label,ensure_ascii=False)+"]"})
 command('POST','/element/'+found['element-6066-11e4-a52e-4f735466cecf']+'/click',{})

def wait_for(script,seconds=10):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  if js(script): return
  time.sleep(.2)
 raise AssertionError('condition timed out: '+script[:160])
def screenshot(name):
 # App-only WebDriver captures; settings are captured only through screenshot_settings redaction.
 (out/(name+'.png')).write_bytes(base64.b64decode(command('GET','/screenshot')))
def screenshot_settings(name):
 # Isolated profile only: hide all editable values and selects, including the map key and local paths.
 js("const s=document.createElement('style');s.id='qa-redaction';s.textContent='input,textarea,select { visibility:hidden!important }';document.head.appendChild(s)")
 try:
  assert js("return [...document.querySelectorAll('input,textarea,select')].every(e=>getComputedStyle(e).visibility==='hidden')")
  command('POST','/execute/async',{'script':'const done=arguments[arguments.length-1];requestAnimationFrame(()=>requestAnimationFrame(()=>done(true)))','args':[]})
  screenshot(name)
 finally: js("document.querySelector('#qa-redaction')?.remove()")
def find_main_pid():
 def descend(pid):
  try: children=[int(x) for x in pathlib.Path(f'/proc/{pid}/task/{pid}/children').read_text().split()]
  except (FileNotFoundError,PermissionError): return []
  return children+[c for child in children for c in descend(child)]
 pids=descend(proc.pid)
 app=next((p for p in pids if pathlib.Path(f'/proc/{p}/comm').read_text().strip()=='firescene-gcs'),None)
 if not app: raise RuntimeError('app process not found under test driver')
 return app
def measure_main():
 app=find_main_pid()
 def tree(pid):
  try: children=[int(x) for x in pathlib.Path(f'/proc/{pid}/task/{pid}/children').read_text().split()]
  except (FileNotFoundError,PermissionError): return []
  return children+[desc for child in children for desc in tree(child)]
 def read(pid):
  try:
   stat=pathlib.Path(f'/proc/{pid}/stat').read_text().split()
   status=pathlib.Path(f'/proc/{pid}/status').read_text().splitlines()
   rss=int(next(x.split()[1] for x in status if x.startswith('VmRSS:')))
   try:
    rollup=pathlib.Path(f'/proc/{pid}/smaps_rollup').read_text().splitlines()
    pss=int(next(x.split()[1] for x in rollup if x.startswith('Pss:')))
   except (FileNotFoundError,PermissionError,StopIteration,ProcessLookupError): pss=None
   return {'ticks':int(stat[13])+int(stat[14]),'rss':rss,'pss':pss,'name':pathlib.Path(f'/proc/{pid}/comm').read_text().strip()}
  except (FileNotFoundError,PermissionError,StopIteration,ProcessLookupError): return None
 samples=[]
 for _ in range(3):
  before={pid:v for pid in [app]+tree(app) if (v:=read(pid))}; start=time.monotonic(); time.sleep(2); elapsed=time.monotonic()-start
  after={pid:v for pid in [app]+tree(app) if (v:=read(pid))}
  groups={'main':{},'webkit':{},'other':{}}
  for pid,v in after.items():
   group='main' if pid==app else 'webkit' if 'WebKit' in v['name'] else 'other'
   g=groups[group];g['count']=g.get('count',0)+1;g['rss_mib']=g.get('rss_mib',0)+v['rss']/1024
   if v['pss'] is not None: g['pss_mib']=g.get('pss_mib',0)+v['pss']/1024
   if pid in before: g['cpu_percent']=g.get('cpu_percent',0)+(v['ticks']-before[pid]['ticks'])/os.sysconf('SC_CLK_TCK')/elapsed*100
  samples.append(groups)
 return {'interval_seconds':2,'samples':samples,'rss_note':'RSS sums may double count shared pages; CPU excludes processes that start after interval begins'}
def menu(name,index):
 ok=js("const b=[...document.querySelectorAll('nav[aria-label=\"주 메뉴\"] button')].find(x=>x.getAttribute('aria-label')===arguments[0]);if(!b)return false;b.click();return true;",name)
 if not ok: raise AssertionError('menu missing')
 wait_for("return document.querySelector('nav [aria-current=page]')?.getAttribute('aria-label')===arguments[0]".replace('arguments[0]',json.dumps(name,ensure_ascii=False)))
 time.sleep(.5)
 if name!='시스템 설정': screenshot(f'{index:02d}')
 return 'navigation active; screenshot saved' if name!='시스템 설정' else 'navigation active; screenshot skipped for key privacy'
def serial_recovery():
 global feed
 if feed: feed.terminate(); feed.wait(timeout=5); feed=None
 menu('시스템 설정',8)
 wait_for("return [...document.querySelectorAll('#settings-aircraft select option')].some(x=>x.textContent.includes('Synthetic PTY'))",15)
 js("const s=document.querySelector('#settings-aircraft select');s.value=[...s.options].find(x=>x.textContent.includes('Synthetic PTY')).value;s.dispatchEvent(new Event('change',{bubbles:true}))")
 pty_state['enabled']=True
 click_text('선택 장치 자동 연결 허용')
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",25)
 pty_state['enabled']=False
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('연결 끊김')",15)
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('HEARTBEAT 대기')",20)
 pty_state['system']=43; pty_state['enabled']=True
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('43 / 1')",15)
 assert js("return document.querySelector('.connection-page')?.innerText.includes('직전 실패')")
 screenshot_settings('serial-auto-recovered-redacted')
 # A successful session resets the budget: after a later loss, three fresh
 # attempts must be made before retrying stops.
 pty_state['enabled']=False
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('재시도 3회 종료')",90)
 assert js("return document.querySelector('.connection-page')?.innerText.includes('직전 실패')")
 pty_state['enabled']=True
 time.sleep(6)
 assert js("return !document.querySelector('.connection-page')?.innerText.includes('43 / 1')")
 click_text('선택 장치 자동 연결 허용')
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('43 / 1')",25)
 click_text('연결 해제')
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('사용자 해제 · 자동 연결 꺼짐')",10)
 time.sleep(8)
 assert js("return !document.querySelector('.connection-page')?.innerText.includes('43 / 1') && document.querySelector('.connection-page')?.innerText.includes('자동 연결: 꺼짐')")
 return 'approved PTY auto connection, new session after loss, three failed retries, explicit re-enable and manual disconnect with device still present'

def partial_telemetry():
 global feed
 control=profile/'sensor-filter'; control.write_text('')
 feed_env=env.copy();feed_env['FIRESCENE_QA_FILTER']=str(control)
 menu('시스템 설정',8);wait_for("return [...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='시뮬레이션 수신 시작'&&!e.disabled)",15);click_text('시뮬레이션 수신 시작')
 feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'normal'],env=feed_env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 def snapshot():
  return command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('connection_status').then(done).catch(e=>done({error:String(e)}))",'args':[]})
 def wait_state(test, timeout=12):
  end=time.monotonic()+timeout
  while time.monotonic()<end:
   state=snapshot()
   if test(state): return state
   time.sleep(.3)
  raise AssertionError('telemetry state timed out: '+str(state)[:500])
 cases=[('ATTITUDE','roll','battery'),('GPS_RAW_INT','gps_lat','roll'),('GLOBAL_POSITION_INT','relative_alt','battery'),('SYS_STATUS','battery','roll'),('VFR_HUD','air_speed','roll')]
 wait_state(lambda s:s.get('phase')=='CONNECTED' and all(s.get('telemetry',{}).get('fields',{}).get(k,{}).get('status')=='LIVE' for k in ['roll','gps_lat','relative_alt','battery']))
 for msg,stale,other in cases:
  control.write_text(msg)
  state=wait_state(lambda s:s.get('phase')=='CONNECTED' and s.get('telemetry',{}).get('fields',{}).get(stale,{}).get('status')=='STALE' and s.get('telemetry',{}).get('fields',{}).get(other,{}).get('status')=='LIVE')
  if msg=='GPS_RAW_INT':
   menu('종합 관제',0);wait_for("return document.body.innerText.includes('GPS 원시 위치 미수신 또는 오래됨')",10)
  else:
   menu('비행 제어',1)
   if msg=='ATTITUDE': wait_for("return !!document.querySelector('.attitude-empty')",10)
   else: wait_for("return [...document.querySelectorAll('.instrument-list > div')].some(e=>e.innerText.includes('오래됨'))",10)
  if msg in ('ATTITUDE','GPS_RAW_INT','SYS_STATUS'):
   code={'ATTITUDE':'attitude_stale','GPS_RAW_INT':'gps_data','SYS_STATUS':'battery_data'}[msg]
   wait_state(lambda s:any(e.get('code')==code and e.get('resolved_ms') is None for e in s.get('safety',{}).get('events',[])))
   if msg=='ATTITUDE': wait_for("return document.querySelector('.safety-banner')?.getAttribute('aria-label')?.includes('자세 데이터 수신 지연')",10)
  screenshot('partial-'+msg.lower())
  control.write_text('')
  wait_state(lambda s:s.get('phase')=='CONNECTED' and s.get('telemetry',{}).get('fields',{}).get(stale,{}).get('status')=='LIVE')
 return 'five independently interrupted message classes age and recover while heartbeat and another field stay live'

def gui_load(rate):
 global feed
 menu('시스템 설정',8);click_text('시뮬레이션 수신 시작')
 feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'rate',str(rate),'30'],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 start=time.monotonic()
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
 deadline=time.monotonic()+8
 while True:
  first=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('connection_status').then(done)",'args':[]})
  if first.get('telemetry',{}).get('fields',{}).get('gps_lat',{}).get('status')=='LIVE': break
  if time.monotonic()>deadline: raise AssertionError('initial GPS missing')
  time.sleep(.2)
 menu('비행 제어',1);wait_for("return !!document.querySelector('.attitude .horizon[role=img]')",15)
 menu('종합 관제',0);wait_for("return document.querySelector('.overview-flight-values')?.innerText.includes('m')",10)
 wait_for("return [...document.querySelectorAll('.map-toolbar button')].some(e=>e.textContent.trim()==='기체 위치'&&!e.disabled)",10)
 menu('비행 기록',7)
 existing=set(profile.rglob('*.jsonl'))
 e=command('POST','/element',{'using':'css selector','value':'[aria-label="기록 이름"]'});eid=e['element-6066-11e4-a52e-4f735466cecf'];command('POST','/element/'+eid+'/value',{'text':f'QA load {rate}'})
 click_text('기록 시작');wait_for("return document.body.innerText.includes('기록 중 ·')",10)
 samples=measure_main()
 before=time.monotonic();menu('안전 관리',6);menu('비행 제어',1);latency=time.monotonic()-before
 wait_for("return !!document.querySelector('.attitude .horizon[role=img]')",10)
 # Check reception while the sender is still active; allow the last frames to drain.
 while time.monotonic()-start<29: time.sleep(.2)
 stdout,stderr=feed.communicate(timeout=8);feed=None
 sent=rate*30;deadline=time.monotonic()+2
 while True:
  state=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('connection_status').then(done)",'args':[]})
  received=state['diagnostics']['messages']
  if received>=sent or time.monotonic()>=deadline: break
  time.sleep(.05)
 assert f'sent={sent} rate={rate} duration=30' in stdout, (stdout,stderr)
 assert state['phase']=='CONNECTED' and received==sent, (state['phase'],received,sent)
 fields=state['telemetry']['fields']
 assert all(fields.get(k,{}).get('status')=='LIVE' for k in ['roll','gps_lat','gps_lon','battery','relative_alt']), fields
 assert fields['gps_lat']['value'] > first['telemetry']['fields']['gps_lat']['value']
 assert state.get('safety',{}).get('connection') not in (None,'UNKNOWN')
 menu('비행 기록',7);click_text('기록 종료');wait_for("return document.querySelector('.recordings-page .source-label')?.innerText.includes('기록 중 아님')",10)
 records=set(profile.rglob('*.jsonl'))-existing; assert len(records)==1 and '"type":"sample"' in next(iter(records)).read_text()
 menu('시스템 설정',8);wait_for("return document.querySelector('.connection-page')?.innerText.includes('연결 끊김')",15)
 return {'rate':rate,'duration_s':30,'sent':sent,'received':received,'process_samples':samples,'two_menu_switches_s':latency,'gpu':'not measured'}

def reset_load():
 global feed
 if feed:
  if feed.poll() is None: feed.terminate()
  feed.communicate(timeout=5);feed=None
 command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];Promise.allSettled([window.__TAURI_INTERNALS__.invoke('stop_recording'),window.__TAURI_INTERNALS__.invoke('disconnect_serial')]).then(()=>done(true))",'args':[]})
 time.sleep(2.5)

def recording_connections():
 global feed
 def invoke(name):
  return command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke(arguments[0]).then(done).catch(e=>done({error:String(e)}))",'args':[name]})
 def begin(name):
  menu('비행 기록',7);e=command('POST','/element',{'using':'css selector','value':'[aria-label="기록 이름"]'});eid=e['element-6066-11e4-a52e-4f735466cecf'];command('POST','/element/'+eid+'/clear',{});command('POST','/element/'+eid+'/value',{'text':name});click_text('기록 시작');wait_for("return document.body.innerText.includes('기록 중 ·')",10);time.sleep(2)
 menu('시스템 설정',8);click_text('시뮬레이션 수신 시작')
 feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'battery-low'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
 begin('QA connection loss'); record_id=invoke('recording_status')['id']
 wait_for("return document.body.innerText.includes('배터리 부족')",15);time.sleep(2)
 feed.terminate();feed.wait(timeout=5);feed=None
 menu('시스템 설정',8);wait_for("return document.querySelector('.connection-page')?.innerText.includes('연결 끊김')",15);time.sleep(2)
 click_text('시뮬레이션 수신 시작');feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'normal'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
 deadline=time.monotonic()+10
 while time.monotonic()<deadline:
  status=invoke('recording_status')
  if not status['active']: break
  time.sleep(.3)
 assert not status['active'] and '새 연결 세션' in status['error'],status
 paths=list(profile.rglob(record_id+'.jsonl'));assert len(paths)==1
 lines=[json.loads(line) for line in paths[0].read_text().splitlines()]
 phases=[v.get('phase') for v in lines if v.get('type')=='connection']
 assert 'CONNECTED' in phases and 'CONNECTION_LOST' in phases, phases
 assert any(v.get('type')=='event' and v.get('event',{}).get('code')=='battery_low' for v in lines)
 session=lines[0]['connection_session'];assert all(v['snapshot']['safety']['session'] in (0,session) for v in lines if v['type']=='sample')
 assert not any(v['type']=='end' for v in lines)
 shutil.copyfile(paths[0],out/'synthetic-loss-session.jsonl')
 begin('QA manual disconnect'); second=invoke('recording_status')['id']
 menu('시스템 설정',8);click_text('연결 해제');time.sleep(2)
 menu('비행 기록',7);click_text('기록 종료');wait_for("return document.querySelector('.recordings-page .source-label')?.innerText.includes('기록 중 아님')",10)
 paths=list(profile.rglob(second+'.jsonl'));assert len(paths)==1
 lines2=[json.loads(line) for line in paths[0].read_text().splitlines()]
 assert any(v.get('type')=='connection' and v.get('phase')=='DISCONNECTED' for v in lines2)
 assert any(v.get('type')=='end' for v in lines2)
 shutil.copyfile(paths[0],out/'synthetic-manual-disconnect.jsonl')
 return {'loss_phases':phases,'battery_warning_recorded':True,'new_session_stops_previous_recording':True,'manual_disconnect_recorded':True,'format_version':lines[0]['version']}

def missing_map_key():
 if args.with_map_key: raise ValueError('no-key test requires isolated profile without a key')
 menu('종합 관제',0)
 wait_for("return document.querySelector('.map-empty')?.innerText.includes('API 키가 설정되지 않았습니다')")
 e=command('POST','/element',{'using':'css selector','value':'[aria-label="위치 검색"]'});eid=e['element-6066-11e4-a52e-4f735466cecf'];command('POST','/element/'+eid+'/value',{'text':'서울'});click_text('검색')
 wait_for("return document.querySelector('.map-search [role=status]')?.innerText.includes('지도 API 키가 필요합니다')")
 screenshot('map-no-key')
 return 'missing-key guidance shown and search rejected without asking for normal map results'

def connection_race():
 global feed
 menu('시스템 설정',8)
 # Intercept the actual Tauri invocation boundary, delaying one completion.
 js("""const original=window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);window.__qaRace={original,commands:[],holdPoll:true};
 window.__firesceneQaInvoke=(cmd,args)=>{const q=window.__qaRace;q.commands.push(cmd);
  if(cmd==='connection_summary'&&q.holdPoll){q.holdPoll=false;return original(cmd,args).then(v=>new Promise(resolve=>{q.releasePoll=()=>resolve(v)}));}
  if(cmd==='disconnect_serial'&&q.holdDisconnect){q.holdDisconnect=false;return original(cmd,args).then(()=>new Promise((resolve,reject)=>{q.releaseDisconnect=()=>reject('QA delayed old disconnect error')}));}
  if(cmd==='connect_simulation'&&q.holdConnect){q.holdConnect=false;return original(cmd,args).then(v=>new Promise(resolve=>{q.releaseConnect=()=>resolve(v)}));}
  return original(cmd,args)}""")
 wait_for("return !!window.__qaRace.releasePoll",8)
 click_text('시뮬레이션 수신 시작')
 feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'normal'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 wait_for("return window.__qaRace.commands.includes('connect_simulation')",5)
 js("window.__qaRace.releasePoll()")
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
 js("window.__qaRace.holdDisconnect=true")
 click_text('연결 해제')
 wait_for("return !!window.__qaRace.releaseDisconnect",8)
 # Directly dispatch the installed React handler to exercise overlapping intents
 # even while the UI's loading guard prevents a normal second click.
 assert js("const b=[...document.querySelectorAll('button')].find(e=>e.textContent.trim()==='시뮬레이션 수신 시작');const k=Object.keys(b).find(x=>x.startsWith('__reactProps$'));if(!k)return false;b[k].onClick();return true")
 js("window.__qaRace.releaseDisconnect()")
 wait_for("return window.__qaRace.commands.filter(x=>x==='connect_simulation').length===2",10)
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
 time.sleep(2)
 assert not js("return document.querySelector('.connection-page')?.innerText.includes('QA delayed old disconnect error')")
 assert js("return window.__qaRace.commands.filter(x=>x==='connect_simulation').length===2 && window.__qaRace.commands.filter(x=>x==='disconnect_serial').length===1")
 click_text('연결 해제')
 wait_for("return !document.querySelector('.connection-page')?.innerText.includes('42 / 1')",10)
 js("window.__qaRace.holdConnect=true")
 click_text('시뮬레이션 수신 시작')
 wait_for("return !!window.__qaRace.releaseConnect",8)
 assert js("const b=[...document.querySelectorAll('button')].find(e=>e.textContent.trim()==='연결 해제');const k=Object.keys(b).find(x=>x.startsWith('__reactProps$'));if(!k)return false;b[k].onClick();return true")
 js("window.__qaRace.releaseConnect()")
 wait_for("return window.__qaRace.commands.filter(x=>x==='disconnect_serial').length===3",10)
 time.sleep(3)
 assert js("return !document.querySelector('.connection-page')?.innerText.includes('42 / 1')")
 click_text('시뮬레이션 수신 시작')
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
 js("delete window.__firesceneQaInvoke")
 return 'held old status across connect; delayed disconnect failure across reconnect; disconnect during connect; subsequent reconnect; actual Tauri invocations counted'

def instrument_values():
 global feed
 menu('비행 제어',1)
 wait_for("return !!document.querySelector('.attitude-empty') && document.querySelector('.flight-metrics')?.innerText.includes('미연결')")
 menu('시스템 설정',8);wait_for("return [...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='시뮬레이션 수신 시작'&&!e.disabled)",15);click_text('시뮬레이션 수신 시작')
 feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'instruments'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
 menu('비행 제어',1)
 wait_for("return document.querySelector('.attitude .horizon')?.getAttribute('aria-label') === '인공수평계 Roll 10.0도 Pitch -5.0도'",15)
 def cells(selector): return js("return Object.fromEntries([...document.querySelectorAll(arguments[0])].map(e=>[e.querySelector('span')?.innerText.trim(),(e.querySelector('strong,b')?.innerText||'').trim()]))",selector)
 wait_for("return document.querySelector('.instrument-list')?.innerText.includes('123.0')")
 metrics=cells('.flight-metrics .metric'); data=cells('.instrument-list > div')
 expected={'방향 · 북쪽 기준':'0.0 °','상대 고도':'12.0 m','해발 고도':'123.0 m','지상 속도':'3.5 m/s','대기 속도':'4.0 m/s','상승 속도 (+상승)':'0.8 m/s','배터리':'85.0 %','전압':'15.7 V','전류':'2.5 A'}
 assert all(metrics.get(k)==v for k,v in {'Roll':'10.0','Pitch':'-5.0','Yaw':'30.0','Heading':'0.0'}.items()),metrics
 assert js("const t=document.querySelector('.horizon g[transform]')?.getAttribute('transform');const m=t?.match(/rotate\\(([^)]+)\\) translate\\(0 ([^)]+)\\)/);return !!m && Math.abs(Number(m[1])+10)<0.01 && Math.abs(Number(m[2])+10)<0.01")
 assert all(data.get(k)==v for k,v in expected.items()),(data,expected)
 strip=js("return document.querySelector('.status-strip')?.innerText")
 assert all(v in strip for v in ['Position','DISARMED','Fix 3','85%']),strip
 menu('종합 관제',0)
 click_text('기체');js("document.querySelector('.context-graphs summary').click()")
 wait_for("return document.body.innerText.includes('GPS 위성')")
 assert js("return [...document.querySelectorAll('.metric')].some(e=>e.innerText.includes('GPS 위성') && e.innerText.includes('12'))")
 return {'attitude':metrics,'flight_data':data,'gps_fix':3,'satellites':12,'armed':'DISARMED','mode':'Position'}

def keyboard():
 menu('종합 관제',0); click_text('입력 미리보기')
 if args.xdotool:
  wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines()
  if len(wins)!=1: raise RuntimeError('expected exactly one test window')
  subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
 click_user('오프라인 입력 연습 ON'); wait_for("return document.body.innerText.includes('오프라인 입력 연습 · 기체 상태')")
 for key,code in [('w','KeyW'),('s','KeyS'),('a','KeyA'),('d','KeyD'),('ArrowUp','ArrowUp'),('ArrowDown','ArrowDown'),('ArrowLeft','ArrowLeft'),('ArrowRight','ArrowRight')]:
  js("window.dispatchEvent(new KeyboardEvent('keydown',{key:arguments[0],code:arguments[1],bubbles:true}));",key,code)
  wait_for("return document.querySelectorAll('.manual-keys .active').length===1")
  js("window.dispatchEvent(new KeyboardEvent('keyup',{key:arguments[0],code:arguments[1],bubbles:true}));",key,code)
  wait_for("return !document.querySelector('.manual-keys .active')")
 js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW'}));window.dispatchEvent(new KeyboardEvent('keydown',{key:'s',code:'KeyS'}));")
 wait_for("return document.body.innerText.includes('전후 0')")
 js("window.dispatchEvent(new KeyboardEvent('keydown',{key:' ',code:'Space'}));")
 wait_for("return !document.querySelector('.manual-keys .active')")
 js("window.dispatchEvent(new KeyboardEvent('keyup',{key:'w',code:'KeyW'}));window.dispatchEvent(new KeyboardEvent('keyup',{key:'s',code:'KeyS'}));window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',code:'Escape'}));")
 wait_for("return !document.body.innerText.includes('오프라인 입력 연습 · 기체 상태')")
 click_user('오프라인 입력 연습 ON'); wait_for("return document.body.innerText.includes('오프라인 입력 연습 · 기체 상태')")
 js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW'}))")
 wait_for("return document.body.innerText.includes('입력 시간 초과')",5)
 click_user('오프라인 입력 연습 ON'); wait_for("return document.body.innerText.includes('오프라인 입력 연습 · 기체 상태')")
 js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW'}))")
 wait_for("return !!document.querySelector('.manual-keys .active')")
 click_text('전체 화면'); wait_for("return document.querySelector('.manual-preview h2')?.innerText==='MONITOR'")
 wait_for("return [...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='전체 화면 종료')")
 click_text('전체 화면 종료'); time.sleep(1); click_user('오프라인 입력 연습 ON'); wait_for("return document.body.innerText.includes('오프라인 입력 연습 · 기체 상태')")
 element=command('POST','/element',{'using':'css selector','value':'[aria-label="위치 검색"]'})
 command('POST','/element/'+element['element-6066-11e4-a52e-4f735466cecf']+'/click',{})
 wait_for("return document.querySelector('.manual-preview h2')?.innerText==='MONITOR'")
 click_user('오프라인 입력 연습 ON'); wait_for("return document.body.innerText.includes('오프라인 입력 연습 · 기체 상태')")
 click_text('현장'); click_text('입력 미리보기'); wait_for("return document.querySelector('.manual-preview h2')?.innerText==='MONITOR'")
 return 'WebDriver click and synthetic DOM events: bindings, opposing input, Space, Escape, timeout, input focus, fullscreen and panel switch reset'


try:
 time.sleep(.8)
 if proc.poll() is not None: raise RuntimeError('tauri-driver failed to start')
 start=time.monotonic()
 response=call('POST','/session',{'capabilities':{'alwaysMatch':{'browserName':'wry','tauri:options':{'application':str(pathlib.Path(args.app).resolve())}}}})
 sid=response['sessionId']
 if args.other_tauri_window_only:
  def other_window():
   handles=command('GET','/window/handles');assert len(handles)==2,handles
   found={}
   for handle in handles:
    command('POST','/window',{'handle':handle});time.sleep(.5)
    label=js("return window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label || null")
    found[label]=handle
   assert 'main' in found and 'qa-untrusted' in found,found
   command('POST','/window',{'handle':found['main']})
   assert args.xdotool
   wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--name','^FireScene GCS$'],env=env,text=True).splitlines();assert len(wins)==1
   subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   positive=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('connection_status').then(()=>done(true)).catch(()=>done(false))",'args':[]});assert positive
   started=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];const i=window.__TAURI_INTERNALS__.invoke;i('virtual_start').then(v=>i('virtual_stop',{ticket:v.ticket}).then(()=>done(true))).catch(e=>done(String(e)))",'args':[]});assert started is True,started
   command('POST','/window',{'handle':found['qa-untrusted']})
   calls=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];Promise.all(['virtual_start','virtual_input','virtual_fault','virtual_lock','virtual_stop','virtual_replay_start','virtual_replay_stop'].map(name=>window.__TAURI_INTERNALS__.invoke(name,{ticket:1,sequence:1,axes:{forward:1,right:0,up:0,yaw:0},fault:'battery_low'}).then(()=>({name,ok:true})).catch(e=>({name,ok:false,error:String(e)})))).then(done)",'args':[]})
   assert len(calls)==7 and all(not c['ok'] for c in calls),calls
   return {'main_connection_status':positive,'untrusted_calls':calls}
  check('untrusted Tauri window virtual IPC',other_window)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.release_ipc_only: wait_for("return !!window.__TAURI_INTERNALS__?.invoke",20)
 else: wait_for("return !!document.querySelector('nav[aria-label=\"주 메뉴\"]')",20)
 if args.startup_only:
  elapsed=time.monotonic()-start
  def startup_timing():
   web=js("""const n=performance.getEntriesByType('navigation')[0],t=performance.timing;return {now:performance.now(),ready:document.readyState,launchPresent:!!document.getElementById('launch-screen'),navigation:n?{domInteractive:n.domInteractive,domContentLoaded:n.domContentLoadedEventEnd,load:n.loadEventEnd}:null,legacy:t?{domInteractive:t.domInteractive-t.navigationStart,domContentLoaded:t.domContentLoadedEventEnd-t.navigationStart,load:t.loadEventEnd-t.navigationStart}:null,resources:performance.getEntriesByType('resource').map(e=>({name:new URL(e.name).pathname,start:e.startTime,duration:e.duration}))}""")
   pid=find_main_pid(); stat=pathlib.Path(f'/proc/{pid}/stat').read_text().split(); proc_started=int(stat[21])/os.sysconf('SC_CLK_TCK')
   return {'driver_seconds':round(elapsed,3),'process_start_after_driver_seconds':round(proc_started-start,3),'scope':'WebDriver session request to navigation DOM; includes driver and webview startup; process time from Linux /proc ticks','web':web}
  check('startup navigation readiness',startup_timing)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.version_only:
  def app_version():
   wait_for("return !!document.querySelector('.first-run[role=dialog]')")
   click_text('English');click_text('Skip setup')
   wait_for("return !document.querySelector('.first-run[role=dialog]')")
   label=js("return document.querySelector('.product-brand')?.innerText || ''")
   assert args.version_only in label,label
   menu('시스템 설정',8)
   return {'displayed_version':args.version_only,'settings_visible':True}
  check('application version and settings',app_version)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.release_ipc_only:
  def release_ipc():
   outcome=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];Promise.all(['virtual_start','virtual_input','virtual_fault','virtual_status','virtual_stop','virtual_lock','virtual_replay_start','virtual_replay_stop'].map(name=>window.__TAURI_INTERNALS__.invoke(name,{ticket:1,sequence:1,axes:{forward:1,right:0,up:0,yaw:0},fault:'battery_low'}).then(()=>({name,ok:true})).catch(e=>({name,ok:false,error:String(e)})))).then(done)",'args':[]})
   assert len(outcome)==8 and all(not item['ok'] and 'not found' in item['error'] for item in outcome),outcome
   status=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('connection_status').then(()=>done({ok:true})).catch(e=>done({ok:false,error:String(e)}))",'args':[]});assert status['ok'],{'calls':outcome,'status':status}
   return json.dumps(outcome,ensure_ascii=False)
  check('release virtual IPC rejected and read-only status available',release_ipc)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.serial_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English'); click_text('Skip setup'); wait_for("return !document.querySelector('.first-run[role=dialog]')")
  check('serial automatic recovery and manual disconnect',serial_recovery)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.partial_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  check('partial telemetry interruption and recovery',partial_telemetry)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.load_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  for rate in [10,30,60,100]:
   check(f'GUI synthetic load {rate} msg/s',lambda rate=rate:gui_load(rate))
   reset_load()
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.recording_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  check('connection recording and session separation',recording_connections)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.no_key_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  check('missing map key handling',missing_map_key)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.race_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  check('controlled async connection ordering',connection_race)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.offline_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  check('launch overlay removed after interface mount',lambda: (_ for _ in ()).throw(AssertionError('launch overlay still present')) if js("return !!document.getElementById('launch-screen')") else 'interface mounted; launch overlay removed')
  screenshot('first-run')
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  check('offline practice basic',keyboard)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.preflight_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def preflight_check():
   menu('안전 관리',6)
   wait_for("return !!document.querySelector('[aria-label=\"비행 전 점검\"]')")
   assert js("const e=document.querySelector('[aria-label=\"비행 전 점검\"]');return e.innerText.includes('미확인') && e.innerText.includes('PX4 arming 검사나 비행 허가가 아니며') && e.querySelectorAll('input[type=checkbox]').length===3 && [...e.querySelectorAll('input')].every(i=>!i.checked)")
   e=command('POST','/element',{'using':'css selector','value':'[aria-label="비행 전 점검"] input[type=checkbox]'});command('POST','/element/'+e['element-6066-11e4-a52e-4f735466cecf']+'/click',{})
   assert js("return document.querySelector('[aria-label=\"비행 전 점검\"] input').checked && document.querySelector('.strip-lock')?.innerText.includes('잠김')")
   menu('비행 제어',1);menu('안전 관리',6)
   wait_for("return !!document.querySelector('[aria-label=\"비행 전 점검\"] input')")
   assert js("return !document.querySelector('[aria-label=\"비행 전 점검\"] input').checked")
   return 'offline observations unknown; manual checks reset on leaving page; command lock remains visible'
  check('read-only preflight checklist',preflight_check)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.layout_diagnostic_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def layout_diagnostic():
   measurements=[]
   def sample(label):
    measurements.append({'stage':label,'webdriver':command('GET','/window/rect'),'layout':js("const e=document.querySelector('.overview-workspace'),m=document.querySelector('.overview-shell .map-real');const rect=x=>x?{width:x.getBoundingClientRect().width,height:x.getBoundingClientRect().height,client:x.clientHeight,scroll:x.scrollHeight}:null;return {innerWidth,innerHeight,workspace:rect(e),map:rect(m),focus:document.hasFocus()}")})
   menu('종합 관제',0);command('POST','/window/rect',{'width':2560,'height':1440});time.sleep(.3);sample('matrix end')
   command('POST','/window/rect',{'width':1100,'height':760});time.sleep(.3);sample('resize before maximize')
   command('POST','/window/maximize',{});time.sleep(.3);sample('after maximize')
   command('POST','/window/rect',{'width':1200,'height':800});time.sleep(.3);sample('requested 1200x800')
   (out/'layout-geometry.json').write_text(json.dumps(measurements,indent=2))
   assert measurements[-1]['layout']['workspace']['scroll']<=measurements[-1]['layout']['workspace']['client']+1,measurements[-1]
   return measurements
  check('overview workspace fits 1200x800 diagnostic',layout_diagnostic)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.write_stop_race_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def write_stop_race():
   assert args.xdotool
   wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines();assert len(wins)==1
   subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   state=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(done).catch(e=>done({error:String(e)}))",'args':[]});assert state.get('ticket'),state
   ticket=state['ticket'];active=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_input',{ticket:arguments[0],sequence:1,axes:{forward:1,right:0,up:0,yaw:0}}).then(done)",'args':[ticket]});assert active['velocity']['forward']==1,active
   journals=list(profile.rglob('virtual-control-events.jsonl'));assert len(journals)==1
   path=journals[0];path.rename(path.with_suffix('.saved'));path.symlink_to('/dev/full')
   race=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1],i=window.__TAURI_INTERNALS__.invoke,ticket=arguments[0];Promise.allSettled([i('virtual_input',{ticket,sequence:2,axes:{forward:0,right:0,up:0,yaw:0}}),i('virtual_stop',{ticket})]).then(v=>done(v.map(x=>x.status)))",'args':[ticket]})
   time.sleep(.25)
   probe=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1],i=window.__TAURI_INTERNALS__.invoke;Promise.all([i('virtual_input',{ticket:arguments[0],sequence:9999,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>false).catch(()=>true),i('virtual_replay_start').then(t=>i('virtual_replay_stop',{ticket:t}).then(()=>true)).catch(()=>false)]).then(done)",'args':[ticket]});assert probe==[True,True],probe
   denied=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(()=>done('unexpected success')).catch(e=>done(String(e)))",'args':[]});assert '이벤트' in denied,denied
   view=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert view['ticket'] is None and view['authority']=='OFFLINE' and all(v==0 for v in view['velocity'].values()),view
   return {'race':race,'old_token_rejected':True,'new_start_rejected':True,'replay_available':True}
  check('write failure and stop IPC race',write_stop_race)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.virtual_recovery_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def virtual_recovery():
   assert args.xdotool
   wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines();assert len(wins)==1
   subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   result=command('POST','/execute/async',{'script':"""const done=arguments[arguments.length-1],i=window.__TAURI_INTERNALS__.invoke,sleep=ms=>new Promise(r=>setTimeout(r,ms));
   (async()=>{let results=[];for(const fault of ['connection_loss','battery_low','gps_loss','attitude_loss','sensor_stale']){
    let v;for(let k=0;k<30;k++){try{v=await i('virtual_start');break}catch(e){await sleep(20)}}if(!v)throw Error('start '+fault);
    let active=await i('virtual_input',{ticket:v.ticket,sequence:1,axes:{forward:1,right:0,up:0,yaw:0}});if(active.velocity.forward!==1)throw Error('inactive '+fault);
    let locked=await i('virtual_fault',{ticket:v.ticket,fault});if(locked.authority!=='CONTROL LOCKED'||Object.values(locked.velocity).some(x=>x!==0))throw Error('not locked '+fault);
    await sleep(150);let recovered=await i('virtual_fault',{ticket:v.ticket,fault:'qa_recover'});if(recovered.ticket!==null||recovered.authority!=='OFFLINE'||recovered.safety!=='NORMAL'||Object.values(recovered.velocity).some(x=>x!==0))throw Error('recovery state '+fault);
    let stale=await i('virtual_input',{ticket:v.ticket,sequence:1,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>false).catch(()=>true);
    let late=await i('virtual_input',{ticket:v.ticket,sequence:99999,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>false).catch(()=>true);if(!stale||!late)throw Error('old input accepted '+fault);
    await sleep(100);let idle=await i('virtual_status');if(idle.ticket!==null||idle.authority!=='OFFLINE')throw Error('auto resume '+fault);
    let next;for(let k=0;k<30;k++){try{next=await i('virtual_start');break}catch(e){await sleep(20)}}if(!next||next.ticket===v.ticket)throw Error('new session '+fault);
    let newInput=await i('virtual_input',{ticket:next.ticket,sequence:1,axes:{forward:1,right:0,up:0,yaw:0}});if(newInput.velocity.forward!==1)throw Error('new input '+fault);
    await i('virtual_stop',{ticket:next.ticket});await sleep(30);results.push(fault);
   }return {ok:true,faults:results}})().then(done).catch(e=>done({ok:false,error:String(e)}))""",'args':[]})
   assert result.get('ok') and len(result['faults'])==5,result
   return result
  check('virtual fault recovery IPC',virtual_recovery)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.virtual_session_races_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def session_races():
   assert args.xdotool
   wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines();assert len(wins)==1
   subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   result=command('POST','/execute/async',{'script':"""const done=arguments[arguments.length-1],i=window.__TAURI_INTERNALS__.invoke,sleep=ms=>new Promise(r=>setTimeout(r,ms));
   (async()=>{let rounds=[];for(let n=0;n<10;n++){
    let v;for(let retry=0;retry<30;retry++){try{v=await i('virtual_start');break}catch(e){await sleep(20)}}if(!v)throw Error('start unavailable '+n);
    await i('virtual_input',{ticket:v.ticket,sequence:1,axes:{forward:1,right:0,up:0,yaw:0}});
    let race=await Promise.allSettled([i('virtual_fault',{ticket:v.ticket,fault:'battery_low'}),i('virtual_stop',{ticket:v.ticket})]);
    await sleep(100);let status=await i('virtual_status');if(status.ticket!==null||status.authority!=='OFFLINE'||Object.values(status.velocity).some(x=>x!==0))throw Error('old session active '+n);
    let newer;for(let retry=0;retry<30;retry++){try{newer=await i('virtual_start');break}catch(e){await sleep(20)}}if(!newer)throw Error('new start unavailable '+n);
    let stale=await i('virtual_stop',{ticket:v.ticket}).then(()=>false).catch(()=>true);if(!stale)throw Error('old stop accepted '+n);
    let fresh=await i('virtual_input',{ticket:newer.ticket,sequence:1,axes:{forward:1,right:0,up:0,yaw:0}});if(fresh.velocity.forward!==1)throw Error('new authority lost '+n);
    await i('virtual_stop',{ticket:newer.ticket});rounds.push(race.map(x=>x.status));await sleep(30);
   }return {ok:true,rounds}})().then(done).catch(e=>done({ok:false,error:String(e)}))""",'args':[]})
   assert result.get('ok') and len(result['rounds'])==10,result
   return result
  check('virtual fault stop and stale stop races',session_races)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.queue_race_only or args.queue_watchdog_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def queue_race():
   assert args.xdotool
   wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines();assert len(wins)==1
   subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   state=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(done).catch(e=>done({error:String(e)}))",'args':[]});assert state.get('ticket'),state
   ticket=state['ticket'];journals=list(profile.rglob('virtual-control-events.jsonl'));assert len(journals)==1
   path=journals[0];path.rename(path.with_suffix('.saved'));os.mkfifo(path,0o600)
   result=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1],ticket=arguments[0];(async()=>{let seq=0;for(let n=0;n<180;n++){for(const f of [1,0]){try{await window.__TAURI_INTERNALS__.invoke('virtual_input',{ticket,sequence:++seq,axes:{forward:f,right:0,up:0,yaw:0}})}catch(e){return {n,seq,error:String(e)}}}}return {error:'queue did not fill'}})().then(done)",'args':[ticket]})
   assert '이벤트' in result['error'] and result['n']>=128,result
   tasks=pathlib.Path('/proc')/str(find_main_pid())/'task';assert any((p/'wchan').read_text().strip()=='wait_for_partner' for p in tasks.iterdir())
   # The stop and watchdog can race; both must fail closed even if audit fails.
   if args.queue_race_only:
    command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_stop',{ticket:arguments[0]}).then(()=>done(true)).catch(()=>done(false))",'args':[ticket]})
   time.sleep(.2)
   probe=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1],i=window.__TAURI_INTERNALS__.invoke;Promise.all([i('virtual_input',{ticket:arguments[0],sequence:99999,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>false).catch(()=>true),i('virtual_replay_start').then(t=>i('virtual_replay_stop',{ticket:t}).then(()=>true)).catch(()=>false)]).then(done)",'args':[ticket]});assert probe==[True,True],probe
   denied=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(()=>done('unexpected success')).catch(e=>done(String(e)))",'args':[]});assert '이벤트' in denied,denied
   view=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert view['ticket'] is None and view['authority']=='OFFLINE' and all(v==0 for v in view['velocity'].values()),view
   return {'queue_failure':result,'old_token_rejected':True,'replay_authority_available':True,'new_start_rejected':True}
  check('full journal queue and watchdog' if args.queue_watchdog_only else 'full journal queue and stop race',queue_race)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.minimized_ipc_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def minimize_ipc():
   assert args.xdotool
   wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines();assert len(wins)==1
   subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   state=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(done).catch(e=>done({error:String(e)}))",'args':[]});assert state.get('ticket'),state
   active=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_input',{ticket:arguments[0],sequence:1,axes:{forward:1,right:0,up:0,yaw:0}}).then(done).catch(e=>done({error:String(e)}))",'args':[state['ticket']]});assert active.get('velocity',{}).get('forward')==1,active
   command('POST','/window/minimize',{});time.sleep(.3)
   info=subprocess.check_output(['/usr/bin/xwininfo','-id',wins[0]],env=env,text=True)
   assert 'Map State: IsUnMapped' in info or 'Map State: IsUnviewable' in info, 'X11 did not minimize/unmap window'
   rejected=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_input',{ticket:arguments[0],sequence:2,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>done(false)).catch(()=>done(true))",'args':[state['ticket']]});assert rejected
   blocked=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(()=>done(false)).catch(()=>done(true))",'args':[]});assert blocked
   command('POST','/window/rect',{'width':1200,'height':800});subprocess.run([args.xdotool,'windowmap',wins[0]],env=env,check=True,timeout=5);subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   time.sleep(.2)
   view=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert view['ticket'] is None and view['authority']=='OFFLINE' and all(v==0 for v in view['velocity'].values()),view
   return 'active input invalidated and old ticket rejected through minimize and restore'
  check('main minimized IPC',minimize_ipc)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.focus_ipc_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def focus_ipc():
   assert args.xdotool
   wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines();assert len(wins)==1
   subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   response=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(v=>done({ok:true,v})).catch(e=>done({ok:false,error:String(e)}))",'args':[]});assert response['ok'],response
   ticket=response['v']['ticket']
   active=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_input',{ticket:arguments[0],sequence:1,axes:{forward:1,right:0,up:0,yaw:0}}).then(v=>done({ok:true,v})).catch(e=>done({ok:false,error:String(e)}))",'args':[ticket]});assert active['ok'] and active['v']['velocity']['forward']>0,active
   timings=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1],ticket=arguments[0];(async()=>{const a=[];for(let n=2;n<=4;n++){const t=performance.now();const v=await window.__TAURI_INTERNALS__.invoke('virtual_input',{ticket,sequence:n,axes:{forward:1,right:0,up:0,yaw:0}});if(v.velocity.forward!==1)throw Error('not active');a.push(performance.now()-t)}return a})().then(done).catch(e=>done({error:String(e)}))",'args':[ticket]});assert isinstance(timings,list) and len(timings)==3,timings
   other=subprocess.Popen(['/usr/bin/xmessage','-name','firescene-qa-focus','focus fixture'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
   try:
    time.sleep(.3); ids=subprocess.check_output([args.xdotool,'search','--onlyvisible','--name','firescene-qa-focus'],env=env,text=True).splitlines();assert len(ids)==1
    subprocess.run([args.xdotool,'windowfocus','--sync',ids[0]],env=env,check=True,timeout=5)
    wait_for("return !document.hasFocus()",5)
    outcome=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];const i=window.__TAURI_INTERNALS__.invoke;Promise.all([i('virtual_start').then(()=>false).catch(()=>true),i('virtual_input',{ticket:arguments[0],sequence:5,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>false).catch(()=>true)]).then(done)",'args':[ticket]});assert outcome==[True,True],outcome
    time.sleep(.2)
    view=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert view['ticket'] is None and view['authority']=='OFFLINE' and all(v==0 for v in view['velocity'].values()),view
   finally:
    other.terminate();other.wait(timeout=5)
   return f'main focus positive control; input IPC round trip ms {timings}; actual X11 focus loss rejects direct start and input, revokes token and neutralizes velocity'
  check('main focus loss IPC',focus_ipc)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.virtual_perf_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  menu('비행 제어',1)
  if args.xdotool:
   wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines()
   assert len(wins)==1;subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
  wait_for("return !!document.querySelector('[aria-label=\"개발용 가상 제어\"]')",15)
  check('virtual performance idle',measure_main)
  click_text('가상 제어 시작');wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('SIMULATION CONTROL')",10)
  check('virtual performance active',measure_main)
  assert js("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('SIMULATION CONTROL')")
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.journal_stall_only or args.journal_stall_stop_only or args.journal_failure_only or args.journal_write_failure_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def journal_stall():
   menu('비행 제어',1)
   if args.xdotool:
    wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines()
    assert len(wins)==1;subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   wait_for("return !!document.querySelector('[aria-label=\"개발용 가상 제어\"]')",15)
   click_text('가상 제어 시작');wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('SIMULATION CONTROL')",10)
   js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW'}))")
   time.sleep(.2)
   state=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert state['velocity']['forward']>0,state
   ticket=state['ticket']; journals=list(profile.rglob('virtual-control-events.jsonl'));assert len(journals)==1
   path=journals[0];path.rename(path.with_suffix('.saved'))
   if args.journal_failure_only or args.journal_write_failure_only:
    if args.journal_failure_only: path.mkdir() # Opening a directory as the log must fail.
    else: path.symlink_to('/dev/full') # Opening succeeds; every write fails with ENOSPC.
    js("window.dispatchEvent(new KeyboardEvent('keyup',{key:'w',code:'KeyW'}))")
    time.sleep(.8)
    result=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];const i=window.__TAURI_INTERNALS__.invoke;Promise.all([i('virtual_input',{ticket:arguments[0],sequence:99999,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>false).catch(()=>true),i('virtual_replay_start').then(t=>i('virtual_replay_stop',{ticket:t}).then(()=>true)).catch(()=>false)]).then(done)",'args':[ticket]})
    assert result==[True,True],result
    denied=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(()=>done('unexpected success')).catch(e=>done(String(e)))",'args':[]});assert '이벤트' in denied,denied
    view=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert view['ticket'] is None and view['authority']=='OFFLINE' and all(v==0 for v in view['velocity'].values()),view
    return ('isolated log write failure' if args.journal_write_failure_only else 'isolated log open failure')+': old input rejected, authority released, new virtual start rejected and velocity neutral'
   os.mkfifo(path,0o600)
   # Reset generates an event, making the writer block opening this FIFO.
   js("window.dispatchEvent(new KeyboardEvent('keyup',{key:'w',code:'KeyW'}))")
   time.sleep(.2)
   tasks=pathlib.Path('/proc')/str(find_main_pid())/'task'
   blocked_threads=[p.name for p in tasks.iterdir() if (p/'wchan').read_text().strip()=='wait_for_partner']
   assert blocked_threads, 'event writer did not block opening FIFO'
   if args.journal_stall_stop_only:
    stopped=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_stop',{ticket:arguments[0]}).then(()=>done(true)).catch(e=>done(String(e)))",'args':[ticket]});assert stopped is True,stopped
    probe=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1],i=window.__TAURI_INTERNALS__.invoke;Promise.all([i('virtual_input',{ticket:arguments[0],sequence:99999,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>false).catch(()=>true),i('virtual_replay_start').then(t=>i('virtual_replay_stop',{ticket:t}).then(()=>true)).catch(()=>false)]).then(done)",'args':[ticket]});assert probe==[True,True],probe
    denied=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(()=>done('unexpected success')).catch(e=>done(String(e)))",'args':[]});assert '이벤트' in denied,denied
    return 'user stop completed while writer blocked; old input rejected, replay authority available and new virtual start denied while audit pending'
   # Reactivate input, then freeze the webview so no input/status IPC occurs.
   js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW'}))")
   time.sleep(.2)
   active=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert active['velocity']['forward']>0,active
   began=time.monotonic()
   command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];setTimeout(()=>{const until=Date.now()+1500;while(Date.now()<until){};done(true)},0)",'args':[]})
   # FIFO still has no reader: verify the interlock independently of the journal.
   outcome=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];const i=window.__TAURI_INTERNALS__.invoke;Promise.all([i('virtual_input',{ticket:arguments[0],sequence:99999,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>false).catch(()=>true),i('virtual_replay_start').then(t=>i('virtual_replay_stop',{ticket:t}).then(()=>true)).catch(()=>false)]).then(done)",'args':[ticket]})
   assert outcome==[True,True],outcome
   view=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert view['ticket'] is None and view['authority']=='OFFLINE' and all(v==0 for v in view['velocity'].values()),view
   return f'writer blocked on isolated FIFO throughout; stale input rejected, interlock released and velocity neutral after {time.monotonic()-began:.3f}s; no hard real-time bound'
  check('watchdog with failed event writer' if args.journal_failure_only or args.journal_write_failure_only else 'watchdog with stalled event writer',journal_stall)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.journal_endurance_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def endurance():
   menu('비행 제어',1)
   if args.xdotool:
    wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines()
    assert len(wins)==1;subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   started=time.monotonic(); sessions=[]
   for batch in range(args.journal_endurance_batches):
    response=command('POST','/execute/async',{'script':"""
     const done=arguments[arguments.length-1], start=arguments[0], i=window.__TAURI_INTERNALS__.invoke;
     const sleep=ms=>new Promise(r=>setTimeout(r,ms));
     (async()=>{let result=[];for(let n=start;n<start+30;n++){
       let s;for(let k=0;k<50;k++){try{s=await i('virtual_start');break}catch(e){await sleep(20)}}
       if(!s)throw Error('start refused '+n);
       const ticket=s.ticket, a={forward:1,right:0,up:0,yaw:0};
       await i('virtual_input',{ticket,sequence:1,axes:a});
       if(n%3===0){await i('virtual_input',{ticket,sequence:2,axes:{...a,forward:0}});await i('virtual_stop',{ticket});}
       else if(n%3===1){await i('virtual_lock',{ticket});}
       await sleep(700);
       const state=await i('virtual_status');if(state.ticket!==null||Object.values(state.velocity).some(v=>v!==0))throw Error('not neutral '+n);
       result.push({ticket,mode:n%3});
     }return result})().then(done).catch(e=>done({error:String(e)}));
    """,'args':[batch*30]})
    assert isinstance(response,list) and len(response)==30,response
    sessions.extend(response)
    print(f'endurance batch {batch+1}/{args.journal_endurance_batches}',flush=True)
   elapsed=time.monotonic()-started; time.sleep(.3)
   journals=list(profile.rglob('virtual-control-events.jsonl'));assert len(journals)==1,journals
   events=[json.loads(line) for line in journals[0].read_text().splitlines()]
   assert len({x['ticket'] for x in sessions})==30*args.journal_endurance_batches
   expected={0:['virtual_started','authority_granted','input_reset','authority_released','virtual_stopped'],1:['virtual_started','authority_granted','safety_lock','safety_lock','authority_released','virtual_stopped'],2:['virtual_started','authority_granted','input_timeout','authority_released','virtual_stopped']}
   for session in sessions:
    actual=[e['code'] for e in events if e['session']==session['ticket']]
    assert actual==expected[session['mode']],(session,actual)
   assert len(events)==sum(len(expected[x['mode']]) for x in sessions)
   assert all(e['source']=='VIRTUAL_MEMORY' for e in events)
   shutil.copyfile(journals[0],out/'virtual-events.jsonl')
   return f'{len(sessions)} sessions, {len(events)} events, {elapsed:.1f}s; expected per-session order and count, no unexpected or missing records'
  check('virtual journal endurance',endurance)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.virtual_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def virtual_check():
   menu('비행 제어',1)
   if args.xdotool:
    wins=subprocess.check_output([args.xdotool,'search','--onlyvisible','--pid',str(find_main_pid())],env=env,text=True).splitlines()
    assert len(wins)==1;subprocess.run([args.xdotool,'windowfocus','--sync',wins[0]],env=env,check=True,timeout=5)
   wait_for("return !!document.querySelector('[aria-label=\"개발용 가상 제어\"]')",15)
   click_text('가상 제어 시작');wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('SIMULATION CONTROL')",10)
   blocked=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_replay_start').then(()=>done(false)).catch(()=>done(true))",'args':[]});assert blocked
   blocked=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('connect_forwarded',{port:14551,route:'qgc'}).then(()=>done(false)).catch(()=>done(true))",'args':[]});assert blocked
   js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW'}))")
   wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('전후 1')")
   time.sleep(.6)
   state=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert state['north']>0,state
   js("window.dispatchEvent(new KeyboardEvent('keyup',{key:'w',code:'KeyW'}))")
   wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('전후 0')")
   time.sleep(.3);state=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert state['velocity']['forward']==0,state
   # Exercise simultaneous opposing keys in the real virtual session, and
   # observe backend velocity after each frontend key-state transition.
   def key(kind, code):
    js("window.dispatchEvent(new KeyboardEvent(arguments[0],{key:arguments[1].slice(-1).toLowerCase(),code:arguments[1]}))",kind,code)
   def expect_axes(forward,right):
    deadline=time.monotonic()+2
    while time.monotonic()<deadline:
     view=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]})
     if view['velocity']['forward']==forward and view['velocity']['right']==right:return view
     time.sleep(.04)
    raise AssertionError(f'expected forward/right {forward}/{right}; got {view}')
   key('keydown','KeyW');expect_axes(1,0)
   key('keydown','KeyS');expect_axes(0,0)
   key('keydown','KeyA');expect_axes(0,-1)
   key('keydown','KeyD');opposed=expect_axes(0,0);opposed_ticket=opposed['ticket']
   key('keyup','KeyW');expect_axes(-1,0)
   key('keyup','KeyA');expect_axes(-1,1)
   key('keyup','KeyS');expect_axes(0,1)
   key('keyup','KeyD');expect_axes(0,0)
   screenshot('virtual-control')
   js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',code:'Escape'}))")
   wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('OFFLINE')")
   stale=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_input',{ticket:arguments[0],sequence:99999,axes:{forward:1,right:1,up:0,yaw:0}}).then(()=>done(false)).catch(()=>done(true))",'args':[opposed_ticket]});assert stale
   time.sleep(.3)
   journals=list(profile.rglob('virtual-control-events.jsonl'));assert len(journals)==1,journals
   journal=[json.loads(line) for line in journals[0].read_text().splitlines()]
   assert all(e['source']=='VIRTUAL_MEMORY' for e in journal)
   assert {'virtual_started','authority_granted','input_reset','authority_released','virtual_stopped'} <= {e['code'] for e in journal},journal
   shutil.copyfile(journals[0],out/'virtual-events.jsonl')
   reverse=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];const i=window.__TAURI_INTERNALS__.invoke;i('virtual_replay_start').then(t=>i('virtual_start').then(()=>false).catch(()=>true).then(ok=>i('virtual_replay_stop',{ticket:t}).then(()=>done(ok)))).catch(()=>done(false))",'args':[]});assert reverse
   for fault in ['연결 손실','배터리 부족','GPS 손실','자세 지연','센서 지연']:
    click_text('가상 제어 시작');wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('SIMULATION CONTROL')",10)
    js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW'}))")
    wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('전후 1')")
    time.sleep(.2)
    active=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert active['velocity']['forward']>0,active
    old_ticket=active['ticket']
    click_text(fault);wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('가상 제어 잠김')",10)
    wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('OFFLINE')",10);time.sleep(.2)
    stopped=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert stopped['ticket'] is None and stopped['authority']=='OFFLINE' and all(v==0 for v in stopped['velocity'].values()),stopped
    blocked=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_input',{ticket:arguments[0],sequence:9999,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>done(false)).catch(()=>done(true))",'args':[old_ticket]});assert blocked
    js("window.dispatchEvent(new KeyboardEvent('keyup',{key:'w',code:'KeyW'}))")
   journal=[json.loads(line) for line in journals[0].read_text().splitlines()]
   shutil.copyfile(journals[0],out/'virtual-events-before-watchdog.jsonl')
   assert sum(e['code']=='simulated_warning' for e in journal)==5,journal
   assert sum(e['code']=='simulated_warning_ended' for e in journal)==5,journal
   assert {e.get('reason') for e in journal if e['code']=='simulated_warning'}=={'connection_loss','battery_low','gps_loss','attitude_loss','sensor_stale'},journal
   # Stop every frontend input request without completing or rejecting it. Do
   # not poll virtual_status: the journal and interlock prove independent expiry.
   click_text('가상 제어 시작');wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('SIMULATION CONTROL')",10)
   js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW'}))")
   wait_for("return document.querySelector('[aria-label=\"개발용 가상 제어\"]')?.innerText.includes('전후 1')")
   time.sleep(.2)
   state=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_status').then(done)",'args':[]});assert state['velocity']['forward']>0,state
   ticket=state['ticket']
   # Block the webview event loop itself; the background thread waits for the
   # WebDriver response while this thread watches only the backend journal.
   freeze=threading.Thread(target=lambda: command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];setTimeout(()=>{const until=Date.now()+1800;while(Date.now()<until){};done(true)},0)",'args':[]}))
   began=time.monotonic()
   freeze.start()
   while time.monotonic()-began<3:
    journal=[json.loads(line) for line in journals[0].read_text().splitlines()]
    if any(e['session']==ticket and e['code']=='authority_released' for e in journal): break
    time.sleep(.02)
   else: raise AssertionError('watchdog did not release without IPC')
   elapsed=time.monotonic()-began
   shutil.copyfile(journals[0],out/'virtual-events-watchdog-observed.jsonl')
   assert any(e['session']==ticket and e['code']=='input_timeout' for e in journal),journal
   freeze.join(timeout=5);assert not freeze.is_alive()
   rejected=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];const i=window.__TAURI_INTERNALS__.invoke; i('virtual_input',{ticket:arguments[0],sequence:9999,axes:{forward:1,right:0,up:0,yaw:0}}).then(()=>done(false)).catch(()=>done(true))",'args':[ticket]});assert rejected
   available=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];const i=window.__TAURI_INTERNALS__.invoke;i('virtual_replay_start').then(t=>i('virtual_replay_stop',{ticket:t}).then(()=>done(true))).catch(()=>done(false))",'args':[]});assert available
   shutil.copyfile(journals[0],out/'virtual-events.jsonl')
   return f'five simulated safety locks; watchdog released active input without frontend calls in {elapsed:.3f}s; stale input rejected and replay authority available'
  check('virtual control GUI',virtual_check)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.replay_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  def replay_check():
   global feed
   menu('시스템 설정',8);wait_for("return [...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='시뮬레이션 수신 시작'&&!e.disabled)",15);click_text('시뮬레이션 수신 시작')
   feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'normal'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
   wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
   menu('비행 기록',7);e=command('POST','/element',{'using':'css selector','value':'[aria-label="기록 이름"]'});eid=e['element-6066-11e4-a52e-4f735466cecf'];command('POST','/element/'+eid+'/value',{'text':'QA replay gate'})
   click_text('기록 시작');wait_for("return document.body.innerText.includes('기록 중 ·')",10);time.sleep(2);click_text('기록 종료');wait_for("return [...document.querySelectorAll('.recording-list button')].some(e=>e.textContent.includes('QA replay gate'))",10)
   menu('시스템 설정',8);click_text('연결 해제');wait_for("return !document.querySelector('.connection-page')?.innerText.includes('42 / 1')",10);menu('비행 기록',7)
   js("[...document.querySelectorAll('.recording-list button')].find(e=>e.textContent.includes('QA replay gate')).click()")
   wait_for("return document.body.innerText.includes('REPLAY · 과거 기록')",15)
   blocked=command('POST','/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke('virtual_start').then(()=>done(false)).catch(()=>done(true))",'args':[]});assert blocked
   wait_for("return [...document.querySelectorAll('.replay-controls button')].some(e=>e.textContent.trim()==='재생'&&!e.disabled)",15);click_text('재생');wait_for("return document.body.innerText.includes('일시정지')")
   return 'synthetic recording and replay succeed while virtual start is denied'
  check('recording replay gate',replay_check)
  sys.exit(any(x['status']=='FAIL' for x in results))
 if args.instruments_only:
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  click_text('English');click_text('Skip setup');wait_for("return !document.querySelector('.first-run[role=dialog]')")
  check('fixed synthetic instrument values',instrument_values)
  sys.exit(any(x['status']=='FAIL' for x in results))
 def first_run():
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  wait_for("return document.activeElement?.closest('.first-run') !== null")
  command('POST','/actions',{'actions':[{'type':'key','id':'keyboard','actions':[{'type':'keyDown','value':'\ue004'},{'type':'keyUp','value':'\ue004'}]}]})
  assert js("return document.activeElement?.textContent.trim()==='English'")
  command('POST','/actions',{'actions':[{'type':'key','id':'keyboard','actions':[{'type':'keyDown','value':'\ue008'},{'type':'keyDown','value':'\ue004'},{'type':'keyUp','value':'\ue004'},{'type':'keyUp','value':'\ue008'}]}]})
  assert js("return document.activeElement?.textContent.trim()==='한국어'")
  assert js("const d=document.querySelector('.first-run'),b=d.querySelectorAll('button');b[0].focus();b[0].dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',shiftKey:true,bubbles:true,cancelable:true}));return document.activeElement===b[b.length-1]")
  screenshot('first-run-ko')
  click_text('English'); wait_for("return document.querySelector('#first-run-title')?.innerText==='Welcome to FireScene GCS'")
  assert js("return document.querySelector('.first-run')?.innerText.includes('rest of the application is currently available in Korean')")
  click_text('Day'); wait_for("return document.querySelector('.first-run')?.classList.contains('theme-day')")
  screenshot('first-run-en-day'); click_text('Dark'); click_text('Skip setup')
  wait_for("return !document.querySelector('.first-run[role=dialog]')")
  assert js("return localStorage.getItem('firescene.setup.complete.v1')==='yes'")
  wait_for("return !!document.activeElement?.matches('nav[aria-label=\"주 메뉴\"] button')")
  js("localStorage.removeItem('firescene.setup.complete.v1')"); command('POST','/refresh',{})
  wait_for("return !!document.querySelector('.first-run[role=dialog]')")
  js("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}))")
  wait_for("return !document.querySelector('.first-run[role=dialog]') && localStorage.getItem('firescene.setup.complete.v1')==='yes'")
  wait_for("return !!document.activeElement?.matches('nav[aria-label=\"주 메뉴\"] button')")
  command('POST','/refresh',{}); wait_for("return !!document.querySelector('nav[aria-label=\"주 메뉴\"]')",20)
  assert js("return !document.querySelector('.first-run[role=dialog]') && !!document.querySelector('.theme-dark')")
 check('first run setup and skip',first_run)
 results.append({'name':'startup','status':'PASS','detail':f'{time.monotonic()-start:.2f}s to navigation'}); print('PASS startup',flush=True)
 screenshot('overview-initial')
 check('idle main process sample',measure_main)
 names=['종합 관제','비행 제어','임무 계획','자율비행','3D 매핑','카메라','안전 관리','비행 기록','시스템 설정']
 for i,n in enumerate(names): check('menu '+n,lambda n=n,i=i:menu(n,i))
 def themes():
  click_text('Day'); wait_for("return !!document.querySelector('.theme-day')"); menu('종합 관제',0); screenshot('day'); click_text('Dark'); wait_for("return !!document.querySelector('.theme-dark')")
 check('theme switch',themes)
 def focus_views():
  for name,label in [('종합 관제','지도 집중 보기'),('3D 매핑','3D 매핑 집중 보기'),('카메라','카메라 집중 보기')]:
   menu(name,0); click_text(label); wait_for("return !!document.querySelector('.body.focus-mode')")
   assert js("return !!document.querySelector('.status-strip .safety-banner') && !!document.querySelector('.strip-lock')")
   screenshot('focus-'+str(names.index(name)))
   click_text('일반 보기'); wait_for("return !document.querySelector('.body.focus-mode')")
  return 'focus and restore; safety and command lock remain visible'
 check('focus views',focus_views)
 def native_fullscreen():
  menu('종합 관제',0); click_text('전체 화면'); wait_for("return [...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='전체 화면 종료')")
  click_text('전체 화면 종료'); wait_for("return [...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='전체 화면')")
 check('native fullscreen',native_fullscreen)
 def flight_and_autonomy():
  menu('비행 제어',1); assert js("return document.body.innerText.includes('자세 정보 확인 불가') && !!document.querySelector('.flight-graphs:not([open])')")
  js("document.querySelector('.flight-graphs summary').click()");wait_for("return !!document.querySelector('.flight-graphs[open]')")
  menu('자율비행',3); assert js("return document.querySelector('.autonomy-page')?.innerText.includes('비행 명령 잠김') && document.body.innerText.includes('아직 사용할 수 없습니다')")
  return 'unavailable attitude, expandable graphs and locked autonomy'
 check('flight and autonomy states',flight_and_autonomy)
 def spatial_controls():
  menu('3D 매핑',4); js("document.querySelector('.spatial-connection summary').click()")
  wait_for("return !!document.querySelector('.spatial-connection[open] input')")
  state=js("const c=document.querySelector('.spatial-canvas canvas');const gl=c.getContext('webgl');return {webgl:!!gl,width:c.width,height:c.height,clientWidth:c.clientWidth,clientHeight:c.clientHeight,dpr:Math.min(devicePixelRatio||1,2)}")
  if not state['webgl']: raise RuntimeError('WebGL unavailable in this environment')
  assert abs(state['width']-round(state['clientWidth']*state['dpr']))<=1 and abs(state['height']-round(state['clientHeight']*state['dpr']))<=1, state
  click_text('3D 매핑 집중 보기')
  wait_for("const c=document.querySelector('.spatial-canvas canvas');return Math.abs(c.width-Math.round(c.clientWidth*Math.min(devicePixelRatio||1,2)))<=1")
  click_text('일반 보기')
  return 'connection controls accessible; WebGL backing size follows focus resize'
 check('3D controls and WebGL resize',spatial_controls)
 if args.fixtures:
  def synthetic_camera():
   menu('카메라',5)
   fixture=subprocess.Popen(['python3',str(ROOT/'tools/camera_perception_fixture.py')],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL); fixtures.append(fixture); time.sleep(.5)
   for selector,value in [('.camera-connection input[placeholder="192.168.1.10"]','127.0.0.1'),('.camera-connection input[placeholder="8766"]','8766')]:
    e=command('POST','/element',{'using':'css selector','value':selector}); command('POST','/element/'+e['element-6066-11e4-a52e-4f735466cecf']+'/value',{'text':value})
   js("document.querySelector('.camera-connection button').click()")
   wait_for("return !!document.querySelector('.camera-view img') && document.querySelectorAll('.camera-view svg rect').length===1",15)
   click_text('카메라 집중 보기');wait_for("return !!document.querySelector('.body.focus-mode .camera-view img')")
   assert js("const e=document.querySelector('.camera-view').getBoundingClientRect();return Math.abs(e.width/e.height-4/3)<0.02")
   wait_for("const r=document.querySelector('.camera-view').getBoundingClientRect();return r.top>=0&&r.bottom<=innerHeight",10)
   screenshot('synthetic-camera-focus');command('POST','/window/rect',{'width':910,'height':512})
   wait_for("const r=document.querySelector('.camera-view').getBoundingClientRect();return r.top>=0&&r.bottom<=innerHeight&&Math.abs(r.width/r.height-4/3)<0.02",10)
   screenshot('synthetic-camera-compact');command('POST','/window/rect',{'width':1200,'height':800});click_text('일반 보기')
   return 'synthetic 640x480 frame, detection overlay and focus aspect ratio'
  check('synthetic camera display',synthetic_camera)
  def synthetic_spatial():
   menu('3D 매핑',4)
   fixture=subprocess.Popen(['python3',str(ROOT/'tools/spatial_fixture.py')],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL); fixtures.append(fixture); time.sleep(.5)
   js("document.querySelector('.spatial-connection summary').click()")
   for selector,value in [('.spatial-connection input[placeholder="192.168.1.10"]','127.0.0.1'),('.spatial-connection input[placeholder="8767"]','8767')]:
    e=command('POST','/element',{'using':'css selector','value':selector}); command('POST','/element/'+e['element-6066-11e4-a52e-4f735466cecf']+'/value',{'text':value})
   js("document.querySelector('.spatial-connection button').click()")
   wait_for("return document.querySelector('.spatial-page')?.innerText.includes('렌더링 4000점')",15)
   click_text('3D 매핑 집중 보기'); screenshot('synthetic-spatial-focus'); click_text('일반 보기')
   return 'synthetic point cloud received and rendered'
  check('synthetic point cloud display',synthetic_spatial)
 if args.matrix:
  for width,height in [(910,512),(1093,614),(1366,768),(1440,900),(1920,1080),(2560,1440)]:
   def matrix_size(width=width,height=height):
    command('POST','/window/rect',{'width':width,'height':height})
    rect=command('GET','/window/rect')
    if rect['width'] != width or rect['height'] != height: raise RuntimeError(f'requested {width}x{height}, got {rect}')
    for theme in ('dark','day'):
     if theme=='day': click_text('Day')
     for i,name in enumerate(names):
      menu(name,i)
      state=js("const s=document.querySelector('.status-strip'),l=document.querySelector('.strip-lock'),a=document.querySelector('.safety-banner');const within=e=>{const r=e.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight};return {lock:within(l),safety:within(a),strip:s.scrollWidth<=s.clientWidth+1,viewport:[innerWidth,innerHeight]}")
      if not (state['lock'] and state['safety'] and state['strip']): raise AssertionError(f'{name} {theme} {state}')
      if name=='종합 관제' and width<=1150:
       map_width=js("return document.querySelector('.overview-shell .map-real').getBoundingClientRect().width")
       assert map_width >= width-110, f'map too narrow: {map_width}'
       click_text('상황 정보'); wait_for("return !!document.querySelector('.overview-context')")
       click_text('상황 정보'); wait_for("return !document.querySelector('.overview-context')")
       click_text('메뉴 펼치기'); wait_for("return document.querySelector('.nav-toggle')?.getAttribute('aria-expanded')==='true'")
       click_text('메뉴 접기'); wait_for("return document.querySelector('.nav-toggle')?.getAttribute('aria-expanded')==='false'")
      if name!='시스템 설정': screenshot(f'matrix-{width}x{height}-{theme}-{i}')
      else: screenshot_settings(f'matrix-{width}x{height}-{theme}-{i}-redacted')
     if theme=='day': click_text('Dark')
    return f'window {width}x{height}; both themes, nine menus; safety and lock in viewport'
   check(f'matrix {width}x{height}',matrix_size)
 def nav_and_context():
  menu('종합 관제',0); click_text('메뉴 접기'); wait_for("return document.querySelector('.body')?.classList.contains('nav-collapsed')"); click_text('메뉴 펼치기'); wait_for("return document.querySelector('.body')?.classList.contains('nav-expanded')")
  click_text('기체'); wait_for("return document.body.innerText.includes('상세 비행 정보 및 그래프')"); click_text('안전'); wait_for("return document.body.innerText.includes('안전 감시는 보조 정보')"); click_text('현장'); wait_for("return document.body.innerText.includes('선택된 현장 없음')")
 check('navigation and context panels',nav_and_context)
 def resize():
  command('POST','/window/rect',{'width':1100,'height':760}); r=command('GET','/window/rect'); assert r['width']>=850 and r['height']>=620
  command('POST','/window/maximize',{}); command('POST','/window/rect',{'width':1200,'height':800})
 check('window resize maximize restore',resize)
 def overview_fits():
  menu('종합 관제',0); assert js("const e=document.querySelector('.overview-workspace');return e.scrollHeight<=e.clientHeight+1")
 check('overview workspace fits 1200x800',overview_fits)
 check('offline practice basic',keyboard)
 def map_search():
  menu('종합 관제',0); wait_for("return !!document.querySelector('[aria-label=\"위치 검색\"]')")
  if not args.with_map_key: raise RuntimeError('map key not enabled')
  el=command('POST','/element',{'using':'css selector','value':'[aria-label="위치 검색"]'}); eid=el['element-6066-11e4-a52e-4f735466cecf']
  command('POST','/element/'+eid+'/value',{'text':'동의대학교'}); click_text('검색')
  wait_for("return !!document.querySelector('.map-results button')",25)
  count=js("return document.querySelectorAll('.map-results button').length")
  js("document.querySelector('.map-results button').click()")
  wait_for("return document.body.innerText.includes('검색 위치로 이동')")
  click_text('검색 위치를 현장으로 선택')
  wait_for("return document.body.innerText.includes('검색 위치를 현장으로 선택했습니다')")
  return str(count)+' results; location explicitly selected'
 check('live map search',map_search)
 def layers():
  assert js("return [...document.querySelectorAll('.map-toolbar button')].find(b=>b.textContent.trim()==='3D 실험')?.disabled === true")
  click_text('위성'); wait_for("return document.querySelector('.map-toolbar button[aria-pressed=true]')?.innerText==='위성'"); click_text('기본'); wait_for("return document.querySelector('.map-toolbar button[aria-pressed=true]')?.innerText==='기본'")
 def globe_experiment():
  menu('종합 관제',0); click_text('Global 실험')
  wait_for("return !!document.querySelector('.globe-experiment')",30)
  wait_for("return document.querySelector('.globe-experiment [role=status]')?.innerText.includes('OpenFreeMap')",45)
  wait_for("return !!document.querySelector('.globe-mission-marker')",20)
  time.sleep(8); screenshot('global-experiment')
  click_text('임무 위치 확대'); time.sleep(3); screenshot('global-mission-zoom')
  click_text('평면'); time.sleep(2); screenshot('global-flat')
  click_text('지구본'); time.sleep(2)
  click_text('기본 지도 복귀'); wait_for("return !document.querySelector('.globe-experiment')")
  wait_for("return document.querySelector('.map-area')?.textContent.includes('검색 위치 선택')")
  return 'Tauri WebKit globe/flat and retained mission selection'
 if args.dev and args.with_map_key: check('experimental globe in Tauri',globe_experiment)
 if args.dev and args.with_map_key:
  def globe_context_loss():
   menu('종합 관제',0); click_text('Global 실험')
   wait_for("return !!document.querySelector('.globe-map canvas')",30)
   js("document.querySelector('.globe-map canvas').dispatchEvent(new Event('webglcontextlost',{cancelable:true}))")
   wait_for("return document.querySelector('.globe-experiment [role=status]')?.innerText.includes('WebGL 연결 손실')",15)
   click_text('기본 지도 복귀'); wait_for("return !document.querySelector('.globe-experiment')")
   return 'synthetic context loss event, warning and tactical return'
  check('global context loss isolation',globe_context_loss)

 check('map layer selection and 3D disabled',layers)
 check('map main process sample',measure_main)
 def qgc_error():
  menu('시스템 설정',8)
  field=js("return [...document.querySelectorAll('label')].find(e=>e.innerText.includes('실행 파일 절대 경로'))?.querySelector('input')?.getAttribute('placeholder')")
  assert field
  element=command('POST','/element',{'using':'css selector','value':'input[placeholder="/home/user/Downloads/QGroundControl.AppImage"]'})
  eid=element['element-6066-11e4-a52e-4f735466cecf']; command('POST','/element/'+eid+'/clear',{}); command('POST','/element/'+eid+'/value',{'text':'/tmp/does-not-exist/QGroundControl.AppImage'})
  click_text('QGroundControl 실행'); wait_for("return document.body.innerText.includes('실행 파일을 찾을 수 없습니다')")
 check('QGroundControl missing executable error',qgc_error)
 def settings_groups():
  menu('시스템 설정',8); click_text('지도 서비스')
  wait_for("const e=document.querySelector('#settings-map'),w=document.querySelector('.workspace');return e && w && e.getBoundingClientRect().top < w.getBoundingClientRect().bottom && w.scrollTop>0")
  click_text('기체 연결'); wait_for("const e=document.querySelector('#settings-aircraft'),w=document.querySelector('.workspace');return e && w && e.getBoundingClientRect().top < w.getBoundingClientRect().bottom")
  return 'map and aircraft shortcuts reach their settings'
 check('settings groups',settings_groups)
 def draft_mission():
  menu('임무 계획',2); wait_for("return [...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='새 임무'&&!e.disabled)")
  click_text('새 임무'); wait_for("return !!document.querySelector('.mission-fields input')")
  element=command('POST','/element',{'using':'css selector','value':'.mission-fields input'}); eid=element['element-6066-11e4-a52e-4f735466cecf']
  command('POST','/element/'+eid+'/clear',{}); command('POST','/element/'+eid+'/value',{'text':'QA isolated mission'})
  click_text('작성 중 저장'); wait_for("return document.body.innerText.includes('작성 중 임무를 저장했습니다')")
  click_text('복제'); wait_for("return document.body.innerText.includes('복제본을 작성 중 상태로 저장했습니다')")
  screenshot('mission-editor-synthetic')
  command('POST','/window/rect',{'width':910,'height':512}); wait_for("return !!document.querySelector('.mission-editor .map-info-disclosure')")
  wait_for("const p=document.querySelector('.mission-editor .map-canvas'),c=p?.querySelector('canvas');if(!p||!c)return false;const a=p.getBoundingClientRect(),b=c.getBoundingClientRect();return Math.abs(a.width-b.width)<2&&Math.abs(a.height-b.height)<2",15)
  time.sleep(2)  # Allow network tiles to repaint before the visual evidence capture.
  screenshot('mission-editor-compact'); command('POST','/window/rect',{'width':1200,'height':800})
  menu('종합 관제',0); wait_for("return document.querySelector('.header-context')?.innerText.includes('QA isolated mission')")
  return 'created and duplicated isolated draft; overview mission shown'
 check('isolated mission draft and duplicate',draft_mission)
 def mission_delete_cancel():
  menu('임무 계획',2); wait_for("return [...document.querySelectorAll('.mission-list button')].some(e=>e.textContent.includes('QA isolated mission'))")
  js("[...document.querySelectorAll('.mission-list button')].find(e=>e.textContent.includes('QA isolated mission')).click()")
  wait_for("return [...document.querySelectorAll('.mission-actions button')].some(e=>e.textContent.trim()==='삭제')")
  js("setTimeout(()=>[...document.querySelectorAll('.mission-actions button')].find(e=>e.textContent.trim()==='삭제').click(),0)")
  try:
   deadline=time.monotonic()+5; message=None
   while time.monotonic()<deadline:
    try: message=command('GET','/alert/text'); break
    except Exception: time.sleep(.1)
   assert message and '삭제' in message
   command('POST','/alert/dismiss',{})
  finally:
   try: command('POST','/alert/dismiss',{})
   except Exception: pass
  assert js("return [...document.querySelectorAll('.mission-list button')].some(e=>e.textContent.includes('QA isolated mission'))")
  return 'native confirmation cancelled; isolated mission remains'
 check('mission delete confirmation cancel',mission_delete_cancel)
 if args.dev:
  def sim_connect():
   global feed
   menu('시스템 설정',8); wait_for("return document.body.innerText.includes('시뮬레이션 수신 시작')")
   click_text('시뮬레이션 수신 시작')
   feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'battery-low'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
   wait_for("return document.body.innerText.includes('42 / 1')",15)
   wait_for(r"return /최근 링크 유효 프레임 수신 \(다른 컴포넌트 포함 가능\):\s*[0-9.]+ 프레임\/초/.test(document.body.innerText)",15)
   wait_for(r"return /마지막 유효 기체 HEARTBEAT 간격:\s*[0-9.]+초/.test(document.body.innerText)",15)
   js("document.querySelector('.connection-page .connection-panel:last-child')?.scrollIntoView({block:'start'})")
   screenshot_settings('connection-synthetic-metrics-redacted')
   return 'simulation receiver and measured frame rate and heartbeat interval'
  check('simulation heartbeat receive',sim_connect)
  def compact_live_status():
   command('POST','/window/rect',{'width':910,'height':512}); menu('종합 관제',0)
   wait_for("return document.querySelector('.status-strip')?.innerText.includes('시뮬레이션')")
   assert js("const b=document.querySelector('.header-context b'),old=b.textContent;b.textContent='임무'.repeat(50);const buttons=[...document.querySelectorAll('.header button')],ok=b.scrollWidth>b.clientWidth&&buttons.every(e=>e.getBoundingClientRect().right<=innerWidth);b.textContent=old;return ok")
   size=js("const e=document.querySelector('.overview-shell .map-canvas'),w=document.querySelector('.overview-workspace');return {height:e.getBoundingClientRect().height,scroll:w.scrollHeight-w.clientHeight}")
   screenshot('compact-live-overview')
   assert size['height']>=150, f'compact map canvas too short: {size}'
   assert size['scroll']<=1, f'compact overview has nested workspace overflow: {size}'
   assert js("return document.querySelector('.status-strip .safety-banner')?.getAttribute('aria-label')?.includes(' · ')")
   assert js("const s=document.querySelector('.status-strip'),a=s.querySelector('.safety-banner'),l=s.querySelector('.strip-lock');const inside=e=>{const r=e.getBoundingClientRect();return r.right<=innerWidth&&r.bottom<=innerHeight};return s.scrollWidth<=s.clientWidth+1&&inside(a)&&inside(l)")
   e=command('POST','/element',{'using':'css selector','value':'[aria-label="위치 검색"]'}); eid=e['element-6066-11e4-a52e-4f735466cecf']; command('POST','/element/'+eid+'/clear',{}); command('POST','/element/'+eid+'/value',{'text':'동의대학교'}); click_text('검색')
   wait_for("return !!document.querySelector('.map-results button')",25)
   assert js("const r=document.querySelector('.map-results button').getBoundingClientRect();return r.top>=0&&r.top<innerHeight&&r.left>=0&&r.right<=innerWidth")
   command('POST','/window/rect',{'width':1200,'height':800})
   return f"map canvas {size['height']:.0f}px; workspace scroll overflow {size['scroll']}px"
  check('compact live source and safety',compact_live_status)
  if args.with_map_key:
   def globe_simulation():
    menu('종합 관제',0); click_text('Global 실험')
    wait_for("return !!document.querySelector('.globe-aircraft-heading')",30)
    wait_for("return document.querySelector('.globe-experiment [role=status]')?.innerText.includes('시뮬레이션 기체')")
    screenshot('global-simulation-heading')
    click_text('기본 지도 복귀'); wait_for("return !document.querySelector('.globe-experiment')")
    return 'synthetic heading marker and simulation source'
   check('global synthetic aircraft heading',globe_simulation)

  def live_safety():
   menu('안전 관리',6); wait_for("return document.querySelector('.safety-page')?.innerText.includes('배터리 부족')",15)
   screenshot('safety-simulation'); assert js("return !!document.querySelector('.safety-event time[datetime]')")
   return 'synthetic battery warning with timestamp shown'
  check('simulation safety event display',live_safety)
  def live_instruments():
   menu('비행 제어',1); wait_for("return !!document.querySelector('.attitude .horizon[role=img]')",15)
   screenshot('flight-instruments-simulation')
   return 'synthetic attitude shown in the instrument'
  check('simulation flight instruments',live_instruments)
  def overview_telemetry():
   menu('종합 관제',0); wait_for("return document.querySelector('.overview-flight-values')?.innerText.includes('m') && document.body.innerText.includes('PX4 HEARTBEAT 수신')")
  check('overview live telemetry',overview_telemetry)
  check('simulation main process sample',measure_main)
  def recording():
   menu('비행 기록',7); wait_for("return !!document.querySelector('[aria-label=\"기록 이름\"]')")
   e=command('POST','/element',{'using':'css selector','value':'[aria-label="기록 이름"]'}); eid=e['element-6066-11e4-a52e-4f735466cecf']; command('POST','/element/'+eid+'/value',{'text':'QA synthetic recording'})
   click_text('기록 시작'); wait_for("return document.body.innerText.includes('기록 중 ·')",10); wait_for("return document.body.innerText.includes('배터리 부족')",15)
   click_text('기록 종료'); wait_for("return document.querySelector('.recordings-page .source-label')?.innerText.includes('기록 중 아님')",12); wait_for("return [...document.querySelectorAll('.recording-list button')].some(e=>e.textContent.includes('QA synthetic recording'))",12)
   js("[...document.querySelectorAll('.recording-list button')].find(e=>e.textContent.includes('QA synthetic recording')).click()")
   wait_for("return document.body.innerText.includes('REPLAY · 과거 기록')",10)
   screenshot('recording-replay-synthetic')
   js("setTimeout(()=>document.querySelector('.replay-panel .danger-action').click(),0)")
   try:
    deadline=time.monotonic()+5; message=None
    while time.monotonic()<deadline:
     try: message=command('GET','/alert/text'); break
     except Exception: time.sleep(.1)
    assert message and '삭제' in message
    command('POST','/alert/dismiss',{})
   finally:
    try: command('POST','/alert/dismiss',{})
    except Exception: pass
   assert js("return document.body.innerText.includes('REPLAY · 과거 기록') && [...document.querySelectorAll('.recording-list button')].some(e=>e.textContent.includes('QA synthetic recording'))")
   wait_for("return [...document.querySelectorAll('.replay-controls button')].some(e=>e.textContent.trim()==='재생'&&!e.disabled)",12); click_text('재생'); wait_for("return document.body.innerText.includes('일시정지')"); click_text('일시정지')
   js("const e=document.querySelector('[aria-label=\"재생 속도\"]');e.value='2';e.dispatchEvent(new Event('change',{bubbles:true}))")
   click_text('JSONL 내보내기'); wait_for("return document.body.innerText.includes('내보내기 완료')")
   return 'low battery warning; synthetic record saved, delete cancelled, replayed, paused and exported to isolated data'
  check('simulation recording and replay',recording)
  def loss_and_manual_reconnect():
   global feed
   feed.terminate(); feed.wait(timeout=5); feed=None
   menu('시스템 설정',8)
   wait_for("return document.querySelector('.connection-page')?.innerText.includes('연결 끊김')",15)
   screenshot_settings('connection-loss-redacted')
   menu('비행 제어',1)
   wait_for("return !!document.querySelector('.attitude-empty') && !document.querySelector('.attitude .horizon')",10)
   screenshot('flight-after-loss')
   menu('시스템 설정',8); click_text('시뮬레이션 수신 시작')
   wait_for("return document.querySelector('.connection-page')?.innerText.includes('HEARTBEAT 대기')",10)
   assert js("return !document.querySelector('.connection-page')?.innerText.includes('42 / 1')")
   feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'normal'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
   wait_for("return document.querySelector('.connection-page')?.innerText.includes('42 / 1')",15)
   screenshot_settings('connection-recovered-redacted')
   menu('비행 제어',1); wait_for("return !!document.querySelector('.attitude .horizon[role=img]')",10)
   menu('시스템 설정',8); click_text('연결 해제')
   wait_for("return document.querySelector('.connection-page')?.innerText.includes('사용자 해제 · 자동 연결 꺼짐')",10)
   time.sleep(3)
   assert js("return !document.querySelector('.connection-page')?.innerText.includes('42 / 1')")
   return 'UDP simulation loss clears instruments, manual new session restores data, user disconnect stays offline'
  check('simulation loss and manual reconnect',loss_and_manual_reconnect)
  def restart_data():
   global sid
   command('DELETE',''); sid=None
   response=call('POST','/session',{'capabilities':{'alwaysMatch':{'browserName':'wry','tauri:options':{'application':str(pathlib.Path(args.app).resolve())}}}})
   sid=response['sessionId']; wait_for("return !!document.querySelector('nav[aria-label=\"주 메뉴\"]')",20)
   assert js("return !document.querySelector('.first-run') && localStorage.getItem('firescene.setup.complete.v1')==='yes' && localStorage.getItem('firescene.ui.theme')==='dark'")
   menu('임무 계획',2); wait_for("return document.body.innerText.includes('QA isolated mission')",15)
   menu('비행 기록',7); wait_for("return document.body.innerText.includes('QA synthetic recording')",15)
   return 'isolated mission, recording, theme and first-run completion survive process restart'
  check('process restart data preservation',restart_data)
  if args.serial_recovery: check('serial automatic recovery and manual disconnect',serial_recovery)
finally:
 if sid:
  try: command('DELETE','')
  except Exception: pass
 if feed:
  feed.terminate(); feed.wait(timeout=5)
 for fixture in fixtures:
  if fixture.poll() is None: fixture.terminate(); fixture.wait(timeout=5)
 if vite:
  vite.terminate(); vite.wait(timeout=5)
 proc.terminate()
 try: proc.wait(timeout=5)
 except subprocess.TimeoutExpired: proc.kill();proc.wait()
 if pty_master is not None:
  pty_state['stop']=True;pty_thread.join(timeout=2);os.close(pty_master)
 shutil.rmtree(profile,ignore_errors=True)
 (out/'results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
print('Results:',out/'results.json')
sys.exit(any(x['status']=='FAIL' for x in results))
