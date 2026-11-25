import {strict as assert} from 'node:assert';
import {withConnectionDeadline} from '../src/connectionDeadline.ts';
assert.equal(await withConnectionDeadline(Promise.resolve('CONNECTED'),100),'CONNECTED');
await assert.rejects(withConnectionDeadline(new Promise(()=>{}),10),/시간 초과/);
await assert.rejects(withConnectionDeadline(Promise.reject(new Error('disconnected')),100),/disconnected/);
