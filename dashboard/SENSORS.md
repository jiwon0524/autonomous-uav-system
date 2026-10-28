# Sensor Monitor 구조 정리

## 파일

- `src/sensors/model.ts`: 독립 센서 데이터/상태, 상태 전이 event, Depth→Target 및 처리된 obstacle→Navigation interface.
- `src/sensors/store.ts`: 공유 external store, 동일 출처 창 사이 BroadcastChannel snapshot 동기화. 실제 backend 연결 없음.
- `src/sensors/SensorMonitor.tsx`: SensorCard, SensorConnection, SensorDetail, RgbView, DepthView, LidarView 및 3분할/확대.
- `App.tsx`, `DashboardPanels.tsx`, `Interactions.tsx`, `models.ts`, `TargetManagement.tsx`, `styles.css`: 공유 상태 연결 및 기존 UI 유지.
- `tests/sensors.test.mjs`: 기본값, 독립 상태, 이벤트 중복 방지, Target ID 매칭, processed obstacle adapter 테스트.

## 상태 및 데이터

RGB/Depth/LiDAR 각각 DISCONNECTED/CONNECTED/NO_DATA/WARNING/ERROR를 지원합니다. 기본값은 DISCONNECTED이며 수신 시간/해상도/FPS/거리/range/scan points는 null입니다. 특정 하드웨어 모델, topic, SDK, 해상도, FOV, 주파수는 가정하지 않습니다.

현재 visualization은 데이터 없음 placeholder이며 영상 decoder/scan renderer를 구현하지 않습니다. RgbView/DepthView/LidarView는 각 typed data를 받는 향후 renderer 교체 지점입니다. frame/scan source는 모델에 종속되지 않는 adapter reference입니다.

Main AI Camera는 RGB 상태를 공유하지만 운용/AI overlay용 영역으로 유지합니다. Sensor Monitor RGB는 진단용 별도 view입니다. 현재 실제 영상/객체/bbox를 추가하지 않습니다.

## 상태 공유와 lifecycle

메인 App의 `useSensors()` snapshot을 SensorStatus/SystemHealth/AI Camera/Target Detail에서 함께 사용합니다. Sensor Monitor도 같은 store API를 사용합니다. 별도 창은 같은 origin의 `uav-sensor-ui-v1` BroadcastChannel을 통해 최초 snapshot을 요청하고 이후 변경을 받습니다.

향후 실제 Bridge adapter는 메인 창에서 `publishSensorUpdate({id, data})`에 실제 데이터를 전달합니다. 실제 호출자는 현재 없습니다. 센서 모니터는 read-only consumer이며 업데이트를 publish하지 않습니다. 별도 창에서도 작동하는 최초 snapshot 요청 구조를 준비했습니다. 현재는 모든 센서가 미연결이라 실제 갱신에 따른 창 간 반영은 미검증입니다.

store는 processing을 시작/중지하지 않습니다. UI mount/unmount, 확대/복귀, 창 닫기에 capture/YOLO/ROS2 제어 handler가 없습니다. 향후 실제 processing은 UI와 독립된 ROS2/Bridge 프로세스에 위치해야 합니다. 여러 메인 대시보드를 동시에 실제 publisher로 사용하는 구성 및 데이터 freshness/bridge 재연결은 실제 sensor adapter 단계에서 정의해야 합니다. 저장된 과거 센서 상태를 재접속 시 connected로 표시하지 않도록 현재 localStorage persistence는 사용하지 않습니다.

## 연계 지점

- Depth targetDistance는 targetId/거리/실제 timestamp를 받습니다. 연결 상태와 ID가 일치할 때만 Target Detail 거리에 반영합니다. 기본은 `--`입니다.
- Raw ScanPoint[]를 장애물로 변환하지 않습니다. ProcessedSensorObstacle은 실제 처리 결과만 받는 독립 interface이고 향후 Navigation marker adapter 입력입니다. 현재 연결하지 않습니다.
- 센서 상태 변화만 event를 만듭니다. 최초 미연결 상태나 동일 상태 반복 업데이트는 event를 생성하지 않습니다. 향후 실제 adapter가 상태를 바꿀 때 Console에 표시되며 최대 100개 센서 event를 메모리에 보관합니다.
- WARNING/ERROR는 해당 센서에만 색상으로 표시합니다. 전체 Safety Warning과 연결하지 않습니다.

## 검증

- `npm run build`: 통과.
- Node 24 `node --experimental-strip-types --test tests/*.test.mjs`: 기존 Target/Navigation 포함 15개 통과.
- 브라우저에서 센서 3분할, 각 센서 확대/전체 센서 복귀, 상세 metadata `--`, System Health 센서 미연결 표시 확인.
- 테스트 fixture는 tests/ 내부에만 있으며 대시보드에 주입하지 않습니다.
- 실제 센서·ROS2 sensor topic·하드웨어 모델·영상·scan·창 사이 live update는 연결하지 않아 실환경 검증하지 않았습니다.
- 기존 ROS2/PX4 Bridge 코드는 수정하지 않았고 보류 중인 연결 테스트를 재개하지 않았습니다.

## 실행

```bash
cd dashboard
npm run dev
```

메인: http://127.0.0.1:5173/
센서 진단 경로: http://127.0.0.1:5173/?view=sensors

메인의 센서 모니터 열기는 기존 이름이 고정된 단일 별도 창을 재사용합니다. 브라우저 팝업 정책은 브라우저별 확인이 필요합니다.
