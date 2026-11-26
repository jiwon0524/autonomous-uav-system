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


## Detection / 선택적 YOLO adapter

Detection = 현재 frame의 객체 후보 탐지, Target Manager = 현재 목표/논리 상태 관리,
Tracker = 선택된 동일 목표의 frame 간 관측 위치 유지, Mission State Machine = 전체 임무 상태,
Flight Control = 실제 PX4 이동 제어. 이번 구현은 Detection만 수행하며 서로 자동 연결하지 않습니다.

`detection/`의 불변 Detection은 TargetType, class_name, 기존 BoundingBox/FrameInfo,
confidence [0,1], 선택적 class_id를 보존합니다. Target UUID는 생성하지 않습니다.
`vision/yolo.py`만 Ultralytics 객체를 다루고 tuple[Detection, ...]로 변환합니다.
입력은 한 장의 BGR image이며 카메라 획득은 별도 demo에 있습니다. 추적/목표 선택은 없습니다.

기본 모델은 공식 소형 pretrained COCO `yolo11n.pt`입니다. 모델/threshold/device/필터는 변경 가능합니다.
PERSON은 모델 names의 person label로 필터링하므로 class 0을 무조건 가정하지 않습니다.
`--classes person car` 또는 `--classes all`로 확장할 수 있습니다. 이름 기준 person은 PERSON,
bicycle/car/motorcycle/bus/truck/train/boat/airplane은 VEHICLE, 나머지는 OBJECT로 분류합니다.
모델 고유 label과 class_id도 보존하며 이름이 다른 custom model은 별도 매핑 확장이 필요합니다.

Windows 수동 검증(저장소 루트, 별도 Python 3.11/3.12 가상환경 권장):
```powershell
py -3.12 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r requirements-vision.txt
.\.venv\Scripts\python.exe -B -m src.autonomy.vision.webcam_demo --camera 0 --confidence 0.25
# 모델 파일/이름 변경: --model C:\models\custom.pt
```
위 `.venv/`는 기존 Git ignore 대상입니다. 저장소 밖의 별도 가상환경을 사용해도 됩니다.
Ultralytics는 추론/PyTorch, opencv-python은 camera/read/display용 선택 의존성입니다.
기본 CPU로 동작하며 기본 모델의 최초 수동 실행은 weight 다운로드/네트워크가 필요할 수 있습니다.
오프라인에서는 미리 받은 --model 파일을 사용합니다. 모델/영상은 Git에 저장하지 않습니다.
공식 API: https://docs.ultralytics.com/models/yolo11/ , https://docs.ultralytics.com/modes/predict/

bbox는 기존 연속 pixel edge 계약을 유지합니다. 잘못된 geometry/score/배열은 해당 frame 전체를
명시적 오류로 거부하고 부분 결과나 이전 bbox를 제공하지 않습니다. Demo는 stderr와 화면에
오류를 알리고 다음 frame을 처리하며 5회 연속 탐지 실패 시 종료합니다. camera open/read 오류는
즉시 종료합니다. q/ESC/창 닫기로 종료하고 camera/window 자원을 정리합니다.
전체 자동 테스트는 기존 unittest 명령으로 실행하며 CV 설치/모델 다운로드/카메라가 필요 없습니다.
실제 설치 호환성, weight 로딩, PERSON 탐지, webcam 종료/권한은 수동 검증 대상입니다.


## Camera Source

`vision/sources/`는 `CameraSource[PixelT]` Protocol(open/read/close/is_open),
불변 CameraFrame envelope(pixels + 기존 FrameInfo), OpenCVSource adapter를 제공합니다.
Protocol과 frame은 OpenCV에 의존하지 않고 import 시 CV 라이브러리를 로드하지 않습니다.
`OpenCVSource(camera_index=...)`는 내장/USB webcam 및 OS가 camera 장치로 인식한 UVC phone,
`OpenCVSource(url=...)`는 사용자 제공 network/IP/phone stream을 처리합니다.
URL은 scheme/host를 검증한 뒤 그대로 backend에 전달합니다. 특정 앱/주소/장치 번호는 가정하지 않습니다.
실제 URL protocol/codec 지원과 open/read blocking 시간은 설치된 OpenCV backend에 따라 다릅니다.
OpenCV API: https://docs.opencv.org/4.x/d8/dfe/classcv_1_1VideoCapture.html

CameraSource → CameraFrame → YOLO → Detection 흐름이며 YOLO와 도메인 모델은 변경하지 않았습니다.
반환 pixels는 복사하지 않으며 OpenCV adapter는 HxWx3 BGR 영상을 제공합니다. envelope/metadata만
불변이므로 demo의 표시용 그리기처럼 pixel 변경 시 공유 consumer가 있다면 호출자가 조율해야 합니다.
FrameInfo는 성공한 read의 순번, 해상도, UTC host 수신 시각, source ID(frame_id)를 담습니다.
시각은 노출 시각/동기화 시각이 아닙니다. 테스트 clock 주입을 지원하며 source 객체 수명 내 순번은
재open 후에도 증가합니다. 새 객체는 0부터 시작하므로 상위 계층이 세션을 구분해야 합니다.
여러 source를 구분하려면 source_id를 지정합니다(URL 기본 ID에는 자격 증명을 넣지 않습니다).

기존 demo 명령은 유지되며 --source 기본값은 camera입니다(의존성 설치는 위와 동일).
```powershell
# 내장/USB/UVC: OS가 할당한 실제 camera index 선택
.\.venv\Scripts\python.exe -B -m src.autonomy.vision.webcam_demo --source camera --camera 1
# network: 실제 stream URL로 교체
.\.venv\Scripts\python.exe -B -m src.autonomy.vision.webcam_demo --source url --url "<video-stream-url>"
```
--model/--confidence/--classes/--device 및 bbox 표시, q/ESC/창 닫기 종료는 유지합니다.
open/close는 idempotent이며 닫힌 source read는 SourceClosedError로 거부합니다.
open 실패 시 열린 handle을 release하고 SourceOpenError, read 실패 시 source를 닫고
FrameReadError를 반환합니다. release 실패도 CameraSourceError로 명시적으로 알립니다.
자동 재연결/백그라운드 thread/가짜 frame은 없습니다. 종료 시 source와 표시 창을 정리합니다.

향후 Gazebo/ROS2 camera, UAV RGB, Depth camera RGB는 별도 adapter로 CameraSource를 구현하고
같은 CameraFrame/YOLO 입력 흐름에 연결합니다. ROS message 수신을 VideoCapture로 강제하지 않습니다.
이번에는 해당 adapter나 Depth 처리를 구현하지 않았습니다. Bluetooth video는 기본 입력 방식이 아닙니다.
새 의존성은 없으며 실제 장치/stream의 연결·성능·종료는 수동 검증해야 합니다.
