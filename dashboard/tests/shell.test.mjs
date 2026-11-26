import test from 'node:test';
import assert from 'node:assert/strict';
import { createBrowserShell } from '../src/platform/stationShell.ts';
test('sensor shell reuses one named window and does not couple close to processing',()=>{
 const original=globalThis.window;let opened=0,focused=0;const popup={closed:false,focus:()=>focused++};
 globalThis.window={location:{href:'http://localhost:5173/',search:''},open:(url,name)=>{opened++;assert.equal(name,'uav-sensor-monitor');assert.equal(new URL(url).searchParams.get('view'),'sensors');return popup;}};
 try{const shell=createBrowserShell();assert.equal(shell.openSensorMonitor(),true);assert.equal(shell.openSensorMonitor(),true);assert.equal(opened,1);assert.equal(focused,1);popup.closed=true;assert.equal(shell.openSensorMonitor(),true);assert.equal(opened,2);}finally{if(original===undefined)delete globalThis.window;else globalThis.window=original;}
});
test('blocked popup returns false for UI guidance without throwing',()=>{const original=globalThis.window;globalThis.window={location:{href:'http://localhost:5173/',search:''},open:()=>null};try{assert.equal(createBrowserShell().openSensorMonitor(),false);}finally{if(original===undefined)delete globalThis.window;else globalThis.window=original;}});
