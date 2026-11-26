# Phase 2 — UI/UX 연구 (진행 중)

확인일: 2026-11-26. 기존 `references/research/` 세 파일과 디자인 시스템·동결·V2 및 Stage 20 보고서를 검토했다. 로컬 연구 파일은 공개 저장소에 포함하지 않는다. 공개 화면의 디자인에서 원칙을 도출하며 내부 시스템의 기능은 추정하지 않는다.

| 자료 | 적용할 원칙 | 적용하지 않을 것 |
|---|---|---|
| [Crew Dragon 공개 디스플레이](https://www.shanemielke.com/work/spacex/crew-dragon-displays/), [공개 웹캐스트](https://www.shanemielke.com/work/spacex/webcast/) | 조용한 정상 상태와 정보 우선순위 | 우주선 운용 기능이나 외형 복제 |
| [NASA 디스플레이 지침](https://www.nasa.gov/reference/appendix-f-vol-2/), [승무원 인터페이스](https://www.nasa.gov/reference/10-0-crew-interfaces-vol-2/) | 핵심 상태의 지속 표시, 오래되거나 없는 값 구분 | NASA 인증 또는 동일 안전 수준 주장 |
| [QGroundControl Fly View](https://docs.qgroundcontrol.com/Stable_V5.0/en/qgc-user-guide/fly_view/fly_view.html) | 지도·상태의 작업 중심 배치 | 명령·제어 기능 모방 |
| [DJI Pilot 2 및 FlightHub 2 안내](https://dl.djicdn.com/downloads/DJI_FlightHub_2/20220825/DJI_FlightHub_2_User_Guide.pdf), [Virtual Cockpit](https://fh.dji.com/user-manual/en/real-time-project-information/virtual-cockpit.html) | 영상 중심 화면 및 명확한 연결 상태 | 원격 조종·촬영 기능 추가 |
| [Fluent 접근성](https://fluent2.microsoft.design/accessibility), [Windows 화면 크기](https://learn.microsoft.com/en-us/windows/apps/design/layout/screen-sizes-and-breakpoints-for-responsive-design) | 위계·대비, 물리 해상도가 아닌 사용 가능한 창 크기 기준 설계 | 특정 플랫폼 컴포넌트 외형 복제 |
| [Apple 레이아웃](https://developer.apple.com/design/human-interface-guidelines/layout), [타이포그래피](https://developer.apple.com/design/human-interface-guidelines/typography) | 정렬·점진적 공개, 텍스트 확대에 따른 재배치 | Apple 전용 조작법 강제 |
| [Figma 패널](https://help.figma.com/hc/en-us/articles/360039831974-Explore-the-navigation-bar-and-left-sidebar), [ArcGIS Pro](https://pro.arcgis.com/en/pro-app/3.5/get-started/user-interface.htm) | 주요 캔버스와 문맥 패널 구분 | 편집기 전체 기능 복제 |
| [Blender 영역](https://docs.blender.org/manual/en/5.0/interface/window_system/areas.html) | 작업 영역 확대 후 이전 배치 복구 | 안전 상태까지 숨기는 전체 집중 모드 및 복잡한 자유 분할 |
| [WCAG 2.2](https://www.w3.org/TR/WCAG22/), [Windows 접근 가능한 텍스트](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessible-text-requirements) | 일반 텍스트 4.5:1, 큰 텍스트 3:1, 확대·키보드 검증 | 색만으로 상태 구분하거나 일부 토큰 측정을 전체 적합성으로 주장 |

Pretendard/Noto Sans KR은 설치 시 한글 표시의 일관성을 기대할 수 있지만 동봉하면 용량과 라이선스 관리가 필요하다. Windows의 Malgun Gothic, Ubuntu 및 시스템 sans-serif를 폴백으로 두고 외부 글꼴을 추가하지 않았다. 실제 플랫폼별 폰트 렌더링과 사용자 가독성 평가는 남았다.

## 2026-11-26 후속 적용 검토

기존 연구의 지속적인 연결·안전 정보, 작업 중심의 큰 캔버스와 필요한 경우에만 상세 정보를 여는 원칙을 적용했다. 1150px 이하에서는 메뉴와 상황 패널이 지도 공간을 상시 차지하지 않도록 했고, 3D 연결 설정은 펼칠 수 있는 상세 영역으로 바꿨다. 비행 명령 잠금과 안전 상태줄은 유지한다. 카드 일괄 전환이나 외부 브랜드 복제는 하지 않았다. 연구에서 제안된 운용자 검증, 목적별 화면 전수 개선, 고배율·실기체 실증은 아직 이 적용만으로 충족되지 않는다.

## 최종 재개 적용 검토

기존 연구의 단계별 상세 공개를 짧은 창의 지도 상태·현장 도구와 비행 그래프에 적용했다. 핵심 지도·기체 상태·명령 잠금은 유지하며 사용자 요청 시 세부 정보를 연다. 이는 기존 연구의 SpaceX/NASA 공개 자료에서 얻은 정보 위계 원칙의 응용이며 타사의 화면이나 미공개 운용 기능을 복제한 것은 아니다. Windows DPI와 실제 운용자 평가가 아직 없어 항공 운용 적합성을 주장하지 않는다.
