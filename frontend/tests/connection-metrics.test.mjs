import test from 'node:test';
import assert from 'node:assert/strict';
import {recentFrameRate,heartbeatInterval} from '../src/connectionMetrics.ts';
test('rate displays only fresh active measurements, including a real zero',()=>{
 assert.equal(recentFrameRate(0,1000,2000,true),'0.0 프레임/초');
 assert.equal(recentFrameRate(2.26,1000,2000,true),'2.3 프레임/초');
 for (const a of [[2,1000,5001,true],[2,1000,999,true],[2,1000,2000,false],[NaN,1000,2000,true]]) assert.equal(recentFrameRate(...a),'확인 불가');
 assert.equal(heartbeatInterval(1020,true),'1.02초'); assert.equal(heartbeatInterval(null,true),'확인 불가'); assert.equal(heartbeatInterval(1000,false),'확인 불가');
});
