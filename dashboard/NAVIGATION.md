# Navigation / Safe Point UI

## 추가/변경

- `src/navigation/model.ts`: SIM_MAP/REAL_MAP, MapMarker, SafePoint, NavigationPath, ObstacleInfo, MapState와 reducer.
- `src/navigation/NavigationUI.tsx`: Navigation/Position Control 공통 LocalMap, MarkerDetail, 목적지 확인, 복귀 위치 선택, Safe Point 설정 단계 UI.
- `src/App.tsx`: 하나의 NavigationData/MapState를 왼쪽 복귀 영역과 오른쪽 두 모드가 공유합니다.
- `DashboardPanels.tsx`, `Interactions.tsx`, `models.ts`, `styles.css`: 기존 화면 구조를 유지하며 공유 상태로 연결합니다.
- `tests/navigation.test.mjs`: UI state 및 ID 기반 관리 단위 테스트.

## 현재 동작

지도는 데이터 없음 상태입니다. Safe Point 설정 시 `현재 위치 데이터가 없어 SAFE POINT를 설정할 수 없습니다.` 안내를 표시합니다. 안전 복귀는 목적지 선택창을 열고, HOME과 Safe Point가 없으면 `사용 가능한 복귀 지점이 없습니다.`를 표시하며 실행 버튼을 비활성화합니다.

위치 제어 / 지도 보기 전환은 페이지 reload 없이 같은 상태를 유지합니다. 기본 데이터는 UAV/HOME null, 빈 Safe Point/marker/path 목록입니다. 가짜 위치나 경로를 만들지 않습니다.

## 향후 adapter 계약

- NavigationData는 실제 외부 데이터만 받습니다. 현재 외부 연결은 없으며 sourceReady=false입니다.
- MapPosition은 LOCAL_NED 또는 WGS84를 명시합니다. 서로 다른 좌표계를 임의 변환하지 않습니다. altitude/distance는 없으면 null입니다.
- Marker 타입: UAV/HOME/SAFE_POINT/DESTINATION/TARGET/LAST_SEEN/OBSTACLE. 각 source가 고유 ID를 제공해야 합니다.
- NavigationPath는 PLANNED/ACTUAL/REPLANNED와 실제 position 배열을 받습니다. 현재 렌더러/지도 API/경로선은 없습니다.
- 실제 지도 adapter는 `select-destination`에 실제 좌표를 전달합니다. 이후 `confirm-destination`과 사용자 확인을 거칩니다. 현재 이동 버튼은 안내만 표시하며 명령 client는 없습니다.
- Safe Point 단계는 begin-safe-point → choose-safe-position → name-safe-point → confirm-safe-point입니다. 확인만으로 ID, createdAt, 좌표를 만들어 저장하지 않습니다. 실제 저장 adapter는 후속 단계에서 연결합니다.
- safePointReducer는 여러 지점의 고유 ID upsert/이름 변경/삭제 상태 갱신 계약을 제공합니다. 현재 실제 데이터가 없어 편집 버튼을 노출하지 않습니다.
- 선택한 marker ID는 LocalMap에 전달되며 공통 상세 dialog를 엽니다. 향후 map renderer가 같은 ID로 위치를 강조할 수 있습니다.
- targetLastSeenMarker는 실제 lastSeenPosition이 있고 LOST/REACQUIRING일 때만 LAST_SEEN 모델을 반환합니다. 현재 앱에는 이 adapter를 연결하거나 marker를 자동 생성하지 않습니다.
- 장애물 위험도는 공통 상세의 상태 색상으로만 준비했습니다. 장애물 탐지만으로 전체 화면 경고를 켜지 않습니다. 실제 collision-risk source는 연결하지 않습니다.

## 검증

- `npm run build`: 통과.
- Node 24 `node --experimental-strip-types --test tests/*.test.mjs`: Target 포함 10개 통과. 테스트 fixture는 앱에 포함되지 않습니다.
- 브라우저: Safe Point 미설정 안내, 복귀 목적지 없음 및 실행 비활성화, 위치 제어 왕복 전환 확인.
- 1920x1080: 전체 페이지 높이 1080, 가짜 marker/target 0개 확인.
- 실제 데이터 기반 marker 상세 및 설정/복귀 흐름은 실제 지도 adapter가 없어 실환경 검증하지 않았습니다.
- ROS2/PX4 Bridge, 상태 polling 및 기존 Target/Flight/Sensor/Console 기능은 유지합니다.

## 실행

```bash
cd dashboard
npm run dev
```

http://127.0.0.1:5173/
