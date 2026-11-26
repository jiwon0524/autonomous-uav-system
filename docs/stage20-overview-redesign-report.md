# Stage 20-2 종합 관제 화면 개편 결과

`design-v2-experimental`, 기준 632fe3f. Stage 20-1 분석은 [design-v2-analysis.md](design-v2-analysis.md), 실험 디자인 규칙은 [design-system-v2-experimental.md](design-system-v2-experimental.md)에 정리했다. 최신 `references/research/firescene-design-v2-research.txt` 전체를 기준으로 A의 절제된 관제 구조와 C의 기존 통합 검색을 적용했다. B의 지구본은 구현하지 않았다. 이 연구 파일은 기존 로컬 Git 제외 규칙에 따라 커밋되지 않는다.

기존에는 지도 아래 6개 그래프와 임무·이벤트·조종 입력이 세로로 이어지고 기체 정보가 반복됐다. 변경 후 지도는 중앙의 가변 공간, 임무/기체/안전/조종은 우측 상황 탭, 고도/속도/방향과 조종 진입은 하단에 둔다. 메뉴는 접고 펼칠 수 있고, 상세 그래프는 요청 시 펼친다. 기록이 중이 아닐 때도 상태를 표시하며 비활성 기록 상태를 주의색으로 표시하지 않는다. Day와 Dark를 모두 유지한다. OpenLayers/VWorld 타일 및 검색, 임무 저장 모델, 통신 및 수신 인터페이스는 교체하지 않았다. 비행 명령은 잠겨 있다. 상황 탭 또는 메뉴 변경으로 조종 컴포넌트가 해제되면 입력 상태는 버려진다.

변경 파일: `frontend/src/App.tsx`, `Telemetry.tsx`, `RecordingStatus.tsx`, `index.css`, `tests/desktop_qa/run.py`, 본 문서와 실험 디자인 시스템. Stage 20-1은 별도 커밋 `7542166`, Stage 20-2는 이 문서를 포함하는 커밋으로 `git log -1`에서 확인한다.

## 검증

- 프런트엔드 13개 테스트 파일, TypeScript 검사 및 Vite 프로덕션 빌드 통과. 큰 JS 청크 경고는 남아 있다. Rust 코드 변경은 없으며 Tauri 명령 권한 통합 테스트 1개 통과.
- 실제 Tauri/WebKitGTK 개발 빌드를 격리 Xvfb와 임시 XDG 프로필에서 검증했다. 9개 메뉴, Dark/Day, 메뉴 접기/펼치기, 상황 탭, 1200×800 종합 화면 내부 스크롤 없음, 크기 조정 요청, 장소 검색과 레이어 선택, 시험 임무 저장·복제 및 상단 임무 표시, simulation HEARTBEAT/종합 계기/배터리 경고, 시험 기록 저장·재생·내보내기, 오프라인 키 입력 및 패널 전환 초기화가 통과했다. WebDriver 클릭과 합성 키 이벤트를 사용했다.
- Rust 권한 테스트 뒤 한 GUI 실행에서 simulation 버튼이 없어 후속 수신·기록 검사가 실패했다. simulation 빌드를 재선택한 뒤 재실행해 통과했다. 빌드 산출물 경합 가능성은 있으나 원인은 확정하지 않았다. 오류 결과와 재시험을 구분한다.
- 변경 전 캡처: `/tmp/firescene-qa-xvfb/00.png`; 변경 후 캡처: `/tmp/firescene-stage20-retest/00.png` (조종 상황 패널, 1200×800). 초기 화면과 Day 캡처 및 기계 판정은 `/tmp/firescene-stage20-retest/`에 있다. 임시 캡처는 Git에 포함하지 않으며 재부팅 시 사라질 수 있다. 캡처 당시 상태와 크기가 완전히 동일한 통제된 시각 비교는 아니다.
- Xvfb에는 일반 데스크톱 창 관리자가 없어 최대화·복원 결과는 실제 Ubuntu 창 관리자에서 별도 확인이 필요하다. 실제 운영자 가독성, 모든 해상도, 물리 키·포커스/최소화, 전체 지도/임무/기록 흐름, 실기체와 Windows WebView2는 미검증이다. [기존 QA의 미완료 항목](full-qa-test-report.md)은 PASS로 변경하지 않았다.

## 용량 및 성능

새 DEB는 요청에 따라 생성하지 않았다. 이전 패키지는 5,713,738 bytes이며 이번 변경 후 설치 크기는 미측정이다. 새 지도 라이브러리나 폰트 파일은 추가하지 않았다. GUI 재시험의 2초 표본은 Tauri 주 프로세스만 측정하며 유휴 3.0%/231.6 MiB, 지도 5.0%/232.5 MiB, simulation 2.0%/233.1 MiB였다. WebKit 자식과 GPU를 포함하지 않고 이전 측정과 실행 조건이 달라 성능 증감 결론으로 사용할 수 없다. 장시간 사용 및 GPU 측정은 남았다.

## 사용자 확인과 다음 단계

저장소 루트에서 `cargo tauri dev`를 실행하고 종합 관제의 메뉴 접기, 현장/기체/안전/조종 탭, 그래프 펼치기, Dark/Day, 지도 및 기존 임무 선택을 확인한다. 실제 드론 없이 입력 연습을 사용할 수 있으며 비행 명령은 전송되지 않는다. Stage 20-3 임무 UX, 20-4 Tactical Map, 지도 엔진 분리, MapLibre 및 Globe는 별도 단계로 보류한다.
