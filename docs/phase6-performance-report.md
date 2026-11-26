# Phase 6 성능 — 중간 측정

Ubuntu 격리 개발 Tauri GUI의 2초 유휴 표본 3회 중앙값은 main RSS 242.44 MiB·CPU 약 2.0%, WebKit RSS 509.39 MiB·CPU 약 3.5% (`~/FireScene-evidence/phase6/gui/firescene-phase6-installer-final-gui/results.json`). Phase 5의 별도 실행 유휴 중앙값은 main 240.54 MiB·6.0%, WebKit 440.69 MiB·12.0%였다. 서로 다른 실행의 짧은 표본이며 RSS에는 공유 페이지가 중복될 수 있어 변화 원인이나 회귀를 단정하지 않는다.

시작 CSS 2,054 bytes, JS 535 bytes, 기체 SVG 1,526 bytes. 메인 JS 청크 669.94 kB (gzip 200.73 kB) 경고가 있으며 실제 로딩 영향은 별도 측정이 필요하다. 비재매핑 로컬 DEB 5,755,692 bytes, 경로 재매핑·심볼 제거 DEB 5,134,216 bytes는 빌드 조건이 달라 순수 디자인 전후 비교가 아니다.

동일 조건의 디자인 적용 전후 시작·메인 진입 시간 3회 중앙값, GPU 및 실제 Windows 성능은 미측정이다. 별도 실행 녹화는 동작 증거이지 정밀한 시작 시간 측정이 아니다.

## 격리 시작 비교 추가

Phase 5 `684b035`를 `git archive`로 임시 복사해 release `--no-bundle`로 빌드하고, 현재 `2372bb6`의 경로 재매핑·stripped DEB에서 추출한 release 실행 파일과 비교했다. 동일 Ubuntu Xvfb 1920×1080, WebKitWebDriver 및 격리 사용자 프로필에서 두 실행을 번갈아 3회 실행했다. WebDriver 세션 요청부터 주 메뉴 DOM 발견까지의 시간은 Phase 5 **1.391/1.348/1.373초, 중앙값 1.373초**, 현재 **3.158/3.132/3.166초, 중앙값 3.158초**다. 차이는 1.785초다 (`~/FireScene-evidence/phase6/performance/firescene-phase6-start-{baseline,current}-{1,2,3}/`).

이 측정은 WebDriver·웹뷰 세션 생성 시간을 포함하고 물리 버튼 클릭부터 화면까지의 사용자 체감 시간이나 각각의 초기화 단계 시간이 아니다. Phase 5는 비패키징 실행 파일, 현재는 DEB 번들 패치 후 추출 실행 파일이며 빌드 경로 재매핑도 다르다. 조건이 완전히 같지 않고 원인을 규명하지 않았으므로 디자인 또는 스플래시가 지연을 유발했다고 단정하지 않는다. 관측된 차이는 추가 원인 분석이 필요한 P2 성능 항목이다. GPU 및 Windows 성능은 미측정이다.

추가 진단으로 현재 소스를 `--no-bundle`로 다시 빌드한 실행 파일의 같은 측정은 단일 표본 3.213초였다 (`/tmp/firescene-phase6-start-nobundle-diagnostic/results.json`). DEB 패치만이 차이를 설명한다는 가설은 지지되지 않지만, 단일 표본이며 정확한 원인이나 각 초기화 단계의 지연을 밝히지 못한다.

## 추가 단계 계측

격리 복사본 단일 실험에서 WebDriver 기준 3.226초, DOM interactive 8ms, DOMContentLoaded 1,784ms, load 1,805ms, 프로세스 생성 약 11ms를 관측했다. 서로 다른 기준 시점과 측정 범위이므로 서로 단순 합산하거나 실제 사용자의 클릭부터 체감 시간으로 해석하지 않는다. 기존 스플래시는 React 마운트 시 제거하며 강제 대기를 두지 않는다. 원인 및 수정 효과는 확정되지 않아 제품 코드를 변경하지 않았고 P2 조사로 유지한다. Windows는 미측정이다.
