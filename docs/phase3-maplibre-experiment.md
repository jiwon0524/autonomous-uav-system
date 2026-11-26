# MapLibre 실험 — 미완료

MapLibre는 생산 의존성에 추가하지 않았다. 독립 벡터 데이터 공급자의 한국어 라벨·약관·비용, WebGL 초기화와 복구, 실기기 성능 및 오버레이 테스트가 아직 수행되지 않았다. OpenLayers와 성능이나 품질을 비교했다고 주장할 수 없다. 이를 확인하기 전 기본 엔진 교체를 보류한다.

2026-11-26: maplibre-gl 5.16.0을 `/tmp`에만 받아 패키지 구성을 확인했다. 배포 앱에 추가하지 않았다. 배포 JS 1,015,696 bytes와 CSS 69,449 bytes는 원본 파일 크기로, Vite 번들 증가량이나 런타임 성능 측정치가 아니다. 실제 렌더링, 벡터 타일 및 품질 비교는 아직 하지 않았다.

분리 실험 `experiments/maplibre-globe/`는 maplibre-gl 6.19.0을 사용한다. 최초 검토한 5.16.0은 GHSA-jrc7-96c5-q579 취약점에 해당하여 교체했고 설치 후 npm audit은 0건이었다. 합성 좌표와 단색 배경만 사용하며 지도 공급자 또는 실제 데이터를 제공하지 않는다. Vite 빌드는 성공했으며 별도 실험 빌드 기준 JS 1,056.02 kB, 워커 547.46 kB, CSS 71.86 kB다(기본 앱 증가량 아님). 헤드리스 Chrome 소프트웨어 WebGL에서 캔버스는 생성됐지만 load 완료는 확인되지 않았다. 이 결과로 한국 지도·지구본 동작·성능을 성공 판정하지 않는다. 실행: 해당 폴더에서 `npm ci`, `npm run dev`. 기본 앱과 빌드에는 포함되지 않는다.

## 2026-11-26 실제 타일 재검증

격리 실험을 MapLibre GL JS 6.19.0 + OpenFreeMap Liberty 스타일로 변경했다. 같은 헤드리스 Chrome/SwiftShader에서 단색 평면은 style.load 24ms, load/idle 45ms였다. 실제 타일 실험은 Chrome virtual-time-budget에서 style.load 뒤 제한시간이 났지만 별도의 실제 경과시간 CDP 실행에서는 style.load 77ms, load 438ms, idle 493ms를 관찰했다. 단일 측정이며 서비스·캐시·기계 조건에 따라 변하므로 엔진 성능 비교가 아니다. 실제 OpenFreeMap 부산 화면을 확인했다. Globe 전환 후 idle과 실제 지구 화면을 확인했다. 따라서 기존 실패가 MapLibre 자체의 호환성 실패라는 증거는 없다. 가상 시간과 실제 시간의 차이는 확인됐으나 최초 실패의 단일 원인은 확정되지 않았다.

초기 실험에는 GeoJSON 워커와 단색 스타일이 포함됐고, 재검증은 소스 없는 평면과 원격 스타일을 각각 사용했다. WebGL 캔버스가 생기는 것과 타일 로딩 완료는 구분한다. 실제 시간 실험에서 error 로그는 관찰되지 않았고 이벤트·타임아웃·context lost를 표시한다. 워커 URL은 MapLibre 6 공식 Vite 예제를 따른다. 글리프·스프라이트 개별 실패, context lost 주입과 네트워크 차단 실험은 아직 미실시다. PCI 장치는 NVIDIA 10de:2f04로 열거되지만 nvidia-smi 드라이버 통신 실패: 실측은 GPU 가속이 아닌 Chrome SwiftShader다. Windows WebView2와 Tauri WebKitGTK 실험은 미검증이다.

Dark 실험은 Liberty 레이어에 색상을 적용한다. 초기 화면에서 도로 번호 방패 대비가 나빠 도로 심볼 색 변경을 제외했으나 수정 후 Chrome SwiftShader 화면에서 도로 번호 대비 개선을 확인했으나 더 많은 지역과 Tauri의 검증이 필요하다. Day는 기존 Liberty이며 FireScene 전용 완성 스타일이 아니다. 건물·한글·도시 5곳 비교 및 고도·임무 오버레이는 미검증이다. 공개 서비스는 실험용이며 생산 기능이나 생산 빌드에 포함하지 않는다.

## Tauri 개발 전용 연결

개발 모드 Global 오버레이를 실제 Tauri/WebKitGTK + Xvfb에서 실행해 OpenFreeMap 타일, 지구본 및 평면 전환, 기본 지도 복귀를 확인했다. 최초 프레임에 흰 타일 조각이 관찰됐고 8초 후 증거에서는 사라졌다. 이는 완전한 품질·GPU 검증이나 에러 주입 성공이 아니다. Windows WebView2, Dark/Day 통합 및 부산 5개 지역 비교는 남았다. 화면: `~/FireScene-evidence/phase3/tauri-globe-matrix/global-experiment.png`.
