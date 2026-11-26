# PHASE 03-1 — ROS2 / PX4 연결 상태

기존 UI를 유지하고 연결 상태 두 개만 연동합니다. 비행 명령 publisher, telemetry, 센서, 지도 API는 없습니다.

## 구조

PX4 SITL → Micro XRCE-DDS Agent → ROS2 `/fmu/out/vehicle_status_v4` → `dashboard_bridge/bridge.py` → HTTP `GET /api/status` → Vite proxy → React polling.

Bridge는 Python 표준 라이브러리 HTTP 서버를 사용합니다. 추가 pip framework는 필요하지 않습니다. ROS2 환경의 `rclpy` 및 빌드된 `px4_msgs.msg.VehicleStatus`는 필요합니다.

- ROS2: rclpy executor가 정상 동작 중일 때 connected. 외부 ROS2 노드 전체의 건강을 의미하지 않습니다.
- PX4: 실제 VehicleStatus callback 수신 후 3초 이내일 때만 connected. 시작 시에는 disconnected.
- 단조 시계를 사용하므로 SITL simulation time 및 시스템 시간 변경과 무관합니다.
- Best-effort / volatile sensor-data QoS로 구독합니다.
- Bridge 불통, HTTP 오류, 잘못된 JSON, executor 정지 시 연결 안 됨.
- 프론트엔드: 응답 완료 후 1초마다 polling, 요청 timeout 2초. 중복 요청 없음.
- Flight Status와 System Health는 App의 동일 hook 결과를 공유합니다.
- Console은 관측된 상태 전이만 기록하며 최대 200개를 메모리에 보관합니다. polling 사이에 일어난 짧은 전이는 관측하지 못할 수 있습니다.
- 상단 통신(LINK), 고도, 속도, 배터리, 위치 및 임무 상태는 연동하지 않습니다.

## Linux/ROS2 환경에서 실행

현재 Windows 작업 폴더에는 기존 PX4/SITL 및 ROS2 workspace가 없습니다. 기존 SITL/Agent 실행 명령과 px4_msgs overlay 경로는 확인하지 못했으므로 만들어 적지 않습니다. 기존에 검증한 명령으로 SITL과 Agent를 실행하세요.

먼저 이 `dashboard_bridge`와 `dashboard` 폴더를 Linux 프로젝트 루트로 복사합니다. 프로젝트 루트에서:

```bash
source /opt/ros/jazzy/setup.bash
# 이어서 기존 px4_msgs workspace의 실제 install/setup.bash를 source합니다.
python3 -c "from px4_msgs.msg import VehicleStatus; import rclpy; print('ROS2 imports OK')"
python3 dashboard_bridge/bridge.py
```

독립 스크립트 자체가 ROS2 node이므로 colcon 패키지를 추가 빌드할 필요가 없습니다. 기본 주소는 `127.0.0.1:8765`, timeout 변경은 `--timeout 5`입니다. ROS_DOMAIN_ID와 RMW 설정은 기존 PX4/ROS2 환경과 같아야 합니다. `VehicleStatus` 타입과 PX4 firmware의 메시지 정의도 일치해야 합니다.

다른 터미널, 프로젝트 루트에서:

```bash
cd dashboard
npm ci
npm run dev
```

브라우저: http://127.0.0.1:5173/

서로 다른 호스트에서 실행하는 경우에만 Bridge `--host`와 `DASHBOARD_BRIDGE_URL`을 실제 네트워크 주소에 맞게 설정하세요. 기본은 localhost이며 인증 없는 API이므로 신뢰된 개발 네트워크에서만 노출합니다.

```bash
# dashboard 터미널에서 실제 Bridge 주소를 설정한 뒤 Vite 재시작
# DASHBOARD_BRIDGE_URL=http://<실제-bridge-host>:8765 npm run dev
```

배포 빌드에서는 `/api/status`를 Bridge로 전달하는 동일 출처 reverse proxy가 필요합니다. Vite 개발/preview 서버에는 proxy가 설정되어 있습니다.

## 실제 연결 확인

```bash
curl --fail http://127.0.0.1:8765/api/status
ros2 topic info /fmu/out/vehicle_status_v4 --verbose
ros2 topic echo /fmu/out/vehicle_status_v4 px4_msgs/msg/VehicleStatus --qos-reliability best_effort --qos-durability volatile --once
```

1. Dashboard만 실행: ROS2/PX4 모두 연결 안 됨.
2. Bridge 실행, VehicleStatus 없음: ROS2 연결됨 / PX4 연결 안 됨.
3. 실제 SITL+Agent 메시지 수신: 두 항목 연결됨(녹색). 왼쪽과 시스템 상태 상세가 동일해야 합니다.
4. SITL 또는 stream 중단: 마지막 수신 후 3초 및 다음 polling 이내 PX4 연결 안 됨. ROS2는 유지.
5. Bridge 종료: 다음 실패한 polling 후 두 항목 연결 안 됨.
6. Console을 열어 연결 전이마다 한 번만 기록되는지 확인합니다.

## 이번 Windows 환경 검증 결과

- TypeScript/Vite production build 통과.
- Python connection state 단위 테스트 통과: 미수신, ROS2만 연결, 수신, timeout 경계, 회복, executor 정지.
- Bridge HTTP endpoint 테스트: 실제 로컬 HTTP 요청으로 JSON/no-store/404 확인.
- 브라우저에서 Bridge 없는 경우 ROS2/PX4 미연결 및 기존 UI 표시 확인.
- ROS2 Jazzy/PX4 runtime 실행 및 실제 topic 수신은 미확인. 현재 Windows이며 사용자가 Linux 환경으로 재부팅해야 하므로 가짜 topic이나 모의 backend를 대시보드에 연결하지 않았습니다.

```bash
python3 -m unittest discover -s dashboard_bridge/tests -v
```

참고: [PX4 ROS2 QoS 가이드](https://docs.px4.io/main/en/ros2/user_guide).
