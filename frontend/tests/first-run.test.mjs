import {test} from 'node:test';
import assert from 'node:assert/strict';
import {initialTheme, needsFirstRun, saveTheme, finishFirstRun} from '../src/setupPreferences.ts';
function memoryStore() { const m=new Map(); return {getItem:k=>m.get(k)??null,setItem:(k,v)=>m.set(k,String(v))}; }
test('setup remains optional and remembers completion and theme without touching mission data',()=>{
 globalThis.localStorage=memoryStore(); localStorage.setItem('unrelated-mission','keep');
 assert.equal(needsFirstRun(),true); assert.equal(initialTheme(),'dark');
 assert.equal(finishFirstRun('en','day'),true); assert.equal(needsFirstRun(),false); assert.equal(initialTheme(),'day');
 saveTheme('dark');assert.equal(initialTheme(),'dark');assert.equal(localStorage.getItem('unrelated-mission'),'keep');
});
test('unavailable storage falls back without preventing the app from opening',()=>{
 globalThis.localStorage={getItem(){throw Error('unavailable');},setItem(){throw Error('unavailable');}};
 assert.equal(needsFirstRun(),true);assert.equal(initialTheme(),'dark');assert.equal(finishFirstRun('ko','dark'),false);assert.doesNotThrow(()=>saveTheme('day'));
});
