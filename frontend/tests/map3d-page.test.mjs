import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const html = readFileSync('public/vworld-3d.html', 'utf8');
const scripts = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map(m => m[1]);
function storage(initial = {}) { const values = new Map(Object.entries(initial)); return { getItem: key => values.get(key) || null, setItem: (key,value) => values.set(key,value), removeItem: key => values.delete(key) }; }

test('SDK loader requires HTTPS and does not modify the official initializer', () => {
  assert.equal(scripts.length, 2);
  let output = '';
  const run = (protocol, key) => { output=''; vm.runInNewContext(scripts[0], {location:{protocol},sessionStorage:storage({'firescene-vworld-key':key}),document:{write:s=>{output+=s}}}); return output; };
  assert.equal(run('tauri:', 'TEST-KEY'), '');
  assert.equal(run('https:', '<bad>'), '');
  const markup=run('https:', 'TEST-KEY');
  assert.match(markup, /^<script src="https:\/\/map\.vworld\.kr\/js\/webglMapInit\.js\.do\?version=3\.0&apiKey=TEST-KEY"><\/script>$/);
  assert.ok(!markup.includes('domain='));
});

test('3D page makes no success claim without SDK and uses supplied coordinates with a mock SDK', () => {
  function run(mockSdk) {
    const elements = Object.fromEntries(['status','setup','key','save','clear'].map(id=>[id,{textContent:'',hidden:false,value:''}]));
    const buttons = ['north','south','east','west','top','reset'].map(view=>({dataset:{view},addEventListener:(_,fn)=>{buttons.find(b=>b.dataset.view===view).click=fn}}));
    const calls=[];
    const context={ window:{addEventListener(){}}, location:{protocol:'https:',hash:'#lon=127.1&lat=37.2&radius=100',reload(){}}, sessionStorage:storage({'firescene-vworld-key':'TEST-KEY'}), URLSearchParams, document:{getElementById:id=>elements[id],querySelectorAll:()=>buttons} };
    if (mockSdk) {
      context.vw={Map:class {setOption(v){calls.push(['options',v])} start(){calls.push(['start'])}},CameraPosition:class {constructor(pos,dir){this.pos=pos;this.dir=dir}},CoordZ:class {constructor(lon,lat,z){Object.assign(this,{lon,lat,z})}},Direction:class {constructor(heading,pitch,roll){Object.assign(this,{heading,pitch,roll})}}};
      context.ws3d={viewer:{camera:{flyTo:arg=>calls.push(['flyTo',arg])}}};
      context.Cesium={Cartesian3:{fromDegrees:(...v)=>v},Math:{toRadians:n=>n*Math.PI/180}};
    }
    vm.runInNewContext(scripts[1],context); return {elements,buttons,calls};
  }
  assert.match(run(false).elements.status.textContent,/실패/);
  const result=run(true);
  assert.match(result.elements.status.textContent,/수신 여부는 확인 필요/);
  assert.equal(result.calls[0][1].initPosition.pos.lon,127.1);
  assert.equal(result.calls[0][1].initPosition.pos.lat,37.2);
  result.buttons.find(b=>b.dataset.view==='top').click();
  assert.equal(result.calls.at(-1)[0],'flyTo');
  assert.equal(result.calls.at(-1)[1].orientation.pitch,-Math.PI/2);
});
