# Mission State Machine

ROS2와 독립적인 순수 Python 임무 도메인입니다. `core/mission_state_machine.py`의
`MissionStateMachine.handle_event(MissionEvent, reason=...)`로만 상태를 변경합니다.
실제 비행제어, ROS node/topic, PX4 명령, AI 추론, 데이터 생성은 없습니다.

상태는 IDLE, TAKEOFF, PATROL, TARGET_DETECTED, TARGET_SELECTED, TRACKING,
TARGET_LOST, REACQUIRING, AVOIDING, REPLANNING, RETURN_HOME, LANDING, HOLD,
MISSION_ABORTED입니다. 상태와 이벤트는 Enum이며 외부 직렬화에는 `.name`을 사용합니다.

주요 전환:

| 현재 상태 | 이벤트 | 다음 상태 |
|---|---|---|
| IDLE | START_MISSION | TAKEOFF |
| TAKEOFF | TAKEOFF_COMPLETED | PATROL |
| PATROL | TARGET_DETECTED | TARGET_DETECTED |
| TARGET_DETECTED | SELECT_TARGET | TARGET_SELECTED |
| TARGET_SELECTED | START_TRACKING | TRACKING |
| TRACKING | TARGET_LOST | TARGET_LOST |
| TARGET_LOST | START_REACQUISITION | REACQUIRING |
| REACQUIRING | TARGET_REACQUIRED | TRACKING |
| PATROL / TRACKING | OBSTACLE_DETECTED | AVOIDING |
| AVOIDING | REPLAN_REQUIRED | REPLANNING |
| REPLANNING | PLAN_READY | 회피 전 PATROL / TRACKING |
| 활성 상태 | REQUEST_RETURN_HOME | RETURN_HOME |
| RETURN_HOME | HOME_REACHED | LANDING |
| LANDING | LANDING_COMPLETED | IDLE |

이번 단계에서 선택한 도메인 정책(아직 실제 운용 정책으로 검증되지 않음):
- TARGET_DETECTED/SELECTED에서 DISMISS_TARGET, TRACKING에서 TRACKING_COMPLETED는 PATROL로 복귀.
- AVOIDING에서 OBSTACLE_CLEARED는 재계획 없이 원래 상태로 복귀.
- HOLD는 IDLE/LANDING/ABORTED/HOLD를 제외한 상태에서 요청 가능. RESUME_MISSION은 직전 상태로 복귀하며 회피 문맥도 보존.
- 복귀 요청은 IDLE/LANDING/ABORTED/RETURN_HOME을 제외한 상태에서 가능.
- ABORT_MISSION은 IDLE/ABORTED 외 상태에서 가능. ABORTED에서는 RESET_MISSION만 허용.
- RESET은 도메인 재설정일 뿐 착륙/안전 확보를 증명하지 않음. 외부 관리자가 실제 조건을 확인해야 함.

잘못된 전환은 InvalidTransition, 잘못된 인자 타입은 TypeError로 거부하며 상태/문맥은 보존합니다.
성공 시 불변 Transition(previous_state, state, event, reason, sequence)을 반환하고 last_transition에 유지합니다.
allowed_events()로 현재 허용 이벤트를 확인할 수 있습니다. 자동 타이머, 로그 저장소, 무한 history는 없습니다.
단일 소유자가 순서대로 이벤트를 전달하는 구조이며 다중 스레드 동시 호출은 지원하지 않습니다.

향후 detector/Target Manager/tracker/planner/Mission Manager가 검증된 이벤트를 전달하고,
별도 ROS2/PX4 adapter, Dashboard, Mission Logger가 전환 결과를 사용할 예정입니다.
현재는 이 모듈들과 연결하지 않으며 상태 변경이 비행 명령 실행을 의미하지 않습니다.

저장소 루트에서 ROS2 없이 테스트(Python 3.10 이상):

```bash
python -B -m unittest discover -s tests -v
```

코드를 임포트할 때 autonomy 패키지 경로는 src 아래입니다. 테스트는 저장소 루트의
Python namespace 경로(src.autonomy.core)를 사용하며 pip 설치나 ROS workspace 빌드가 필요하지 않습니다.

## Target Manager

`targets/target_manager.py`는 순수 Python 객체 목록/식별/primary 선택 및 추적 상태 관리입니다.
Mission State Machine을 import하거나 직접 호출하지 않으며 ROS2/카메라/Tracker와 연결하지 않습니다.

- Detection: 화면의 객체 발견. Target Manager: 객체와 현재 목표 관리.
- Tracker: 선택된 동일 객체를 프레임 간 추적. Mission State Machine: 전체 임무 상태 관리.
- Flight Control: 실제 PX4 이동 명령 생성(이번 계층에는 없음).

Target은 UUID 내부 ID, PERSON/VEHICLE/OBJECT 유형, 상태, 선택 여부, 선택적 외부 ID,
마지막 Observation과 TargetChange(최근 변경과 최근 상태 전환을 구분)를 가진 불변 snapshot입니다. 목록은 tuple로 반환합니다.
register_target/get_target/list_targets/select_target/get_selected_target/deselect_target,
start_tracking/mark_lost/start_reacquiring/reacquire/update_observation API를 제공합니다.
외부 ID는 하드웨어에 종속되지 않는 ExternalTargetId(source, value)이며 같은 쌍이면
같은 내부 Target을 갱신합니다. 다른 type으로 재등록하면 거부합니다. 외부 ID가 없으면
등록마다 별개 Target이며 시각적 중복 판별/서로 다른 외부 ID의 동일 객체 매칭은 하지 않습니다.
source는 세션/출처 범위를 포함해 ID 재사용 충돌을 방지하도록 외부 adapter가 정해야 합니다.

상태: DETECTED → SELECTED → TRACKING → LOST → REACQUIRING → TRACKING.
REACQUIRED는 지속 상태가 아니라 TRACKING 복귀 전환 이벤트입니다.
TargetChange는 이전/현재 상태, 이벤트, 선택 전후, 논리적 순번을 보존합니다.
last_changes는 최근 변경 작업의 불변 tuple(교체 시 두 대상)을 제공하며 전체 이력 저장소는 아닙니다.
미래 adapter가 Target event를 Mission event로 변환할 수 있지만 현재 연결하지 않습니다.

선택한 정책(향후 운용 검증/변경 필요):
- primary는 최대 1개. 기본 REPLACE, 선택적으로 REJECT 정책을 생성자에서 지정합니다.
- 교체/해제 시 이전 SELECTED/TRACKING은 DETECTED, LOST/REACQUIRING은 LOST로 남깁니다.
  LOST를 다시 선택해도 SELECTED로 정상화하지 않고 명시적 재탐색/재획득을 거쳐야 합니다.
- 같은 primary 재선택과 같은 Observation 재전달은 변경 없는 idempotent 동작입니다.
- 관측 sequence는 외부가 제공하는 target별 증가 정수입니다. 오래되거나 같은 순번의 충돌 관측은 거부합니다.
  재획득은 REACQUIRING인 primary와 더 새로운 Observation을 요구합니다. 동일성 판단은 외부 adapter 책임입니다.
- 관측 갱신 자체는 LOST를 복구하지 않습니다. 실제 수신 adapter는 재획득 시 reacquire에 관측을 직접 전달합니다.
- 외부 clock을 읽지 않습니다. 선택적 observed_at은 timezone-aware datetime이며 sequence가 순서 기준입니다.
- 단일 소유자의 순차 호출을 전제로 하며 자동 timeout/추론/스레드/명령/가짜 데이터 생성은 없습니다.

저장소 루트의 기존 `python -B -m unittest discover -s tests -v`로 Mission과 Target 테스트를 함께 실행합니다.

## Tracking 계약

기존에는 Target 관측 순번/상태 관리만 있었으며 bbox 또는 Tracker 구현은 없었습니다.
`tracking/`은 불변 BoundingBox, ImageSize, FrameInfo, TrackingObservation과
제네릭 `Tracker[FrameT]` Protocol만 제공합니다. 실제 tracker/adapter는 아직 없습니다.

역할: Detection은 객체 후보 발견, Target Manager는 현재 목표/논리 상태 관리,
Tracker는 선택된 목표의 frame별 관측 위치 제공, Mission State Machine은 전체 임무 상태,
Flight Control은 실제 PX4 이동 명령을 담당합니다. 현재 서로 자동 연결하지 않습니다.

선택한 좌표/결과 정책:
- bbox는 좌상단 원점의 연속 pixel edge(x 오른쪽/y 아래), x2/y2는 exclusive 최대 경계.
  유한·비음수 좌표와 양수 면적만 허용하며 자동 clipping/정규화는 하지 않습니다.
- width/height/center는 bbox에서 계산합니다. ImageSize가 있으면 경계 밖 bbox를 거부합니다.
  이미지 해상도 정보는 향후 화면 중심 오차 계산용이며 현재 오차 제어/거리/3D 변환은 없습니다.
- TrackingObservation은 기존 Target UUID를 참조하며 ID를 발급하거나 대상 존재를 검증하지 않습니다.
  FrameInfo는 기존 Observation의 sequence/선택적 timezone-aware 시각 계약을 재사용합니다.
- observed=True는 bbox 필수, confidence는 None 또는 유한한 [0,1]. 실패는 bbox/confidence/center 없음.
  success는 observed의 파생값이므로 별도의 추적 상태 머신은 없습니다.
- 선택적 외부 ID는 기존 ExternalTargetId를 재사용합니다. frame_id는 frame 메타데이터이며 Target ID가 아닙니다.
- 향후 실제 YOLO/ByteTrack/OpenCV adapter가 Tracker.initialize(선택 UUID, 초기 ROI, frame, frame_info)와
  track(UUID, frame, frame_info)를 구현합니다. FrameT는 불투명한 입력 payload로 특정 CV 타입을 가정하지 않습니다.
  결과의 UUID/frame 메타데이터 일치와 실제 객체 동일성은 adapter 책임입니다.
- 관측 실패 자체는 Target LOST를 호출하지 않습니다. LOST/REACQUIRING 정책은 기존 Target Manager에 남고,
  향후 상위 adapter가 결과를 해석합니다. 초기화/순번 검증 등 tracker 내부 규약은 concrete adapter 책임입니다.

기존 `python -B -m unittest discover -s tests -v`에 Tracking 계약 테스트가 포함됩니다.
테스트 fixture는 tests/에만 있으며 runtime 가짜 관측/모델/영상은 생성하지 않습니다.
