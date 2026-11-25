import {strict as assert} from 'node:assert';
import {nedToEnu,rotateUnitQuaternion} from '../src/spatialMath.ts';
assert.deepEqual(nedToEnu([1,2,3]),[2,1,-3]);
const r=rotateUnitQuaternion([1,0,0],[0,0,Math.SQRT1_2,Math.SQRT1_2]);
assert.ok(Math.abs(r[0])<1e-12 && Math.abs(r[1]-1)<1e-12 && Math.abs(r[2])<1e-12);
assert.throws(()=>rotateUnitQuaternion([1,0,0],[0,0,0,0]));
