# 카메라·AI Perception 연동 규격 초안 v1

Stage 12 시험용 계약이다. 팀원과 합의 및 실제 Pi·카메라·GUI 검증 전이며 특정 카메라나 추론 프레임워크에 종속되지 않는다. Pi의 카메라/AI → 별도 영상 TCP → GCS, FC → 읽기 전용 MAVLink → GCS, Pi 상태 → Stage 11 TCP → GCS다. 상태와 영상 연결은 별개이며 서로의 연결을 자동으로 시작하지 않는다. GCS는 영상 채널에서 애플리케이션 데이터를 송신하지 않는다.

## 영상 기술 검토

| 후보 | 지연·대역폭·Pi 부하 | WebKitGTK / WebView2, 통합·크기·확장 |
|---|---|---|
| JPEG 연속 프레임 (MJPEG 계열) | 프레임 단위이고 단순하지만 대역폭과 인코딩 부하가 클 수 있음 | JPEG 디코더 활용, 프로토타입은 Rust에서 크기 검증; 화면 호환성 실측 필요. 새 엔진 없음 |
| RTSP | RTP/코덱에 따라 낮은 지연 가능, 네트워크·Pi 부하는 코덱과 설정에 의존 | WebView 직접 재생을 가정할 수 없어 브리지/엔진 필요 가능; 플랫폼 패키지와 운영 복잡도 증가 |
| WebRTC | 저지연·혼잡 대응 후보, 인코더·신호 교환 부담 | 브라우저 API 사용 가능성이 있으나 두 WebView에서 코덱·권한·ICE 검증 필요. 신호 서버 등 추가 설계 |
| WebSocket 이미지 | JPEG/PNG 프레임 단위; TCP 정체와 압축 비용 | 웹 API와 통합 용이하나 프레이밍·인증·대역폭 제한 필요, JPEG 비용은 그대로 |
| H.264 | 프레임 간 압축으로 대역폭 절감 가능, 인코더/하드웨어와 설정에 의존 | 코덱 자체는 전송이 아니며 컨테이너·전송·플랫폼 디코더가 필요. 라이선스·설치·호환성 확인 필요 |

수치·CPU/RAM·지연은 실제 장비와 부하에 의존하며 측정하지 않았다. [WebRTC 코덱 기준](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/WebRTC_codecs)과 [Windows 코덱 지원](https://learn.microsoft.com/windows/apps/develop/media-authoring-processing/supported-codecs)을 참고하되 이것이 특정 WebView의 동작 보장은 아니다. 현재는 표준 multipart MJPEG가 아닌 **길이 프레이밍 JPEG/TCP**를 선택한다. 이 방식은 프레임별 탐지를 원자적으로 결합하고 WebView에 임의 원격 URL을 제공하지 않는다. 실제 운용에서는 지연·품질·보안·자원 측정 후 WebRTC/H.264 등을 별도 평가한다.

## 시험 전송

GCS 카메라 화면에서 숫자 IP와 포트를 입력한다. 클라이언트가 해당 단일 주소에만 TCP 접속한다. Pi 서버는 반복해서 네트워크 바이트 순서 unsigned 32-bit `header_length`, unsigned 32-bit `jpeg_length`, UTF-8 JSON 헤더, JPEG 바이트를 순서대로 쓴다. JSON에 영상 바이트를 넣지 않는다. 헤더는 1–16,384 bytes, JPEG는 1–524,288 bytes, JPEG SOF 해상도와 헤더가 일치해야 하며 최대 1920×1080이다. JPEG 시작/종료와 SOF를 확인하지만 전체 이미지 디코딩은 WebView에서 하므로 디코딩 실패도 처리한다. JPEG 디코더 공격 내성은 별도 검증해야 한다.

송신은 최대 10 fps(프레임 간 최소 100ms), 시험 권장 5 fps. 한도 초과, 유효하지 않은 형식, 미완료 프레임 3초, 접속 실패 또는 연결 종료 시 접속을 끊는다. 자동 재접속은 없다. 최신 프레임 하나만 보관하고 이전 것은 교체한다. 카메라 화면을 떠나면 연결을 끊는다. 화면 표시가 3초 이상 갱신되지 않으면 영상을 지운다. 프레임 ID는 연결 내에서 단조 증가해야 한다.

## JSON 헤더 및 탐지

모든 헤더 필드는 필수이며 알려지지 않은 필드는 거부한다. 탐지 객체의 `tracking_id`와 `depth_m`은 생략하거나 null로 보낼 수 있다. `protocol_version`: 1; `device_id`, `model_id`: 1–64 ASCII 영숫자/`-_.`; `frame_id`: 0–2^53-1; `timestamp_ms`: 촬영 시각 UTC Unix 밀리초. 발신·수신 시계 차이는 5초 이내여야 하므로 동기화가 필요하다. 이 검사는 지연의 정확한 측정이나 인증/재생 방지가 아니며 UI의 지연은 UNKNOWN이다. `width`, `height`는 JPEG 원본 픽셀.

`detection_frame_id`와 `detection_timestamp_ms`는 각각 영상 `frame_id`, `timestamp_ms`와 완전히 같아야 한다. 탐지 결과가 없으면 빈 배열을 전송한다. `detections` 최대 50개. 각 객체의 `object_id`, `class_id`, `tracking_id`(null 허용)는 같은 ID 문자·길이 규칙, `class_name`은 제어 문자 없는 UTF-8 1–80바이트, `confidence`는 0–1이다. `bbox`는 `[x,y,width,height]`이며 좌상단 원점, 오른쪽 x·아래 y, 각 값은 영상 크기에 대한 0–1 정규화 값이다. 너비·높이는 양수이고 박스 끝은 1 이하여야 한다. `depth_m`은 미터, 0 초과 1000 이하 또는 null이며 송신자가 제공한 추정치다. 물리적 정확성은 검증되지 않았다. 모델이나 결과는 현장 인원 존재 여부를 확정하지 않는다.

```json
{"protocol_version":1,"device_id":"pi-01","frame_id":42,"timestamp_ms":1795560000000,"width":640,"height":480,"model_id":"detector-1","detection_frame_id":42,"detection_timestamp_ms":1795560000000,"detections":[{"object_id":"obj-1","class_id":"person","class_name":"person","confidence":0.85,"bbox":[0.1,0.2,0.3,0.4],"tracking_id":"track-1","depth_m":2.5}]}
```

예시 시각은 실행 시 현재 UTC 캡처 시각으로 교체한다. 전송 중 필드 확장은 v1에서 거부되므로 새 버전을 먼저 협의한다.

## Depth 경계 (미구현)

원본 depth는 frame ID, UTC 캡처 시각 및 시계 기준, width/height, encoding(예: uint16/float32와 엔디언), 원본값당 미터 변환, 최소·최대 유효 거리, invalid 값, 정렬 대상 및 보정 버전을 명시해야 한다. camera intrinsics는 해상도별 fx/fy/cx/cy, 왜곡 모델·계수, 좌표계 및 캘리브레이션 버전을 포함한다. 색상 컬러맵은 시각화 영상일 뿐 원본 미터 거리를 대체하지 않는다. 전송·렌더링·거리 측정은 아직 구현되지 않았다.

## 보안 및 확인

상태와 영상은 모두 평문·미인증이다. 신뢰 가능한 격리망에서만 시험하고 운영 전 상호 인증 TLS, 자격 증명·권한, 재생 방지 및 자원 제한을 설계해야 한다. 장치 ID는 인증이 아니다. 임의 URL, 파일, 셸, FC 명령 경로는 없다. TCP 커널 버퍼·JPEG 디코딩 및 공격성 고속 연결의 실제 CPU/RAM 한계는 미측정이다.

개발 PC에서 Pillow 설치 후 `python3 tools/camera_perception_fixture.py`를 실행하고 GUI 카메라 메뉴에 `127.0.0.1:8766`을 입력한다. 영상은 **DEVELOPMENT FIXTURE** 합성 이미지다. `--case multiple`, `mismatch`, `stale`, `stop`, `invalid`, `flood`로 여러 객체와 거부·중단 상황을 수동 시험할 수 있다. 별도 터미널의 `tools/companion_status_fixture.py`는 상태 채널 시험용이다. 시험 서버·이미지는 DEB에 포함되지 않는다. 실제 Pi의 서버는 위 바이트 프레임을 생성해야 하며 상태 프로토콜은 [Companion 규격](companion-interface-spec.md)을 따른다.
