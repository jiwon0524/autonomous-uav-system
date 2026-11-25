# 공간 매핑 인터페이스 초안 v1

이 규격은 팀 합의 전의 격리 시험망 프로토타입이다. Pi/팀원 SLAM → 독립 TCP 점군 채널 → GCS. Stage 11 상태와 Stage 12 RGB 영상 연결, FC 읽기 전용 연결은 별개이며 자동 연동하지 않는다. GCS가 SLAM·정합·비행 제어를 수행하지 않는다.

## 선택 근거

| 방식 | 적용 및 비용 |
|---|---|
| 길이 프레이밍 Binary TCP | 이 단계 선택. 연속 스트림·순서 보장과 적은 파싱 비용, 짧은 코드. TCP 혼잡 시 오래된 패킷이 지연될 수 있어 시각·시간 제한 필요 |
| WebSocket Binary | 브라우저/웹 통합이 쉬우나 프레이밍·서버·접근 제어 추가. 실제 WebView 및 Pi 부하 미측정 |
| PLY | 파일 교환에 적합하며 ASCII/바이너리 변형 존재. 실시간 갱신·세션 정책은 별도 정의 필요 |
| PCD | 점군 필드·ASCII/바이너리·압축 파일 교환에 유용하나 변형 검증과 호환성 비용이 있다. [PCL 형식 설명](https://pointclouds.org/documentation/tutorials/pcd_file_format.html) |
| 압축/타일 포맷 | 대용량 저장·전송 잠재력은 있으나 선택·디코더·캐시·갱신 정책이 필요 |

Windows/Ubuntu TCP는 표준 Rust API를 사용하나 Windows 빌드·실행은 미검증. 전송 크기·CPU·RAM·지연은 실측하지 않았다. 파일 불러오기/저장이나 PLY/PCD 파서는 아직 없다. 대형 Open3D/PCL/ROS2 런타임은 추가하지 않는다.

뷰어는 직접 WebGL 1 `POINTS`를 사용한다. Three.js는 시점·장면 관리가 풍부하지만 추가 번들 및 API 관리가 필요하고 Babylon.js는 더 넓은 엔진 기능과 추가 번들이 필요하다. 기존 OpenLayers는 이 점군용 3D 렌더러가 아니다. 직접 WebGL은 한 가지 제한된 점군 용도에서 의존성을 늘리지 않지만 카메라·축·리소스 관리를 직접 검증해야 한다. [WebGL POINTS API](https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/drawArrays)를 사용한다. 실제 GPU·WebKitGTK·WebView2 호환성과 메모리는 아직 미측정이다.

## 전송 및 메시지

GCS 3D 매핑 화면에서 숫자 IP와 포트를 지정한다. Pi는 반복해서 big-endian u32 헤더 길이, big-endian u32 payload 길이, UTF-8 JSON 헤더, payload를 쓴다. 헤더 1–4096 bytes, payload 1–1,500,000 bytes, 1–100,000점, 초당 최대 2 스냅샷(최소 500ms 간격). 권장 1Hz. 3초 접속 및 5초 유효 프레임 무수신 제한. 오류·순서 역전·초과 속도에서 연결을 닫으며 자동 재접속하지 않는다. 데이터 큐는 없고 최신 스냅샷 하나만 교체한다. 화면 종료 시 연결과 GPU 버퍼를 해제한다.

헤더 필드: `protocol_version` 정수 1, `device_id`, `session_id`, `map_id` (각 1–64 ASCII 영숫자와 `-_.`), `frame_id`, `map_revision`(0–2^53-1), `timestamp_ms`(UTC Unix ms, GCS 시각과 ±5초), `coordinate_frame`, `point_count`, `position_encoding`, `color_encoding`, `payload_length`, `data_type`. `mission_id`는 생략 또는 null이나 동일 ID 규칙. 미지 필드는 거부한다. 같은 session/map 내 frame과 revision은 반드시 증가한다. 새 session/map 스냅샷은 이전 결과를 통째로 교체하며 누적하지 않는다. 식별자가 가리키는 현장과 임무는 사용자/팀의 별도 검증이 필요하다.

`coordinate_frame`: `local_map`, `enu`, `ned`, `camera_optical`, `camera_body` 중 하나. `position_encoding`은 `xyz_f32_le`, `color_encoding`은 `rgb8`, `data_type`은 `snapshot`만 지원한다. 페이로드는 점마다 little-endian IEEE754 f32 x/y/z(미터) 뒤에 unsigned byte R/G/B, 총 15 bytes. 길이는 `point_count*15`와 같아야 한다. 비유한 좌표, ±100,000m 밖 좌표 및 초과 크기는 거부한다. 합성 점군은 실제 데이터처럼 배포하지 않는다.

```json
{"protocol_version":1,"device_id":"pi-01","session_id":"session-1","map_id":"map-1","frame_id":10,"timestamp_ms":1795560000000,"coordinate_frame":"local_map","point_count":1000,"position_encoding":"xyz_f32_le","color_encoding":"rgb8","payload_length":15000,"map_revision":10,"data_type":"snapshot","mission_id":null}
```

예시 시각은 현재 UTC로 바꾼다. 후속 증분의 삭제·교체·재전송은 **미정/지원하지 않음**이며, v1은 delta를 거부한다. 송신자는 손실 복구를 위해 완전한 최신 snapshot을 다시 보내되 같은 연결에서는 frame/revision을 증가시켜야 한다. 확장은 새로운 버전 합의가 필요하다.

## 좌표·자세 및 임무

Camera Optical은 일반적으로 x 오른쪽/y 아래/z 전방, body는 기체별 축 관례를 별도 확인, local map은 송신자가 축·원점·단위를 명시해야 한다. ENU는 동·북·상, NED는 북·동·하, WGS84는 지구 기준 위경도와 별도의 고도 datum이다. `local_map`과 `camera_body`의 실제 축 방향 및 원점은 아직 명세 필드에 없으므로 UNKNOWN으로 취급한다. 이 구현은 좌표계 이름을 표시할 뿐 자동 변환하지 않는다. 단위 기준 변환 함수의 NED→ENU는 `[e,n,-d]`, 단위 quaternion 회전은 별도 테스트했으며 지리 정합에 사용하지 않는다.

내부 파라미터는 렌즈 투영/왜곡, 외부 파라미터는 두 프레임 간 회전·이동이다. 카메라 자세·경로·시야, 기체 자세, camera-to-body 외부 보정은 아직 수신·표시하지 않는다. FC 자세를 카메라와 동일하게 간주하지 않는다. GPS와 SLAM 좌표를 합산하지 않는다. Depth 원본의 frame/time/크기/인코딩/단위/무효값/intrinsics는 [카메라 규격](camera-perception-interface-spec.md)에 정의하며 이번 단계에서 수신·검증 구현은 없다.

`mission_id`는 송신자가 주장하는 메타데이터일 뿐 기존 임무에 자동 연결하거나 같은 현장임을 확인하지 않는다. 저장할 경우 추후 mission ID, session ID, map ID, 출처, 생성 시각, 좌표계, 정합 증거 및 파일 식별자를 검증해 독립 관리해야 한다. 임의 경로 또는 사용자 데이터 파일에 접근하지 않는다.

## 지도 분리와 보안

Prior는 기존 VWorld 실험 구조이며 실제 3D 렌더링 미검증. Live는 수신 점군 단독. Combined는 실제 사전 지도, 유효 점군, 좌표 변환, 정합 기준 및 정확성 검증이 모두 준비되기 전 비활성이다. 보이지 않는 내부를 완성하지 않는다. 이 채널은 평문·미인증·재생 방지 없음; 격리 시험망에서만 사용한다. 운영 전 상호 인증 TLS, 키/권한 관리, 재생 방지와 자원 공격 저항 검토가 필요하다. ID는 인증이 아니다.

## 시험

GUI가 있는 개발 Ubuntu에서 `python3 tools/spatial_fixture.py`를 시작하고 3D 매핑의 Live Mapping에서 `127.0.0.1:8767`로 연결한다. 합성 점군 전용이며 앱에 내장되거나 DEB에 포함되지 않는다. `--case session`, `invalid`, `length`, `flood`, `stop`을 각각 별도 서버 실행에서 사용해 세션 교체 및 오류를 시험한다. 실제 Pi/카메라/SLAM, WebGL 화면과 각 오류 시나리오의 GUI 결과는 미검증이다.
