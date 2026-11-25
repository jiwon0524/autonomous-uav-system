# Stage 22 Windows 네이티브 빌드 준비

기준 브랜치 `design-v2-experimental`, Stage 21 `79fb17a`. 인터넷 사용을 전제로 기본 WebView2 설치 모드를 `downloadBootstrapper`로 바꿨다. 기존 런타임을 재사용하고 없을 때 다운로드하는 설정이다. 오프라인 대안은 `src-tauri/offline-webview.conf.json`에 별도로 남겼다. 실제 설치기 크기와 런타임 동작은 Windows 빌드·설치 후 확인해야 한다. 공식 [Tauri Windows 설치 문서](https://v2.tauri.app/distribute/windows-installer/)는 다운로드 방식은 설치 시 네트워크가 필요하고 오프라인 방식은 설치 파일에 약 127MB를 더할 수 있다고 설명한다.

수동 Release 워크플로는 사용자 승인 후에만 사용하며 `--draft --prerelease`로 읽기 전용 기체 통신 시험용임과 비행 명령 잠금을 명시한다. 빌드·설치·KOA FC 통신·서명 상태 및 공개 여부는 아직 검증되지 않았다. 기본 브랜치 변경과 Release 공개는 별도 승인이 필요하다.

## Push 전 조사

GitHub의 기본 브랜치는 `main`, 이력상 실험 브랜치는 아직 원격에 없었다. 로컬 기준 39개 커밋, 325개 고유 blob을 확인해 현재 로컬 VWorld 키의 정확한 바이트, 대표 GitHub/AWS/Google 토큰과 개인키 패턴, VWorld UUID 키 패턴 일치 항목은 없었다. 이 제한적 검색은 과거 다른 키나 모든 민감 정보의 부재를 보장하지 않는다. 연구 자료, 원시 JSONL/데이터베이스, 개발 로그는 확인된 Git 이력에 없었다. `tools/camera_test_fixture.jpg`는 합성 단색 시험 이미지이며 프런트엔드 테스트와 GUI 테스트 코드는 개발 소스에 포함되지만 설치 패키지에 리소스로 추가하지 않았다. 기존 보고서 8개에는 `/home/jiwon/` 로컬 사용자 경로가 커밋되어 있고 Git 이력의 일부로 공개된다. 미추적 Stage 0 보고서와 로컬 연구 파일은 제외했다.

기본 브랜치의 수동 `workflow_dispatch`는 워크플로 파일이 기본 브랜치에 있어야 한다. 전체 실험 브랜치를 병합하지 않고 `.github/workflows/windows-build.yml`만 기본 브랜치에 별도 반영하는 방법을 우선 사용한다. 해당 기본 브랜치 변경은 사용자 승인 전 수행하지 않는다. 이후 Actions에서 실험 브랜치를 선택해 Windows 2022 네이티브 빌드와 테스트, NSIS 생성, SHA256, Artifact 업로드를 검증한다. 실패 시 로그를 분석하여 수정하고, 빌드 성공 후 다운로드·내용·서명 및 Windows 설치를 별도로 검증한다. 사전 릴리스 공개는 별도 승인 전 진행하지 않는다.

## 2026-11-25 Windows 네이티브 실행 결과

사용자 승인 후 실험 브랜치 `0f1b1f5`를 일반 push하고, 최신 원격 `main`에 `.github/workflows/windows-build.yml` 한 파일만 추가하는 `b9790d6`을 fast-forward했다. 기존 main 코드와 태그 및 Git 이력은 변경하지 않았다. 저장소 로컬 Git 이메일을 GitHub noreply로 설정했다. **기존 이력의 개인 이메일과 로컬 사용자 경로는 이전에 공개된 상태이며 재작성하지 않았다.**

Windows 2022 네이티브 [Actions 실행](https://github.com/jiwon0524/autonomous-uav-system/actions/runs/45629925426)은 커밋 `0f1b1f5f3a80c5b381e686c6e5cadbba9e98612e`로 성공했다. 프런트엔드 테스트 24개 및 TypeScript 검사 PASS. Windows Rust 34 PASS / 1 ignored, Tauri 권한 통합 1 PASS. COM 이름 검증과 Windows 임무 동시 쓰기 잠금 단위 테스트가 통과했다. 실제 COM 포트 열기와 KOA FC 통신은 테스트하지 않았다.

NSIS 설치 파일 `FireScene-GCS_0.1.0_x64-setup.exe`, **3,726,867 bytes**, SHA256 `04ad113530f57ff4f839921848c7da2631c0d01779cc6f454d6e3c21788f5de5`. 다운로드 후 체크섬이 일치했다. [Actions Artifact](https://github.com/jiwon0524/autonomous-uav-system/actions/runs/45629925426/artifacts/15032127176)는 2026-12-09 14:23:28 UTC 만료 예정이며 GitHub 로그인이 필요할 수 있다. 로컬 사본은 Git에서 제외된 `dist/windows/`에 있다.

7-Zip으로 추출한 구성은 x64 `firescene-gcs.exe`와 NSIS 설치기 플러그인(`System`, `nsDialogs`, `nsis_tauri_utils`, `StartMenu`, `NSISdl`) 및 마법사 이미지다. NSIS 부트스트랩 외피는 PE32 x86이지만 포함된 FireScene 본체는 PE32+ x86-64다. 설치기와 본체의 PE 인증서 테이블이 모두 없어 **서명되지 않은 시험 빌드**다. 바이너리의 import table에는 Windows 시스템 DLL 및 Universal CRT 계열이 나타난다. WebView2의 다운로드 설치는 설정상 구성되었으며 실제 설치기 실행으로 확인하지 않았다.

설치기와 추출한 본체에 대해 현재 로컬 VWorld 키의 정확한 바이트와 생성기·QA 표식을 검색한 결과 일치 항목이 없었다. NSIS 목록에 simulation 생성기·QA 도구·외부 프로그램·연구 자료는 별도 파일로 없었다. 문자열 검사는 압축 또는 다른 인코딩의 모든 비밀·기능 부재를 증명하지 않는다. 기본 feature 빌드이며 소스상 simulation receiver는 기본 handler에 등록되지 않는다.

**Windows 실제 설치·제거, SmartScreen, WebView2 미설치 상태, COM 접근·포트 점유, FC와 실 데이터, 업데이트·데이터 보존은 NOT TESTED.** Release는 생성하거나 공개하지 않았다. 실행 가능한 설치 파일 확보와 사용자 노트북에서의 동작 확인은 별개다.
