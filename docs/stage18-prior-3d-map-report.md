# Stage 18 VWorld Prior 3D 사전 지도 보고

기준 `dev/firescene-gcs`, Stage 15 `e9ad812`, 디자인 `design-v1.0`, Alpha `alpha-v0.1.0`. **기능 완료 아님**: 실제 VWorld 3D 건물·지형과 원격 WebView 권한 격리는 검증되지 않았고 기본 배포는 비활성이다. Stage 16–17 실기체 연결·비행 제어 검증은 하드웨어 준비 후로 보류한다. 이 문서를 포함한 커밋은 `git log -1`로 확인한다. 외부 배포·원격 push는 하지 않았다.

## 원인과 공식 자료

Stage 7에서 3D 초기화 스크립트의 응답은 확인했으나 실제 3D 타일·도메인 인증·WebGL 렌더링은 확인하지 못했다. 당시 받은 SDK 초기화 코드의 프로토콜 분기는 Linux Tauri 커스텀 스킴에서 HTTPS가 아닌 후속 리소스를 선택할 수 있었다. 이는 코드 관찰이며 GUI에서의 실패 원인을 증명하지 않는다. 이번에는 동일 초기화 요청을 반복하지 않았다.

[VWorld 공식 2026 교육 예제](https://github.com/V-world/V-world_API_sample/blob/master/%EB%B8%8C%EC%9D%B4%EC%9B%94%EB%93%9C_%EA%B5%90%EC%9C%A1/2026%EB%85%84_%EB%B8%8C%EC%9D%B4%EC%9B%94%EB%93%9C_%EA%B5%90%EC%9C%A1/Readme.md)는 HTTPS WebGL 3D API 3.0과 `vw.Map`을 보여준다. [공식 3D 예제](https://github.com/V-world/V-world_API_sample/blob/master/%5BWebGL3%5D%20%ED%81%B4%EB%A6%AD%EB%90%9C%20%EB%AA%A8%EB%8D%B8%20%EC%88%A8%EA%B8%B0%EA%B8%B0.html)는 카메라와 3D 객체 조회를 사용한다. 이는 모든 지형·건물의 제공 범위나 특정 도메인·앱 이용 허가를 보증하지 않는다. 현재 사용자 키의 등록 도메인, 인증키 종류, 별도 호스팅 허용 및 실제 타일 이용 조건은 **UNKNOWN**이다. VWorld에 직접 확인해야 한다.

[Tauri capability 문서](https://v2.tauri.app/security/capabilities/)는 remote 권한 설정과 Linux iframe 구분의 제약을 설명한다. [Tauri capability 참조](https://v2.tauri.app/reference/acl/capability/)는 capability가 매칭되지 않는 WebView에는 IPC 접근을 제공하지 않는다고 설명한다. 현재 코드는 명시적 main 전용 capability 및 원격 창을 사용하지만 실제 Linux/Windows에서 앱 명령 거부를 시험하지 않았으므로 격리를 검증된 것으로 취급하지 않는다.

## 현재 구현

기존 실험 feature 및 빌드 시 고정 HTTPS URL을 유지하고, 선택 위치를 `Prior Map`에서 볼 수 있도록 2D의 경도·위도·조회 반경을 공유해 기존 `open_map3d`에 전달한다. 좌표와 반경을 프론트엔드에서 검사하고 Rust의 기존 범위 검사도 유지한다. 별도 창은 고정된 HTTPS origin/path만 탐색하고 쿼리·인증정보를 금지한다. 기존 2D 화면의 실험 버튼과 창 닫기 후 복원 경로도 유지한다. Prior 화면은 feature·URL 미설정 시 버튼을 비활성화하고 이유와 미검증 상태를 표시한다. Live Mapping 구현 및 Combined View의 비활성 상태는 유지한다.

기존 `frontend/public/vworld-3d.html`은 SDK 시작 요청, 선택 좌표·반경 값, 방위·상단·초기화 및 SDK의 탐색 UI를 준비한다. 지도 데이터·카메라 조작의 실제 동작은 미검증이다. 조회 반경은 값으로만 표시되며 3D 원/건물 윤곽은 아직 그리지 않는다. 3D 데이터 없는 지역 자동 판별도 미구현이다. 실험 창은 별도 창 닫기로 종료되며 앱의 Prior 화면을 떠나는 것만으로 자동 닫히지 않으므로 사용자가 창을 닫아야 한다. 앱 종료 시 리소스 정리 및 GPU 영향은 GUI 미검증이다.

## 사용자 조건과 검증 방법

1. VWorld 계정의 키와 등록 서비스 URL을 확인하고, 제공자에게 **정확한 HTTPS 출처, 호스팅 방식, 키 전달 및 Tauri 사용의 허용 조건**을 문의한다. 기존 키와 설정은 변경하지 않았다. 출처를 임의로 바꾸거나 인증 정책을 우회하지 않는다.
2. 사용자가 승인한 신뢰 가능한 HTTPS 호스팅에만 검토된 `frontend/public/vworld-3d.html`을 배포한다. 이 작업에서는 호스팅·가입·배포하지 않았다. 이 페이지와 외부 SDK가 로컬 키를 읽을 수 있으므로 키는 비밀로 보장되지 않으며 호스트의 콘텐츠를 신뢰해야 한다.
3. 보안 검증용 빌드에서만 `FIRESCENE_3D_PAGE_URL=https://<승인된-도메인>/<페이지>` 설정 후 `cargo tauri dev --features experimental-vworld-3d`를 사용한다. 기본 DEB는 이 기능이 꺼져 있다. 등록 키·URL이 일치해야 하는지, 포트·경로 규칙은 제공자에 확인한다.
4. 일반 그래픽 세션에서 2D 건물 선택·반경 → Prior Map에서 좌표 확인 → 실험 창 열기 후 실제 건물·지형·시점·확대/축소·오류·창 종료 및 2D 복원, 미제공 지역을 각각 확인한다. 외부 WebView에서 모든 민감 명령의 접근 거부와 메인 관제·안전 감시 유지도 별도로 시험한다. UI 실패나 검은 원격 캡처만으로 원인을 단정하지 않는다. 승인된 도메인 및 런타임 보안 시험 전 프로덕션 활성화 금지.

## 검증 결과 및 제한

- 프론트엔드 10개 테스트 파일, 좌표/반경 경계 및 기존 모의 SDK 페이지 테스트 통과. TypeScript·Vite 빌드 통과. 모의 SDK는 실제 VWorld 초기화나 렌더링 증거가 아니다.
- Rust 기본 47개, simulation 48개 및 각각 Tauri capability 정적 검사 1개 통과, 합성 성능 시험 각 1개 제외. 실험 feature `cargo check` 통과. 로컬 소켓 검사는 승인된 환경에서 실행했다. 런타임 IPC 거부는 **PENDING**이다.
- Ubuntu 24.04 amd64 릴리스·DEB 생성 성공: `src-tauri/target/release/bundle/deb/FireScene GCS_0.1.0_amd64.deb` **5,710,340 bytes**, Stage 15 5,709,346 bytes 대비 **994 bytes 증가**. 실행 파일 18,698,616 bytes, frontend/dist 700,198 bytes. DEB 목록에 개발 테스트 도구·대용량 지도 자산은 없다. Vite 500 kB 메인 청크 경고는 남는다.
- 실제 3D 지도·GUI·도메인 인증·데이터 제공 범위, 창 자원 해제, 보안 경계, CPU/GPU 성능 및 Windows는 **PENDING**. QGC·PX4 실제 기체와 기능 회귀의 GUI 동시 시험도 보류다. 지도·점군·카메라로 비행 명령을 실행하지 않는다.
