# Safety / Obstacle / System Health

## 변경 파일

- `src/safety/model.ts`: HealthStatus/HealthSnapshot, SafetyEvent/Notification/Log, Obstacle/FailsafeState, reducer, summary, marker adapter.
- `src/safety/useSafety.ts`: 기존 ROS2/PX4 연결과 센서 상태를 Health로 변환하는 hook.
- `src/safety/SafetyUI.tsx`: Notification 닫기, Critical 외곽 경고와 배너, compact FAILSAFE.
- `src/App.tsx`: 기존 상태 source와 Safety UI/Console/Navigation을 연결.
- `src/components/DashboardPanels.tsx`: Health 요약/상세와 Failsafe 표시.
- `src/navigation/model.ts`, `NavigationUI.tsx`: 장애물 감지 시간, 경로 영향 상세, severity별 marker 항목 스타일.
- `src/styles.css`, `tests/safety.test.mjs`: 기존 레이아웃 유지, 분리된 테스트.

## 상태

NORMAL/WARNING/CRITICAL/DISCONNECTED/NO_DATA/ERROR를 지원합니다. 8개 장비 상태는 독립적입니다. 현재 연결 source가 없는 센서와 통신은 미연결, 위치/배터리는 데이터 없음입니다. 실제 ROS2/PX4 연결 source와 센서 store는 유지합니다.

Health summary는 Critical → Error → Warning → Disconnected → No Data → Normal 우선순위로 계산하며 문제 단계의 개수를 표시합니다. 이는 표시용 집계입니다. 미연결을 Critical로 변환하거나 flight command를 생성하지 않습니다.

센서 CONNECTED는 Health의 NORMAL로 표시합니다. 배터리 threshold/거리 threshold는 없습니다. PX4/ROS2 실제 수신 검증은 계속 보류 상태입니다.

## 미래 실제 adapter 계약

- `useSafety()`의 dispatch는 health/event/clear-event/obstacle/remove-obstacle/failsafe 상태 입력과 dismiss UI 동작을 제공합니다. 현재 외부 Safety 데이터 source는 연결하지 않았습니다.
- 기존 여섯 연결 항목은 기존 실제 connection/sensor source가 기준입니다. 향후 더 세밀한 Health는 해당 source adapter에서 통합해야 합니다. position/battery/link는 별도 실제 Health 입력을 받도록 준비했습니다.
- SafetyEvent는 고유 event ID, scope DEVICE 또는 FLIGHT, severity WARNING/CRITICAL, cause/device/time/direction을 받습니다. 같은 ID는 중복 처리하지 않습니다. 최근 1000 event ID를 기억합니다. 실제 source는 새 발생마다 새 ID를 제공해야 합니다.
- 화면 외곽 border와 작은 banner는 활성 FLIGHT + CRITICAL event에만 표시됩니다. DEVICE Critical, 장애물 Critical, 센서 DISCONNECTED, Target LOST로 자동 활성화하지 않습니다.
- clear-event는 실제 source가 위험 해제를 통보할 때 사용합니다. Notification dismiss는 상태를 해제하지 않습니다.
- Health warning/critical 전이와 obstacle 최초/등급 변경은 notification과 log를 생성합니다. 같은 상태 반복 수신이나 거리값만 변경된 obstacle은 중복 알림을 생성하지 않습니다.
- Notification은 관련 장비/원인/severity/time을 표시합니다. 최근 100개 보관, 미확인 최신 4개를 화면에 표시합니다. 현재 정상/미연결 기본 상태에서는 자동 notification을 만들지 않습니다.
- Console은 최근 200개 Safety log를 메모리에 보관하며 실제 전이/발생/해제만 기록합니다. 새로고침 시 영구 보관하지 않습니다.
- Obstacle은 실제 처리 결과만 입력합니다. raw LiDAR scan을 위험 판정이나 obstacle로 변환하지 않습니다. 위치가 null이면 지도 marker를 생성하지 않습니다.
- position 있는 obstacle만 기존 공통 MarkerDetail로 전달합니다. 상태/방향/거리/센서/최초·최근 감지/경로 영향과 UNKNOWN/NONE/BLOCKED/REPLANNING/NEW_PATH 표현을 준비했습니다. 실제 지도 렌더링은 연결하지 않았습니다.
- failsafe는 실제 source가 제공한 reason/status만 표시합니다. 복귀/착륙 등 자동 동작은 없습니다.

## 검증

- `npm run build`: 통과.
- Node 24 `node --experimental-strip-types --test tests/*.test.mjs`: 기존 Target/Navigation/Sensor 포함 22개 통과.
- 테스트 fixture는 tests/ 안에만 존재하며 production UI나 자동 simulation으로 연결되지 않습니다.
- 알림 닫기와 상태 해제 분리, 중복 polling, device Critical과 flight Critical 구분, 해제 및 recovery를 reducer 테스트로 검증했습니다.
- 브라우저: 기본 warning/critical/obstacle 0개, FAILSAFE 상태 없음, Health 펼침/접힘 확인.
- 1920x1080: pageHeight=1080 확인.
- 실제 warning/critical source가 없어 실환경 발생/복구와 경고 UI의 실제 데이터 기반 동작은 미검증입니다. UI에 테스트 버튼을 추가하지 않았습니다.
- ROS2/PX4 Bridge 파일을 수정하지 않았고 기존 Navigation/Target/Sensor/Flight UI를 유지했습니다.

## 실행

```bash
cd dashboard
npm run dev
```

http://127.0.0.1:5173/
