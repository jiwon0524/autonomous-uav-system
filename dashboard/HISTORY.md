# Mission History / Flight Record

## 파일

- `src/history/model.ts`: MissionRecord/Type/Status/Event/TargetRecord/FlightPathPoint/Metrics/Filter와 미래 MissionRepository API interface. 기록별 note update와 timeline ID 중복 방지/시간 정렬.
- `src/history/MissionHistory.tsx`: 내부 비행 기록 진입, 빈 상태, 조건부 목록/필터, 상세 정보, timeline, 기록 지도 영역, Target 기록, NOTE 편집.
- `src/App.tsx`: 비행 기록 버튼과 내부 화면 전환, 실제 기록을 받을 빈 session state.
- `src/styles.css`: 기존 Light Theme 스타일 내 목록/상세 화면.
- `tests/history.test.mjs`: 기록 자동 생성 방지, note 편집, timeline 및 필터 테스트.

## 현재 상태

기록은 빈 배열입니다. 기본 화면은 `저장된 비행 기록이 없습니다.`이며 가짜 Mission/Event/경로/metric을 생성하지 않습니다. Console/Flight Control/Safety 이벤트를 History로 복사하거나 Mission으로 자동 저장하지 않습니다.

목록 데이터가 들어오면 날짜/임무 유형/상태/Target 이벤트/경고 필터와 내부 scroll을 표시합니다. 데이터가 없으면 필터는 숨깁니다.

Mission 상세는 null 값을 `--`로 표시합니다. 시간/경로/목표/감지/경고 값은 실제 source 값만 받으며 임의 계산하지 않습니다. PATROL은 예정 기능을 유지하며 기록 UI가 실제 자동순찰을 활성화하지 않습니다.

NOTE 저장/수정은 App session state만 갱신합니다. 저장 문구는 세션 저장임을 명시합니다. 기록 화면에서 나갔다 돌아와도 App이 유지되면 note는 유지되지만 새로고침하면 사라집니다. 없는 Mission ID에 note를 저장해도 새 기록을 만들지 않습니다.

향후 MissionRepository list/get/saveNotes와 실제 backend adapter를 연결할 수 있습니다. 현재 repository 구현, DB/HTTP endpoint, 실제 자동 저장은 없습니다. 실제 Mission Manager가 lifecycle과 기록을 제공할 때 별도로 연결해야 합니다.

Event 타입은 SYSTEM/PX4/ROS2/FLIGHT/DETECT/TARGET/TRACK/SENSOR/OBSTACLE/SAFETY/NAVIGATION입니다. severity/metadata를 저장할 수 있으며 SENSOR/SAFETY 등 유형별 timeline 필터가 있습니다. Console과 독립된 데이터이며 자동 이벤트 생성 로직은 없습니다.

Mission flightPath에 실제 timestamp/position/altitude를 받고 mapMarkers에 HOME/SAFE_POINT/TARGET/OBSTACLE/LAST_SEEN을 보관할 수 있습니다. 현재 지도는 데이터 없음 영역이며 실제 API나 경로 렌더러를 구현하지 않습니다.

성능 지표는 optional nullable 구조만 준비하며 detection count/duration/lost/reacquisition/replanning 등을 계산하지 않습니다.

## 검증

- `npm run build`: 통과.
- Node 24 `node --experimental-strip-types --test tests/*.test.mjs`: 기존 포함 26개 통과.
- 브라우저에서 내부 비행 기록 진입, 빈 상태 문구, 비행 관제 복귀 확인.
- 실제 Mission 데이터가 없으므로 기록 기반 상세/필터/NOTE UI의 실환경 동작은 미검증입니다. note 갱신/필터/timeline 로직은 테스트 fixture로 검증했으며 fixture를 앱에 연결하지 않았습니다.
- 기존 Bridge, 센서, Safety, Target, Navigation 기능과 실제 ROS2/PX4 연결 검증 보류 상태를 유지합니다.

## 실행

```bash
cd dashboard
npm run dev
```

http://127.0.0.1:5173/
