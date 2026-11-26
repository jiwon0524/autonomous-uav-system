import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';

test('slow startup offers retry without fabricating progress', () => {
 const status={textContent:''}, retry={hidden:true}; let timeout, reloaded=false;
 const elements={'launch-screen':{},'launch-status-text':status,'launch-retry':retry};
 const window={setTimeout(fn,ms){assert.equal(ms,8000);timeout=fn},location:{reload(){reloaded=true}}};
 vm.runInNewContext(readFileSync(new URL('../public/launch.js',import.meta.url),'utf8'),{window,document:{getElementById:id=>elements[id]}});
 timeout(); assert.equal(retry.hidden,false); assert.match(status.textContent,/startup is taking longer/); retry.onclick(); assert.equal(reloaded,true);
});
test('completed startup leaves the delayed callback harmless', () => {
 let timeout; const window={setTimeout(fn){timeout=fn}};
 vm.runInNewContext(readFileSync(new URL('../public/launch.js',import.meta.url),'utf8'),{window,document:{getElementById:()=>null}});
 assert.doesNotThrow(timeout);
});
