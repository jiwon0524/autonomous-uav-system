# Target Management UI

- `src/targets/model.ts`: Target 타입, ID별 upsert reducer, 선택/카메라 참조/추적 요청 상태, 향후 console event 형식.
- `src/targets/TargetManagement.tsx`: 허용/미확인 목록, 데이터가 있을 때 유형 필터, 선택 상세 dialog.
- App의 target state는 빈 값으로 시작합니다. 실제 데이터 adapter와 자동 갱신은 연결하지 않습니다.
- 향후 실제 adapter는 `dispatchTarget({type:'upsert', targets: actualTargets})`를 사용합니다. ID가 같으면 대체하고 최초 탐지 시간을 보존합니다. 알려진 최근 탐지 시간보다 오래된 프레임은 무시합니다.
- AUTHORIZED만 허용 목록으로, UNKNOWN/TRACKING/LOST/REACQUIRING은 미확인/추적 목록으로 분류합니다. UNKNOWN은 위험 확정을 의미하지 않습니다.
- 추적 시작은 UNKNOWN 대상에 대한 요청 의도만 기록합니다. backend 확인 전에는 TRACKING으로 전환하지 않으며 현재는 미연결 안내만 표시합니다.
- 카메라에서 확인은 동일 ID 참조만 저장합니다. 현재 카메라 미연결 안내를 표시하며 이미지나 bbox를 만들지 않습니다.
- LOST 마지막 탐지 시간/LOCAL_NED 위치, 재탐색, 실제 소스가 제공한 reacquiredAt 표시를 준비했습니다. 좌표와 타임스탬프는 UI에서 생성하지 않습니다.
- event 타입/formatter는 향후 Console adapter용이며 이벤트를 자동으로 생성하거나 현재 console에 주입하지 않습니다.
- 목록은 높이 제한 및 내부 scroll을 사용하며 빈 상태는 기존 compact 높이를 유지합니다.
- `node --experimental-strip-types --test tests/targets.test.mjs` (Node 24)로 reducer 테스트 5개 실행. fixture는 테스트 파일에만 있고 앱에 포함되지 않습니다.
- 빌드 및 빈 상태 브라우저 확인 완료. 실제 데이터가 없어 상세 dialog/필터/추적 요청의 실제 데이터 기반 브라우저 검증은 추후 필요합니다.
- 기존 Bridge 코드와 상태 hook은 변경하지 않았습니다.
