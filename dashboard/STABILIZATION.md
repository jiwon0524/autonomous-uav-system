# 안정화 점검 결과

## 1. 발견한 문제

- House import 및 이전 ObstacleWarning component/CSS가 미사용 상태.
- Panel이 DashboardPanels에 있어 센서/타겟/지도와 순환 import 발생.
- 여러 컴포넌트에 동일 날짜 포맷 helper가 반복됨.
- 센서 창 BroadcastChannel 입력 검증 부족, HMR 종료 때 채널 cleanup 없음, 동일 millisecond 상태 전이에 event ID 충돌 가능.
- Console sensor event key를 배열 index로 생성하고 Safety key를 숫자 offset으로 만들어 장기 운용 시 식별 불안정 가능.
- 연결 hook의 React state updater 내부 sequence 변경으로 StrictMode에서 부수효과 가능.
- Target/Safety dictionary가 외부 ID를 Object prototype 이름과 혼동할 가능성.
- 미사용 검사 옵션이 꺼져 있고 root Git ignore가 없어 Python cache/임시 산출물이 포함될 가능성.

## 2. 수정한 문제

- 공통 Panel/Badge를 작은 독립 파일로 이동하여 runtime import cycle 해소.
- read-only Bridge fetch/response validation을 services/bridgeStatus.ts로 분리. UI/ROS2 경계 유지.
- 날짜 포맷 helper 통합, invalid timestamp는 -- 처리.
- 센서 메시지 전체 snapshot/event 검증과 숫자 finite/field 검증 추가. HMR dispose 때 channel close. 지원 불가/닫힌 channel 예외가 UI를 깨뜨리지 않게 처리.
- 센서 event ID에 local sequence 추가. Console key에 source namespace 사용. 연결 이벤트 sequence는 state updater 밖에서 생성.
- arbitrary ID 처리를 own-property 기준으로 보정하고 prototype 변경 가능성 제거.
- Navigation SafePoint draft의 불필요한 type cast 및 DestinationControls 중복 type 제거.
- TypeScript noUnusedLocals/noUnusedParameters 적용.
- gitignore 및 현재 상태 문서 갱신.

## 3. 변경 파일

App.tsx, useConnectionStatus.ts, tsconfig.json, components/DashboardPanels.tsx, components/Interactions.tsx, targets/model.ts, targets/TargetManagement.tsx, navigation/model.ts, navigation/NavigationUI.tsx, sensors/store.ts, sensors/SensorMonitor.tsx, safety/model.ts, safety/SafetyUI.tsx, history/MissionHistory.tsx, styles.css, README.md, root/dashboard .gitignore.

정리용 추가 파일: components/Panel.tsx, utils/format.ts, services/bridgeStatus.ts, sensors/validation.ts, tests/stabilization.test.mjs, tests/shell.test.mjs, STABILIZATION.md. 새 사용자 기능이나 dependency는 추가하지 않았습니다.

## 4. 제거한 중복/불필요 코드

미사용 House import, 대체된 ObstacleWarning 컴포넌트, 구형 health-state/health-icon/health-toggle와 collision-warning/risk-indicator CSS, 중복 날짜 helper, 도달 불가능한 SAFE POINT 안내 ternary, 중복 optional type. 미래 adapter/model interface는 유지했습니다.

## 5. Build 결과

npm run build 통과 (TypeScript + Vite). package.json의 기존 script만 사용했습니다. dependency 추가/변경 없음.

## 6. Lint 결과

lint script/ESLint가 없어서 미실행. 새 lint 도구를 설치하거나 존재하지 않는 script를 실행하지 않았습니다. rg 검사에서 production src의 명시적 any, console.log/debug, setInterval fake loop, browser persistence, flight command publisher는 발견되지 않았습니다.

## 7. TypeScript/테스트 결과

로컬 설치된 tsc --noEmit 통과. strict + noUnusedLocals + noUnusedParameters. 기존 테스트와 회귀 테스트 포함 Node tests 33개 통과. Bridge Python unit/HTTP tests 4개 및 py_compile 통과. 실제 rclpy/PX4 runtime은 실행하지 않았습니다.

초기 작업 폴더 루트의 npx 실행은 로컬 tsc를 못 찾아 registry 조회가 실패했습니다. 이후 dashboard/node_modules/.bin/tsc.cmd를 직접 사용해 통과했으며 설치나 dependency 변경은 없었습니다.

## 8. Runtime 결과

- 메인 렌더링, 비행 기록 빈 상태 진입/복귀 통과.
- System Health/Console 펼침·접기 통과.
- Position Control/Navigation 전환, 전진 안내, Emergency confirmation 통과.
- 센서 모니터 열기 버튼 호출 확인. 내장 브라우저 도구에 별도 popup이 노출되지 않아 실제 OS 창 생성/포커스는 미검증. 이름 고정 창 재사용/blocked popup 처리는 테스트 2개로 확인.
- 진단 경로를 검증용 탭으로 직접 열어 RGB/Depth/2D LiDAR 3분할·확대·복귀 확인 후 검증용 탭을 닫음.
- 1920x1080: 3 columns, camera 580px, document height 1080px, 가짜 Target/Obstacle/Critical 0개.
- typography/레이아웃/주요 버튼 디자인은 변경하지 않음.
- Bridge 없는 환경의 /api/status 연결 실패는 예상되는 미연결 상태이며 가짜 connected로 대체하지 않음.

## 9. Ubuntu에서 검증할 항목

실제 ROS2/PX4 VehicleStatus 수신/timeout/재연결, QoS와 px4_msgs firmware 호환성, 원격 Bridge 접근/배포 proxy. Telemetry/실제 명령/센서/YOLO/지도/Mission 저장은 아직 구현/연동 전이므로 별도 단계에서 검증해야 합니다. 기존 Bridge topic 및 코드를 변경하거나 가짜 대체 구현을 만들지 않았습니다.

## 10. Git 전 주의

현재 폴더는 .git이 없는 독립 작업 폴더여서 git diff/status 비교가 불가합니다. commit/push/init 없음. 실제 autonomous-uav-system 저장소 루트로 dashboard 및 dashboard_bridge를 반영할 때 root .gitignore도 검토하세요. node_modules/dist/build/work/outputs/logs/cache/비밀 env는 staging하지 않고 package-lock 및 필요한 설정/문서는 보존하세요. ignore는 이미 tracked인 파일을 자동 제외하지 않습니다.

남은 의도적 제한: 실제 데이터 기반 Target 상세/History NOTE/알림·Critical 시각 상태와 sensor live cross-window update는 실제 source가 없어 실환경 미검증. component 타입과 순수 reducer 회귀 테스트는 유지합니다. frontend는 raw scan/거리/배터리로 위험도를 계산하지 않습니다.
