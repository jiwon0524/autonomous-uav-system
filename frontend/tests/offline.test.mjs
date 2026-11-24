import { test } from 'node:test';
import assert from 'node:assert/strict';
import { build } from 'esbuild';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';

test('initial UI shows disconnected state and all navigation destinations', async () => {
  const dir = await mkdtemp('node_modules/.firescene-test-');
  try {
    const result = await build({ entryPoints: ['src/App.tsx'], bundle: true, write: false, platform: 'node', format: 'esm', external: ['react', 'react/jsx-runtime'] });
    const path = `${dir}/app.mjs`;
    await writeFile(path, result.outputFiles[0].text);
    const { default: App } = await import(pathToFileURL(`${process.cwd()}/${path}`));
    const html = renderToStaticMarkup(React.createElement(App));
    for (const label of ['종합 관제', '비행 제어', '임무 계획', '자율비행', '3D 매핑', '카메라', '안전 관리', '비행 기록', '시스템 설정', '기체 미연결', '텔레메트리 미연결', '미수신', '확인 불가', '비행 명령 잠김']) assert.ok(html.includes(label), label);
    for (const sample of ['18.4', '76%', 'F450-01', '대전 테스트 현장', '실시간 운용 샘플']) assert.ok(!html.includes(sample), sample);
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('connection panel shows test heartbeat and marks lost data as previous', async () => {
  const dir = await mkdtemp('node_modules/.firescene-test-');
  try {
    const result = await build({ entryPoints: ['src/Connection.tsx'], bundle: true, write: false, platform: 'node', format: 'esm', external: ['react', 'react/jsx-runtime'] });
    const path = `${dir}/connection.mjs`; await writeFile(path, result.outputFiles[0].text);
    const { ConnectionSettings } = await import(pathToFileURL(`${process.cwd()}/${path}`));
    const sample = { phase: 'CONNECTED', source: 'SIMULATION', port: 'TEST-PORT', error: null, last_heartbeat_ms: 1700000000000, vehicle: { system_id: 42, component_id: 1, vehicle_type: 2, autopilot_type: 12, mavlink_version: 1, heartbeat_ms: 1700000000000 } };
    const connected = renderToStaticMarkup(React.createElement(ConnectionSettings, { control: { connection: sample, ports: [], auto: null, last: null, scan: '', retry: '', error: '' } }));
    assert.ok(connected.includes('42 / 1')); assert.ok(connected.includes('PX4 HEARTBEAT 수신')); assert.ok(connected.includes('송신자 인증')); assert.ok(connected.includes('SIMULATION')); assert.ok(connected.includes('출처 미검증'));
    const lost = renderToStaticMarkup(React.createElement(ConnectionSettings, { control: { connection: { ...sample, phase: 'CONNECTION_LOST', vehicle: null }, ports: [], auto: null, last: null, scan: '', retry: '', error: '' } }));
    assert.ok(lost.includes('연결 끊김')); assert.ok(lost.includes('이전 수신')); assert.ok(!lost.includes('42 / 1'));
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('automatic selection requires one explicitly identified device and retries are bounded', async () => {
  const dir = await mkdtemp('node_modules/.firescene-test-');
  try {
    const result = await build({ entryPoints: ['src/auto.ts'], bundle: true, write: false, platform: 'node', format: 'esm' });
    const path = `${dir}/auto.mjs`; await writeFile(path, result.outputFiles[0].text);
    const { candidate, retryDelay } = await import(pathToFileURL(`${process.cwd()}/${path}`));
    const allowed = { vid: 1, pid: 2, serial: 'TEST-ONLY', baud: 115200 };
    const port = { name: 'test-device', vid: 1, pid: 2, serial_number: 'TEST-ONLY' };
    assert.equal(candidate([], allowed).port, null);
    assert.equal(candidate([{...port, serial_number: 'OTHER'}], allowed).port, null);
    assert.equal(candidate([port], allowed).port.name, 'test-device');
    assert.equal(candidate([port, {...port, name: 'duplicate'}], allowed).ambiguous, true);
    assert.equal(candidate([], allowed).port, null);
    assert.equal(candidate([port], allowed).port.name, 'test-device');
    assert.deepEqual([0,1,2,3].map(retryDelay), [5000,10000,20000,null]);
  } finally { await rm(dir, { recursive: true, force: true }); }
});
