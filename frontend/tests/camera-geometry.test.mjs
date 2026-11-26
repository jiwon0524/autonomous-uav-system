import { strict as assert } from 'node:assert';
import { pixelBox } from '../src/cameraGeometry.ts';
assert.deepEqual(pixelBox([0.25,0.125,0.5,0.5],640,480),{x:160,y:60,width:320,height:240});
assert.deepEqual(pixelBox([0,0,1,1],1920,1080),{x:0,y:0,width:1920,height:1080});
