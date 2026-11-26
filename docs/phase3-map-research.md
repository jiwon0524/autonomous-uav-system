# Phase 3 지도 연구 — 진행 기록

확인일 2026-11-26. 기존 firescene-design-v2-research, design-research, technical-research와 Phase 2 및 VWorld 보고서의 방향은 조용한 지도 위에 기체·현장·범위·출처를 우선하는 작업 공간이다. 검색 결과 이동과 임무 변경을 분리하고 사전 3D와 실시간 재구성을 구분한다.

OpenLayers는 현재 VWorld 래스터 및 GeoJSON 오버레이, 검색과 저장 흐름이 이미 통합된 기본 엔진이다. MapLibre GL JS는 WebGL 벡터 스타일링 및 지구본을 제공하는 별도의 렌더러이며 데이터·한글 라벨·라이선스는 공급자의 별도 문제다. MapLibre 자체는 BSD-3-Clause이고 키를 요구하지 않지만 타일 공급자는 별도 조건을 부과할 수 있다. CesiumJS는 Apache 2.0의 고정밀 지구본·3D Tiles 엔진이나 호스팅 데이터 서비스는 별도 조건이다. 엔진의 무료 라이선스만으로 지도 데이터 배포 권한이나 비용을 판단하지 않는다.

출처: https://maplibre.org/maplibre-gl-js/docs/ ; https://maplibre.org/maplibre-gl-js/docs/guides/mapbox-migration-guide/ ; https://maplibre.org/roadmap/maplibre-gl-js/globe-view/ ; https://cesium.com/platform/ . 한국 데이터 품질, 실제 비용, Windows WebView2 및 Ubuntu WebKitGTK 성능은 별도 실험 전 미확인이다. OpenLayers 교체는 결정하지 않았다.

## 공급자 예비 검토 (2026-11-26)

OpenFreeMap 공식 홈페이지는 OSM 기반 데이터, 키·등록 없는 공개 인스턴스, 상업적 사용과 출처 표시를 설명한다. 오프라인 다운로드에는 별도 제한이 있다. 자원봉사·후원 기반 운영이므로 서비스 지속성을 보장할 수 없다. https://openfreemap.org/ ; https://openfreemap.org/quick_start/ . OSM 데이터의 세부 정확도는 지역별 현장 비교 전 미확인이다. Protomaps는 자체 호스팅 후보지만 호스팅 비용·약관을 별도로 확인해야 한다. MapTiler는 무료 및 유료 플랜이 있는 계정 기반 서비스이며 비용·사용 조건 검토 없이 도입하지 않는다: https://www.maptiler.com/cloud/pricing/ . VWorld 검색·건물 API는 렌더러와 분리된 현재 백엔드 호출로 유지 가능하나 조합 배포 조건은 아직 검토되지 않았다. 다섯 부산 지역과 도로명·지번, 높이·갱신 시점, 캐싱 및 데스크톱 배포 적합성 비교는 미완료다.

2026-11-26 재확인: OpenFreeMap 공식 페이지는 MapLibre 출처 표시, 상업적 사용 허용, SLA 미제공을 명시한다. 이용 약관은 공개 서비스의 자동 데이터 수집 및 별도 조건 없는 오프라인 다운로드를 제한한다. 서비스는 무보증이며 중단될 수 있다. 개발 실험에서만 사용하며 배포 채택을 확정하지 않았다. https://openfreemap.org/ ; https://openfreemap.org/tos/ .
