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
