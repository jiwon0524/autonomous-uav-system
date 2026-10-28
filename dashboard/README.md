# UAV 자율비행 관제 시스템 — Windows 안정화 기준점

React + TypeScript + Vite 기반 UI입니다. UI와 향후 실제 데이터 연결 계약을 구현했으며, 실제 시스템 검증은 Ubuntu 단계에서 진행합니다. 새로운 기능이나 Desktop packaging은 이번 안정화에서 추가하지 않았습니다.

## 실행과 검사

Node.js 24 환경을 기준으로 확인했습니다. 프로젝트 루트에서:

```bash
cd dashboard
npm ci
npm run dev
```

http://127.0.0.1:5173/

```bash
npm run build
./node_modules/.bin/tsc --noEmit
node --experimental-strip-types --test tests/*.test.mjs
```

Windows PowerShell에서는 TypeScript 검사 명령을 `./node_modules/.bin/tsc.cmd --noEmit`으로 실행합니다. 기존 script는 dev/build/preview뿐입니다. lint script와 ESLint는 현재 없으며 설치하지 않았습니다. TypeScript strict + noUnusedLocals + noUnusedParameters 검사를 사용합니다.

## 구현된 범위

- 한글 Light Theme 3열 UI, 기존 카메라 크기와 typography 유지
- 비행 제어/복귀/비상 확인 UI, Position Control, HOME/여러 Safe Point 구조
- Target Management, Navigation/Path/Obstacle/Last Seen 타입과 빈 상태
- 단일 Sensor Monitor 진입, RGB/Depth/LiDAR 독립 상태, 확대·복귀 및 상세 UI
- System Health, Safety 알림/경계선 구조, FAILSAFE, System Console
- 비행 기록 빈 상태/목록/상세/Timeline/NOTE session state와 repository 계약
- 읽기 전용 ROS2/PX4 Bridge 코드와 상태 polling

실제 telemetry, 센서, 위치, 지도, Mission 저장은 연결하지 않았습니다. 실제 데이터 없는 항목은 `--`, 데이터 없음, 연결 안 됨, 대기, 예정으로 표시합니다. 이동 단위 0.5/1/2m는 입력 옵션이고, 00:00은 미시작 타이머 표시이며, UAV-01은 UI 식별 문구입니다. 테스트 fixture는 tests/에만 존재하며 앱에 import하지 않습니다.

## 계층 경계

- React UI → `services/bridgeStatus.ts` → 상대 경로 GET `/api/status` → Vite proxy → Dashboard Bridge.
- 원격 UAV/Bridge는 `DASHBOARD_BRIDGE_URL`로 지정할 수 있습니다. UI는 ROS2 Python이나 특정 센서 SDK를 직접 호출하지 않습니다.
- `platform/stationShell.ts`는 Browser window 처리를 캡슐화합니다. Sensor Monitor window lifecycle은 sensor processing을 제어하지 않습니다.
- 센서 동기화 메시지는 `sensors/validation.ts`에서 검증하며 잘못된 payload는 무시합니다. HMR 종료 때 UI BroadcastChannel을 정리합니다.
- 실제 비행 command client/publisher는 없습니다. Flight Control은 안내/확인 UI만 제공합니다.
- Target Lost/센서 미연결은 Flight Critical이 아닙니다. 명시적인 활성 FLIGHT+CRITICAL 이벤트만 외곽 경고를 켭니다.

## 실제 검증 보류 / 다음 개발 환경

Ubuntu 24.04 + ROS2 Jazzy + PX4 SITL + Gazebo + Micro XRCE-DDS에서 다음을 별도로 검증해야 합니다.

1. 기존 px4_msgs overlay와 firmware 메시지 버전, DDS domain/QoS 일치
2. `/fmu/out/vehicle_status_v4` 실제 수신, stream timeout, Bridge 종료·복구
3. 원격 PC에서 Bridge 접근과 배포 시 `/api` reverse proxy
4. telemetry, flight command, RGB/YOLO/Depth/LiDAR, 실제 지도, 실제 Mission 저장 (현재 미구현/미검증)

Windows에서는 ROS2/PX4를 실행하지 않았습니다. 기존 Bridge를 가짜 서버로 교체하지 않았습니다. 실행/검증 절차는 [Bridge README](../dashboard_bridge/README.md)를 참고하세요.

세부 구조: [Target](TARGET_MANAGEMENT.md), [Navigation](NAVIGATION.md), [Sensor](SENSORS.md), [Safety](SAFETY.md), [History](HISTORY.md), [안정화 결과](STABILIZATION.md).

## Git

현재 제공된 폴더는 Git 저장소가 아닙니다. commit/push/init은 하지 않았습니다. root/dashboard `.gitignore`는 node_modules, build/dist, Python cache, logs, 임시 work/outputs를 제외합니다. 실제 저장소로 옮길 때 `.gitignore`도 함께 반영하세요. 이미 추적된 파일은 ignore만으로 해제되지 않으므로 사용자가 staging 전 확인해야 합니다. package-lock.json과 필요한 설정·문서는 보존합니다. `.env`에 연결 주소 외에 비밀정보를 넣지 마세요.

디자인 참고: [GCS Dashboard Design — 정은우](https://www.figma.com/community/file/1695333621629166803/gcs-dashboard-design), CC BY 4.0.
