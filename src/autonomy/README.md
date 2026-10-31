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
