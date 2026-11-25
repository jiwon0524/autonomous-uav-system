# Stage 22 Windows 네이티브 빌드 준비

기준 브랜치 `design-v2-experimental`, Stage 21 `79fb17a`. 인터넷 사용을 전제로 기본 WebView2 설치 모드를 `downloadBootstrapper`로 바꿨다. 기존 런타임을 재사용하고 없을 때 다운로드하는 설정이다. 오프라인 대안은 `src-tauri/offline-webview.conf.json`에 별도로 남겼다. 실제 설치기 크기와 런타임 동작은 Windows 빌드·설치 후 확인해야 한다. 공식 [Tauri Windows 설치 문서](https://v2.tauri.app/distribute/windows-installer/)는 다운로드 방식은 설치 시 네트워크가 필요하고 오프라인 방식은 설치 파일에 약 127MB를 더할 수 있다고 설명한다.

수동 Release 워크플로는 사용자 승인 후에만 사용하며 `--draft --prerelease`로 읽기 전용 기체 통신 시험용임과 비행 명령 잠금을 명시한다. 빌드·설치·KOA FC 통신·서명 상태 및 공개 여부는 아직 검증되지 않았다. 기본 브랜치 변경과 Release 공개는 별도 승인이 필요하다.

## Push 전 조사

GitHub의 기본 브랜치는 `main`, 이력상 실험 브랜치는 아직 원격에 없었다. 로컬 기준 39개 커밋, 325개 고유 blob을 확인해 현재 로컬 VWorld 키의 정확한 바이트, 대표 GitHub/AWS/Google 토큰과 개인키 패턴, VWorld UUID 키 패턴 일치 항목은 없었다. 이 제한적 검색은 과거 다른 키나 모든 민감 정보의 부재를 보장하지 않는다. 연구 자료, 원시 JSONL/데이터베이스, 개발 로그는 확인된 Git 이력에 없었다. `tools/camera_test_fixture.jpg`는 합성 단색 시험 이미지이며 프런트엔드 테스트와 GUI 테스트 코드는 개발 소스에 포함되지만 설치 패키지에 리소스로 추가하지 않았다. 기존 보고서 8개에는 `/home/jiwon/` 로컬 사용자 경로가 커밋되어 있고 Git 이력의 일부로 공개된다. 미추적 Stage 0 보고서와 로컬 연구 파일은 제외했다.

기본 브랜치의 수동 `workflow_dispatch`는 워크플로 파일이 기본 브랜치에 있어야 한다. 전체 실험 브랜치를 병합하지 않고 `.github/workflows/windows-build.yml`만 기본 브랜치에 별도 반영하는 방법을 우선 사용한다. 해당 기본 브랜치 변경은 사용자 승인 전 수행하지 않는다. 이후 Actions에서 실험 브랜치를 선택해 Windows 2022 네이티브 빌드와 테스트, NSIS 생성, SHA256, Artifact 업로드를 검증한다. 실패 시 로그를 분석하여 수정하고, 빌드 성공 후 다운로드·내용·서명 및 Windows 설치를 별도로 검증한다. 사전 릴리스 공개는 별도 승인 전 진행하지 않는다.
