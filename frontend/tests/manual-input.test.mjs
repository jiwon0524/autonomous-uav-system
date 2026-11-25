import {strict as assert} from 'node:assert';
import {axes, HOLD_TIMEOUT_MS, initialPreview, KEY_BINDINGS, previewReducer} from '../src/manualInput.ts';
let s=initialPreview();
s=previewReducer(s,{type:'enter',ready:false});assert.equal(s.mode,'MONITOR');
s=previewReducer(s,{type:'enter',ready:true});assert.equal(s.mode,'MANUAL_PREVIEW');
assert.deepEqual(KEY_BINDINGS,{KeyW:'forward',KeyS:'back',KeyA:'left',KeyD:'right',ArrowUp:'up',ArrowDown:'down',ArrowLeft:'yawLeft',ArrowRight:'yawRight'});
s=previewReducer(s,{type:'down',action:'forward',at:10,repeat:false});s=previewReducer(s,{type:'down',action:'left',at:20,repeat:false});s=previewReducer(s,{type:'down',action:'yawRight',at:30,repeat:false});assert.deepEqual(axes(s.held),{forward:1,right:-1,up:0,yaw:1});
s=previewReducer(s,{type:'down',action:'back',at:40,repeat:false});assert.equal(axes(s.held).forward,0);
s=previewReducer(s,{type:'up',action:'back'});assert.equal(axes(s.held).forward,1);
s=previewReducer(s,{type:'clear'});assert.deepEqual(axes(s.held),{forward:0,right:0,up:0,yaw:0});
s=previewReducer(s,{type:'down',action:'forward',at:50,repeat:false});assert.equal(axes(s.held).forward,0);
s=previewReducer(s,{type:'up',action:'forward'});s=previewReducer(s,{type:'down',action:'forward',at:60,repeat:true});assert.equal(axes(s.held).forward,0);
s=previewReducer(s,{type:'down',action:'forward',at:60,repeat:false});assert.equal(axes(s.held).forward,1);
s=previewReducer(s,{type:'tick',at:60+HOLD_TIMEOUT_MS,ready:true});assert.equal(s.mode,'MONITOR');assert.deepEqual(s.held,{});
s=previewReducer(s,{type:'tick',at:5000,ready:true});assert.equal(s.mode,'MONITOR');
s=previewReducer(s,{type:'enter',ready:true});s=previewReducer(s,{type:'down',action:'up',at:6000,repeat:false});s=previewReducer(s,{type:'tick',at:6050,ready:false});assert.equal(s.mode,'MONITOR');assert.deepEqual(s.held,{});
for(const reason of ['blur','hidden','menu','escape','off','shutdown','editable']) {s=previewReducer(s,{type:'enter',ready:true});s=previewReducer(s,{type:'down',action:'up',at:1,repeat:false});s=previewReducer(s,{type:'exit',reason});assert.equal(s.mode,'MONITOR');assert.deepEqual(s.held,{});}

for(const mode of ['MANUAL_CONTROL','AUTONOMOUS']) { const future={...initialPreview(),mode}; assert.equal(previewReducer(future,{type:'enter',ready:true}).mode,mode); }
