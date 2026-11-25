import {strict as assert} from 'node:assert';
import {priorTarget} from '../src/priorMapSelection.ts';
assert.deepEqual(priorTarget({center:[127.1,37.2],radius:100}),{lon:127.1,lat:37.2,radius:100});
for(const item of [null,{center:[127],radius:1},{center:[181,37],radius:100},{center:[127,86],radius:100},{center:[127,37],radius:0},{center:[127,37],radius:2001},{center:[NaN,37],radius:100}])assert.equal(priorTarget(item),null);
