#!/usr/bin/env python3
"""Isolated real Tauri smoke QA via W3C WebDriver; no production dependencies."""
import argparse, base64, json, os, pathlib, shutil, socket, subprocess, sys, tempfile, time, urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
PARSER = argparse.ArgumentParser()
PARSER.add_argument('--driver', required=True, help='tauri-driver executable')
PARSER.add_argument('--native-driver', required=True, help='WebKitWebDriver executable')
PARSER.add_argument('--app', default=str(ROOT/'src-tauri/target/debug/firescene-gcs'))
PARSER.add_argument('--results', default='/tmp/firescene-desktop-qa')
PARSER.add_argument('--dev', action='store_true', help='start Vite and use debug simulation build')
PARSER.add_argument('--with-map-key', action='store_true', help='copy existing key into isolated private config')
args=PARSER.parse_args()
out=pathlib.Path(args.results).resolve(); out.mkdir(parents=True,exist_ok=True)
with socket.socket() as probe:
 probe.bind(('127.0.0.1',4444))  # refuse to attach to an unrelated existing driver
# Refuse any persistent or user-owned data directory as a test profile.
profile=pathlib.Path(tempfile.mkdtemp(prefix='firescene-qa-',dir='/tmp'))
env=os.environ.copy()
for k,v in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:
 p=profile/v; p.mkdir(mode=0o700); env[k]=str(p)
if args.with_map_key:
 source=pathlib.Path.home()/'.config/org.firescene.gcs/vworld-api-key'
 dest=profile/'config/org.firescene.gcs/vworld-api-key'; dest.parent.mkdir(mode=0o700,parents=True); shutil.copyfile(source,dest); dest.chmod(0o600)
vite=None; feed=None
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
  if name == 'simulation recording and replay': screenshot('recording-failure')
def click_text(label):
 ok=js("const e=[...document.querySelectorAll('button')].find(x=>x.textContent.trim()===arguments[0]); if(!e||e.disabled)return false; e.click();return true;",label)
 if not ok: raise AssertionError('button unavailable: '+label)
def wait_for(script,seconds=10):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  if js(script): return
  time.sleep(.2)
 raise AssertionError('condition timed out: '+script[:160])
def screenshot(name):
 # App-only WebDriver captures; system settings is excluded because it contains a key field.
 (out/(name+'.png')).write_bytes(base64.b64decode(command('GET','/screenshot')))
def measure_main():
 def descend(pid):
  try: children=[int(x) for x in pathlib.Path(f'/proc/{pid}/task/{pid}/children').read_text().split()]
  except (FileNotFoundError,PermissionError): return []
  return children+[c for child in children for c in descend(child)]
 pids=descend(proc.pid)
 app=next((p for p in pids if pathlib.Path(f'/proc/{p}/comm').read_text().strip()=='firescene-gcs'),None)
 if not app: raise RuntimeError('app process not found under test driver')
 def read():
  stat=pathlib.Path(f'/proc/{app}/stat').read_text().split(); status=pathlib.Path(f'/proc/{app}/status').read_text().splitlines()
  rss=next(x.split()[1] for x in status if x.startswith('VmRSS:'))
  return int(stat[13])+int(stat[14]),int(rss)
 before,_=read(); start=time.monotonic(); time.sleep(2); after,rss=read(); cpu=(after-before)/os.sysconf('SC_CLK_TCK')/(time.monotonic()-start)*100
 return f'main process only: {cpu:.1f}% one-core CPU, RSS {rss/1024:.1f} MiB; 2s sample'
def menu(name,index):
 ok=js("const b=[...document.querySelectorAll('nav[aria-label=\"주 메뉴\"] button')].find(x=>x.getAttribute('aria-label')===arguments[0]);if(!b)return false;b.click();return true;",name)
 if not ok: raise AssertionError('menu missing')
 wait_for("return document.querySelector('nav [aria-current=page]')?.getAttribute('aria-label')===arguments[0]".replace('arguments[0]',json.dumps(name,ensure_ascii=False)))
 time.sleep(.5)
 if name!='시스템 설정': screenshot(f'{index:02d}')
 return 'navigation active; screenshot saved' if name!='시스템 설정' else 'navigation active; screenshot skipped for key privacy'
try:
 time.sleep(.8)
 if proc.poll() is not None: raise RuntimeError('tauri-driver failed to start')
 start=time.monotonic()
 response=call('POST','/session',{'capabilities':{'alwaysMatch':{'browserName':'wry','tauri:options':{'application':str(pathlib.Path(args.app).resolve())}}}})
 sid=response['sessionId']; wait_for("return !!document.querySelector('nav[aria-label=\"주 메뉴\"]')",20)
 results.append({'name':'startup','status':'PASS','detail':f'{time.monotonic()-start:.2f}s to navigation'}); print('PASS startup',flush=True)
 check('idle main process sample',measure_main)
 names=['종합 관제','비행 제어','임무 계획','자율비행','3D 매핑','카메라','안전 관리','비행 기록','시스템 설정']
 for i,n in enumerate(names): check('menu '+n,lambda n=n,i=i:menu(n,i))
 def themes():
  click_text('Day'); wait_for("return !!document.querySelector('.theme-day')"); menu('종합 관제',0); screenshot('day'); click_text('Dark'); wait_for("return !!document.querySelector('.theme-dark')")
 check('theme switch',themes)
 def resize():
  command('POST','/window/rect',{'width':1100,'height':760}); r=command('GET','/window/rect'); assert r['width']>=850 and r['height']>=620
  command('POST','/window/maximize',{}); command('POST','/window/rect',{'width':1200,'height':800})
 check('window resize maximize restore',resize)
 def keyboard():
  menu('종합 관제',0); click_text('오프라인 입력 연습 ON'); wait_for("return document.body.innerText.includes('오프라인 입력 연습 · 기체 상태')")
  # DOM events exercise app event handlers, not physical keyboard or WM focus behavior.
  js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'w',code:'KeyW',bubbles:true}));")
  wait_for("return document.querySelector('.manual-keys .active')?.innerText.includes('W')")
  js("window.dispatchEvent(new KeyboardEvent('keyup',{key:'w',code:'KeyW',bubbles:true}));")
  wait_for("return !document.querySelector('.manual-keys .active')")
  js("window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true}));")
  wait_for("return !document.body.innerText.includes('오프라인 입력 연습 · 기체 상태')")
  return 'synthetic DOM key events: W down/up, Escape; physical focus not tested'
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
  return str(count)+' results; selected'
 check('live map search',map_search)
 def layers():
  assert js("return document.querySelector('.map-toolbar button[disabled]')?.textContent.includes('3D')")
  click_text('위성'); wait_for("return document.querySelector('.map-toolbar button[aria-pressed=true]')?.innerText==='위성'"); click_text('기본'); wait_for("return document.querySelector('.map-toolbar button[aria-pressed=true]')?.innerText==='기본'")
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
 def draft_mission():
  menu('임무 계획',2); wait_for("return [...document.querySelectorAll('button')].some(e=>e.textContent.trim()==='새 임무'&&!e.disabled)")
  click_text('새 임무'); wait_for("return !!document.querySelector('.mission-fields input')")
  element=command('POST','/element',{'using':'css selector','value':'.mission-fields input'}); eid=element['element-6066-11e4-a52e-4f735466cecf']
  command('POST','/element/'+eid+'/clear',{}); command('POST','/element/'+eid+'/value',{'text':'QA isolated mission'})
  click_text('작성 중 저장'); wait_for("return document.body.innerText.includes('작성 중 임무를 저장했습니다')")
  click_text('복제'); wait_for("return document.body.innerText.includes('복제본을 작성 중 상태로 저장했습니다')")
  return 'created and duplicated isolated draft'
 check('isolated mission draft and duplicate',draft_mission)
 if args.dev:
  def sim_connect():
   global feed
   menu('시스템 설정',8); wait_for("return document.body.innerText.includes('시뮬레이션 수신 시작')")
   click_text('시뮬레이션 수신 시작')
   feed=subprocess.Popen([str(ROOT/'src-tauri/target/debug/examples/telemetry_feed'),'battery-low'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
   wait_for("return document.body.innerText.includes('42 / 1')",15)
   return 'development feed system 42, simulation receiver'
  check('simulation heartbeat receive',sim_connect)
  check('simulation main process sample',measure_main)
  def recording():
   menu('비행 기록',7); wait_for("return !!document.querySelector('[aria-label=\"기록 이름\"]')")
   e=command('POST','/element',{'using':'css selector','value':'[aria-label="기록 이름"]'}); eid=e['element-6066-11e4-a52e-4f735466cecf']; command('POST','/element/'+eid+'/value',{'text':'QA synthetic recording'})
   click_text('기록 시작'); wait_for("return document.body.innerText.includes('기록 중 ·')",10); wait_for("return document.body.innerText.includes('배터리 부족')",15)
   click_text('기록 종료'); wait_for("return document.querySelector('.recordings-page .source-label')?.innerText.includes('기록 중 아님')",12); wait_for("return [...document.querySelectorAll('.recording-list button')].some(e=>e.textContent.includes('QA synthetic recording'))",12)
   js("[...document.querySelectorAll('.recording-list button')].find(e=>e.textContent.includes('QA synthetic recording')).click()")
   wait_for("return document.body.innerText.includes('REPLAY · 과거 기록')",10)
   wait_for("return [...document.querySelectorAll('.replay-controls button')].some(e=>e.textContent.trim()==='재생'&&!e.disabled)",12); click_text('재생'); wait_for("return document.body.innerText.includes('일시정지')"); click_text('일시정지')
   js("const e=document.querySelector('[aria-label=\"재생 속도\"]');e.value='2';e.dispatchEvent(new Event('change',{bubbles:true}))")
   click_text('JSONL 내보내기'); wait_for("return document.body.innerText.includes('내보내기 완료')")
   return 'low battery warning; synthetic record saved, replayed, paused and exported to isolated data' 
  check('simulation recording and replay',recording)
finally:
 if sid:
  try: command('DELETE','')
  except Exception: pass
 if feed:
  feed.terminate(); feed.wait(timeout=5)
 if vite:
  vite.terminate(); vite.wait(timeout=5)
 proc.terminate()
 try: proc.wait(timeout=5)
 except subprocess.TimeoutExpired: proc.kill();proc.wait()
 shutil.rmtree(profile,ignore_errors=True)
 (out/'results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
print('Results:',out/'results.json')
sys.exit(any(x['status']=='FAIL' for x in results))
