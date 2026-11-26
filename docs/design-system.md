# FireScene GCS 디자인 시스템 v1.0

기준 구현은 `86971e8`의 `frontend/src/App.tsx`와 `frontend/src/index.css`다. 새로운 화면은 이 구현과 의미 토큰을 재사용한다. 지도 중심의 관제, 명확한 정보 계층, 침착한 정상 상태와 읽을 수 있는 경고를 원칙으로 한다. 사용성이나 안전성 개선이 필요한 경우 근거와 검증을 기록하고 변경할 수 있다. SpaceX는 디자인 철학의 참고이며 FireScene의 기능과 브랜드는 독립적이다.

## 테마와 토큰

Aerospace Dark가 기본이며 Soft Neutral Day를 상단 버튼으로 선택한다. 선택은 현재 세션에만 유지되고 재시작 시 Dark로 돌아간다. 값의 원본은 `frontend/src/index.css`다. 새 화면에서 색을 개별 지정하지 않고 해당 의미의 토큰을 사용한다.

| 역할 | CSS 토큰 | Dark | Day |
|---|---|---|---|
| 배경 / 표면 / 높은 표면 | `--bg` / `--surface` / `--surface-raised` | #141617 / #1b1e1f / #222627 | #edede8 / #fafaf7 / #f1f2ed |
| 지도 작업 공간 | `--map` | #202425 | #e6e8e1 |
| 기본 / 보조 / 낮은 강조 텍스트 | `--text` / `--muted` / `--dim` | #f0f1ee / #a6adaa / #7c8581 | #262b28 / #5e6862 / #6d7771 |
| 강조 / 경고 / 위험 / 비활성 | `--accent` / `--warning` / `--critical` / `--disabled` | #c4d1c8 / #e3b977 / #ed8580 / #8c9490 | #365e4c / #8d551c / #a3322e / #68736c |
| 구분선 / 호버 / 선택 | `--line` / `--hover` / `--selected` | #383e3c / #2b302e / #343c38 | #d4d9d1 / #e6eae3 / #dce6dd |

`--space-1`~`--space-6`은 4, 8, 12, 16, 24, 32px이고 `--radius`는 4px이다. 현 구현의 일부 레이아웃 치수는 직접 지정되어 있다. 기존 스케일과 정렬을 우선하고 새 컴포넌트는 토큰을 활용한다. 글꼴은 Inter, Noto Sans KR, system-ui 순으로 시스템에서 찾으며 외부 폰트를 다운로드하지 않는다. 수치는 tabular numerals를 사용하고 단위를 값과 함께 배치한다. 아이콘은 `Icon`의 간결한 선형 SVG 스타일을 따른다. 버튼은 키보드 포커스 표시를 유지한다.

## 화면과 상태

- 상단에 연결, 비행 모드, ARM, GPS, 배터리 및 명령 잠금 상태를 유지한다. 종합 관제는 지도 작업 공간, 현재 비행 정보, 임무 및 이벤트, 기체 정보 패널로 구성한다.
- 메뉴는 종합 관제, 비행 제어, 임무 계획, 자율비행, 3D 매핑, 카메라, 안전 관리, 비행 기록, 시스템 설정이다. 향후 계기와 지도·영상·3D 기능도 같은 계층, 간격, 아이콘과 테마를 따른다.
- 실제 데이터가 없으면 `—`, 미수신, 미연결 또는 확인 불가를 표시한다. 0과 구분하며 가상의 기체 자세, 값, 임무 진행이나 정상 안전 상태를 표시하지 않는다. 경고는 색상과 문구·아이콘을 함께 사용한다. 명령 잠금은 유지한다.
- 실제/시뮬레이션 구분, 오래된 데이터와 경고 심각도는 향후 데이터 모델과 함께 명시적으로 구현·검증해야 한다. 현재는 연결, freshness 판정 및 실시간 경고 기능이 없다. 색상 토큰의 존재가 안전 기능 구현이나 인증을 뜻하지 않는다.

## 출처 및 후속 개발

[공개 Crew Dragon 디스플레이](https://www.shanemielke.com/work/spacex/crew-dragon-displays/)와 [SpaceX 웹캐스트](https://www.shanemielke.com/work/spacex/webcast/)는 정보 계층 및 데이터 표현의 참고다. QGroundControl은 비행 관제 개념의 참고이며 기능을 복제하거나 구현했다고 뜻하지 않는다. 원본 Figma는 로컬 [firescene-figma.zip](../references/figma/firescene-figma.zip), 심층 연구는 [디자인 연구](../references/research/design-research.txt), [기술 연구](../references/research/technical-research.txt)에 있다. 이 자료들은 로컬 전용이며 Git에 포함되지 않아 다른 체크아웃에서는 제공되지 않을 수 있다. 공개 및 배포 전 권한을 검토한다. 세 번째 공개 포트폴리오의 접근 제한과 분석 범위는 [리뉴얼 보고서](aerospace-design-renewal.md)에 기록했다.
