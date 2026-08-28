# RustFS Launcher

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README_ZH.md">简体中文</a> |
  <a href="./README_DE.md">Deutsch</a> |
  <a href="./README_FR.md">Français</a> |
  <a href="./README_JA.md">日本語</a> |
  한국어 |
  <a href="./README_HI.md">हिन्दी</a>
</p>

RustFS Launcher는 내 PC에서 [RustFS](https://github.com/rustfs/rustfs)를 돌리기 위한 작은 데스크톱 앱입니다. 폴더를 고르고 Launch를 누르면 이 컴퓨터에 S3 호환 API가 뜹니다. 웹 콘솔은 선택입니다.

RustFS 바이너리는 설치 파일에 들어 있습니다. 따로 받을 필요 없습니다.

화면 문구는 영어입니다. 이 문서만 여러 언어로 있습니다.

![시작 전 Launcher 창](docs/images/launcher-ready.png)

## 다운로드

빌드는 [Releases](https://github.com/rustfs/launcher/releases)에 있습니다.

| 컴퓨터 | 파일 |
| --- | --- |
| Windows 10/11, 64비트 | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS, Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS, Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

**Source code**는 앱이 아닙니다. Git 소스입니다.

최신 태그에 소스 zip만 있으면, Assets에 `setup.exe`나 `.app.zip`이 있는 릴리스까지 내려가세요.

ARM Windows는 x86_64 설치 파일을 에뮬레이션으로 실행합니다. Linux 패키지는 아직 없습니다.

## Windows: 받기, 설치, 실행

대부분 이 순서를 탑니다.

### 1. 설치 파일 받기

1. [github.com/rustfs/launcher/releases](https://github.com/rustfs/launcher/releases)를 엽니다.
2. 설치 파일이 있는 릴리스를 고릅니다.
3. **Assets**에서 `rustfs-launcher-windows-x86_64-…-setup.exe`를 받습니다.

### 2. 설치

`.exe`를 더블클릭합니다.

**Windows에서 PC를 보호했습니다**가 뜰 수 있습니다. Authenticode 서명이 아직 없어서 SmartScreen이 막는 겁니다. 이 저장소 Releases에서 받은 파일이면 **추가 정보**, 그다음 **실행**을 누르세요.

NSIS 설치 화면을 따라가면 시작 메뉴에 **RustFS Launcher**가 생깁니다.

### 3. 데이터 폴더를 먼저 만들기

Launcher가 폴더를 만들어 주지는 않습니다. 탐색기에서 빈 폴더를 만드세요. 예: `D:\rustfs\data`. 다른 파일은 넣지 마세요.

PowerShell:

```powershell
New-Item -ItemType Directory -Force -Path D:\rustfs\data
```

### 4. 폼 채우기

시작 메뉴에서 **RustFS Launcher**를 엽니다.

- **Data Path** — **Browse**를 누르거나 폴더를 창으로 끌어다 놓습니다. 필수이고, 폴더가 이미 있어야 합니다.
- **API Port** — 비어 있으면 `9000`.
- **Host** — `127.0.0.1`이면 이 컴퓨터에서만 접속됩니다.
- **Console Endpoint** — 기본은 꺼짐. 웹 UI가 필요하면 켭니다. 포트는 보통 `9001`. API와 같은 번호는 안 됩니다.
- **Access Key / Secret Key** — 처음엔 `rustfsadmin` / `rustfsadmin`이 들어 있습니다. 다른 사람도 쓰는 PC면 바꾸세요.

그다음 **Launch RustFS**.

### 5. 올라온 뒤

상태가 **Service Online**으로 바뀌고, 폼은 잠기며 버튼은 **Stop RustFS**가 됩니다.

![시작에 성공한 Launcher](docs/images/launcher-running.png)

**API** 카드는 `http://127.0.0.1:9000`, **Console**은 `http://127.0.0.1:9001`을 엽니다. 콘솔은 방금 넣은 키로 로그인합니다.

S3 클라이언트는 path-style, 포트 9000입니다.

서버 로그는 데이터 폴더 안이 아니라 옆에 생깁니다. `D:\rustfs\data`면 `D:\rustfs\logs`입니다.

## 창 구성

왼쪽이 설정, 오른쪽이 로그입니다.

**App Logs**는 Launcher 쪽입니다. **RustFS Output**은 서버 프로세스입니다. 실패하면 **RustFS Output**부터 보세요.

왼쪽 업데이트 칸에서 GitHub의 새 Launcher를 확인할 수 있습니다. 그 두 라벨은 아직 중국어입니다(`版本与更新`, `检查更新`). 업데이트는 앱 전체(포함된 RustFS 포함)를 바꿉니다. 데이터 폴더는 건드리지 않습니다. RustFS가 실행 중이면 멈추기 전에 물어봅니다.

창을 닫아도 종료가 아닙니다. 트레이로 숨고 RustFS는 계속 돌아갑니다. 트레이 아이콘 오른쪽 클릭 → **Quit**여야 서버가 멈추고 끝납니다. **Show** 또는 왼쪽 클릭하면 창이 돌아옵니다. 앱을 다시 실행하면 이미 열린 창만 앞으로 옵니다.

## macOS

칩에 맞는 zip을 받아 압축을 풀고 `RustFS Launcher.app`을 응용 프로그램으로 옮깁니다.

GitHub 빌드는 공증되지 않았습니다. 처음 열 때 막힐 수 있습니다.

```bash
xattr -cr "/Applications/RustFS Launcher.app"
open "/Applications/RustFS Launcher.app"
```

또는 앱을 Control-클릭한 뒤 **열기**.

## 안 될 때

**설치 파일이 막힘.** 이 저장소에서 받았는지 확인하세요. 속성에서 **차단 해제**는 확인 후에 해도 됩니다.

**Data path is required / does not exist.** 폴더를 먼저 만들고 다시 Browse.

**포트가 사용 중.** 다른 번호를 쓰거나 9000 / 9001을 쓰는 프로그램을 끄세요.

**콘솔이 안 열림.** Console Endpoint를 켜야 합니다. 포트는 9001이지 7001이 아닙니다.

**Detected Externally.** 그 API 포트에서 이미 뭔가가 듣고 있는데, 이 Launcher가 띄운 프로세스는 아닙니다. 그 프로세스를 끄거나 포트를 바꾸세요. **Stop RustFS**는 이 앱이 시작한 프로세스만 멈춥니다.

**Launch 직후 종료.** 로그 탭 둘 다 보세요. 흔한 원인은 폴더 없음, 쓰기 권한 없음, 포트 충돌입니다.

## 로컬 노드입니다

Launcher는 RustFS를 데스크톱 프로세스로 돌립니다. Windows 서비스가 아닙니다. 시험과 개발용입니다. 운영 클러스터는 Linux 쪽입니다: [docs.rustfs.com/en/installation](https://docs.rustfs.com/en/installation).

Windows 긴 안내: [Install RustFS on Windows](https://docs.rustfs.com/en/installation/windows).

## 소스에서 빌드

[CONTRIBUTING.md](CONTRIBUTING.md)를 보세요.

## 라이선스

[Apache License 2.0](LICENSE).
