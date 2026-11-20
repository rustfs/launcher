# RustFS Launcher

[English](README.md) · [Deutsch](README_DE.md) · [Français](README_FR.md) · [日本語](README_JA.md) · **한국어** · [简体中文](README_ZH.md) · [हिन्दी](README_HI.md)

RustFS Launcher는 S3 호환 오브젝트 스토리지 서버인 [RustFS](https://github.com/rustfs/rustfs)를 내 컴퓨터에서
실행해 주는 작은 데스크톱 앱입니다. 폴더를 하나 고르고 버튼을 한 번 누르면 RustFS가 모든 네트워크
인터페이스의 `9000` 포트에서 수신합니다. 이 컴퓨터에서는 `http://127.0.0.1:9000`입니다. 같은 네트워크의
다른 기기는 이 컴퓨터의 주소와 같은 포트로 접속합니다. 처음 키는 `rustfsadmin` / `rustfsadmin`입니다.

서버는 앱 안에 함께 들어 있습니다. 따로 설치할 것도, 터미널을 열 일도, 설정 파일을 고칠 일도 없습니다.

![시작 후: 수신 주소, 현재 키, 그리고 중지](docs/images/launcher-running.png)

## 이런 데 쓸 수 있습니다

- Docker 없이 노트북에서 오브젝트 스토리지를 돌려 개발·테스트하거나 개인 백업 대상으로 사용하기.
- 아무 S3 클라이언트나 SDK를 엔드포인트로 향하게 해서 클라우드의 버킷처럼 쓰기. 이 컴퓨터에서는 `127.0.0.1`입니다. 같은 네트워크의 다른 기기에서는 이 컴퓨터의 주소와 포트 `9000`을 씁니다.
- 시작한 뒤 수신 주소와 현재 키를 복사하기. 로그는 직접 열기 전까지 숨겨져 있습니다.
- 원할 때 RustFS를 시작하고 멈추기. 서버가 도는 동안 런처는 트레이에 남아 있습니다.
- 런처와 그 안에 들어 있는 RustFS를 앱에서 바로 업데이트하기.

## 내려받기

[릴리스 페이지](https://github.com/rustfs/launcher/releases)에서 사용 중인 환경에 맞는 파일을 받으세요.

| 사용 환경 | 파일 |
| --- | --- |
| Windows 10 / 11 (64비트) | `rustfs-launcher-windows-x86_64-<version>-setup.exe` (약 50 MB) |
| macOS (Apple Silicon) | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS (Intel) | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

Windows on ARM에서도 동작합니다. x86_64 빌드가 에뮬레이션으로 실행됩니다. 리눅스 패키지는 아직 없으며,
리눅스에서는 [RustFS 바이너리](https://github.com/rustfs/rustfs/releases)를 직접 실행하면 됩니다.

파일 용량이 큰 이유는 RustFS 서버 전체가 설치 파일에 들어 있기 때문입니다.

## Windows, 하나씩 따라 하기

### 1. 설치 파일 내려받기

[릴리스 페이지](https://github.com/rustfs/launcher/releases)를 열고 최신 릴리스의 **Assets**를 펼친 뒤
`-setup.exe`로 끝나는 파일을 클릭합니다. Edge나 Chrome은 흔치 않은 설치 파일에 경고를 띄우기도 합니다.
물어보면 **유지**를 선택하세요.

### 2. SmartScreen 넘어가기

파일을 두 번 클릭합니다. **"Windows의 PC 보호"** 화면이 나오면 **추가 정보**를 누르고 **실행**을 선택하세요.
설치 파일에 아직 상용 코드 서명 인증서가 없어서 나오는 경고이며, 파일 자체에 문제가 있다는 뜻은 아닙니다.

### 3. 설치 마법사 진행하기

마법사는 환영 화면, 라이선스, 설치 위치, 시작 메뉴 폴더를 차례로 묻습니다.

- 기본 설치 위치는 `C:\Users\<사용자>\AppData\Local\RustFS Launcher`입니다. 현재 사용자 계정에만 설치되므로
  관리자 권한을 요구하지 않습니다.
- PC에 Microsoft WebView2 런타임이 없으면 설치 파일이 한 번 내려받습니다. 이 단계에는 인터넷 연결이
  필요합니다.
- 마지막 화면에서 **Run RustFS Launcher**는 체크된 상태로 두세요. 바탕 화면 바로 가기도 여기서 만들 수
  있습니다.

설치가 끝나면 시작 메뉴의 **RustFS Launcher**에서 실행할 수 있습니다.

### 4. 데이터 폴더 만들기

RustFS는 지정한 폴더에 오브젝트를 저장하며, 그 폴더가 미리 있어야 합니다. 탐색기에서 `D:\RustFS\data` 같은
폴더를 먼저 만들어 두세요. 여유 공간이 있는 드라이브의 빈 폴더가 가장 안전합니다.

서버는 자체 로그 파일을 그 폴더 옆의 `logs` 폴더(이 예시에서는 `D:\RustFS\logs`)에 기록합니다.

### 5. 데이터 폴더 고르기

![시작 전: 데이터 경로와 Launch RustFS](docs/images/launcher-ready.png)

첫 화면은 데이터 경로와 **Launch RustFS**입니다. **Launch RustFS**는 항상 보입니다. 경로가 비어 있으면 불투명도 40%로 눌러지지 않습니다. **Browse**로 고르거나, 경로를 입력하거나, 폴더를 창에 놓으세요.

호스트, 포트, 콘솔, 키는 이 화면에 없습니다. 헤더 끝의 톱니바퀴가 **Settings**를 엽니다. 호스트, API 포트, 콘솔을 고치는 작은 정사각 패널입니다. 두 번째 시작 버튼도, 별도 페이지도 아닙니다.

![설정: 호스트, API 포트, 콘솔](docs/images/launcher-config.png)

새로 설치하면 기본값은 다음과 같습니다.

| 설정 | 기본값 |
| --- | --- |
| **Host** | `0.0.0.0`. 같은 네트워크의 다른 기기에서 도달할 수 있습니다. 이 컴퓨터는 계속 `127.0.0.1`을 엽니다. |
| **API 포트** | `9000` |
| **Console** | 켜짐, 포트 `9001` |
| **Access key** / **Secret key** | `rustfsadmin` / `rustfsadmin`. 런처가 새 쌍을 만들지 않습니다. |

호스트, 포트, 콘솔은 시작 전에 **Settings**에서 바꿉니다. 키는 서버가 올라온 뒤 **Change administrator**에서 바꿉니다. 그 변경은 다음 시작부터 쓰입니다. 실행 중인 서버는 다시 시작할 때까지 시작할 때 쓰던 키를 유지합니다.

바닥글에는 현재 버전, 그다음 **Upgrade**, **Website**, **Docs**, **GitHub**가 있습니다. **Upgrade**는 업데이트 패널을 엽니다. 주 버튼이 아닙니다.

입력한 값은 저장되므로 다음부터는 클릭 한 번으로 시작할 수 있습니다.

### 6. Launch RustFS 누르기

경로와 **Launch RustFS**는 스크롤하지 않아도 보입니다. 960×760 창과 최소 720×640에서도 같습니다. 로그는 서버가 시작된 뒤 **View logs**를 누르기 전까지 숨겨져 있습니다. **Hide logs**나 Escape는 패널을 닫고 그 버튼으로 포커스를 되돌립니다. 시작 점검이나 업데이트 메시지는 패널을 열지 않습니다.

**Launch RustFS**를 클릭합니다. 시작 중에는 **Launching…**으로 바뀝니다.

시작, 중지 또는 업데이트가 실패하면 화면에 이유와 고치는 방법이 나옵니다. 이 안내가 경로와 **Launch RustFS**(서버가 올라온 뒤에는 **Stop RustFS**)를 화면 밖으로 밀거나 로그 패널을 열지는 않습니다. 데이터 폴더가 없거나 포트가 이미 사용 중인 경우가 가장 흔합니다. RustFS는 데이터 폴더 옆 `logs` 폴더에도 자세한 내용을 씁니다. 줄 단위 출력이 필요할 때만 **View logs**를 누르세요.

### 7. 스토리지 사용하기

시작에 성공하면 경로 칸은 사라집니다. 콘솔이 켜져 있으면 런처가 기본 브라우저에서 콘솔을 한 번 엽니다. 이후 상태 확인에서는 다시 열지 않습니다. 화면에는 수신 주소와 서버가 쓰고 있는 액세스 키, 시크릿 키가 보입니다. 주소 옆의 **Open in browser**는 같은 콘솔을 엽니다. 주소를 복사하지 않고 S3 포트도 열지 않습니다. 액세스 키와 시크릿 키에는 각각 **Copy**가 있습니다. 이들은 테두리 버튼입니다. 주소와 키는 바로 쓸 수 있습니다. 확인 단계는 없습니다.

기본 호스트에서 수신 주소는 `http://0.0.0.0:9000`입니다. 이 컴퓨터에서는 S3 클라이언트를 `http://127.0.0.1:9000`으로 향하게 합니다. 같은 네트워크의 다른 기기에서는 이 컴퓨터의 주소와 포트 `9000`을 씁니다. 키는 바꾸기 전까지 `rustfsadmin`과 `rustfsadmin`입니다.

**Open Console**은 웹 콘솔을 엽니다. **View logs**는 로그를 엽니다. **Change administrator**는 다음 시작에 쓸 다른 액세스 키와 시크릿 키를 정합니다. 이 화면에서 채워진 버튼은 **Stop RustFS**뿐입니다. 서버를 멈춥니다.

S3 클라이언트를 엔드포인트로 향하게 합니다.

```bash
aws --endpoint-url http://127.0.0.1:9000 s3 mb s3://demo
aws --endpoint-url http://127.0.0.1:9000 s3 cp report.pdf s3://demo/
```

자격 증명은 화면에 나온 액세스 키와 시크릿 키, 리전은 `us-east-1`, 주소 방식은 path-style을 사용하세요. 다른 쌍을 쓰려면 **Change administrator**를 누르고 서버를 멈춘 뒤 다시 시작하세요.

콘솔에는 같은 액세스 키와 시크릿 키로 로그인합니다.

![RustFS 웹 콘솔의 버킷 목록](docs/images/rustfs-console.png)

### 8. 멈추거나, 계속 켜 두기

**Stop RustFS**를 누르면 서버가 멈추고 경로와 **Launch RustFS**로 돌아옵니다.

창을 닫아도 종료되지 않습니다. 런처는 알림 영역으로 숨고 RustFS는 계속 동작합니다. 트레이 아이콘을 클릭하면
창이 다시 나오고, 오른쪽 클릭 후 **Quit**을 고르면 서버를 멈추고 앱을 종료합니다.

### 9. 제거하기

**설정 → 앱 → 설치된 앱**에서 **RustFS Launcher**를 제거하거나, 시작 메뉴 폴더의 제거 프로그램을 사용하세요.
데이터 폴더는 그대로 남으니 필요 없으면 직접 지우면 됩니다.

## macOS

1. `.app.zip`의 압축을 풀고 **RustFS Launcher**를 **응용 프로그램**으로 끌어다 놓습니다.
2. 배포 빌드가 Apple 공증을 받지 않아 첫 실행은 차단됩니다. 앱을 오른쪽 클릭해 **열기**를 고르거나,
   터미널에서 격리 속성을 지우세요.

   ```bash
   xattr -cr "/Applications/RustFS Launcher.app"
   open "/Applications/RustFS Launcher.app"
   ```

3. 이후는 4~8단계와 같습니다. 창을 닫은 뒤에는 Dock 아이콘을 클릭하면 다시 나타납니다.

## 화면 살펴보기

![시작 전: 데이터 경로와 Launch RustFS](docs/images/launcher-ready.png)

시작 전 화면은 데이터 경로, **Browse**, **Launch RustFS**와 설정 톱니바퀴, 바닥글입니다. 호스트, 포트, 콘솔, 키는 **Settings**나 **Change administrator**에 있습니다. 로그는 열기 전까지 보이지 않습니다.

**시작 후.** 경로 칸은 사라집니다. 수신 주소는 화면에 남고 옆에 **Open in browser**가 있습니다. 액세스 키와 시크릿 키에는 각각 **Copy**가 있습니다. **Open Console**, **View logs**, **Change administrator**는 **Stop RustFS** 위에 있습니다. **Open Console**로 콘솔을 다시 열 수 있습니다.

**이미 수신 중인 프로그램.** 이 런처가 실행하지 않은 프로그램이 호스트와 포트에서 응답하면 경로 화면이 남고 그 사실을 알려 줍니다. **Settings**에서 포트를 바꾸거나 그 프로그램을 멈추세요. **Stop RustFS**는 나오지 않습니다. 이 런처가 그 프로세스를 소유하지 않기 때문입니다.

**Upgrade.** 바닥글에 현재 버전이 나옵니다. **Upgrade**는 **Check for Updates**와, 있으면 런처 및 RustFS 업데이트가 있는 패널을 엽니다. 첫 화면의 주 버튼이 아닙니다.

**로그.** **View logs**를 누르기 전까지 숨겨져 있습니다. 시작 점검과 업데이트 메시지는 패널을 열지 않습니다. **Hide logs**나 Escape가 닫습니다. *App Logs*는 런처 자신의 기록입니다. 무엇을 찾았고, 무엇을 실행했고, 왜 실패했는지가 나옵니다. *RustFS Output*은 서버가 콘솔에 출력한 내용인데, 최근 RustFS 빌드는 상세 로그를 파일에 쓰기 때문에 연 뒤에도 이 탭은 비어 있는 경우가 많습니다. Auto-scroll은 새 줄을 따라가고, Clear는 두 탭을 모두 비웁니다.

## 파일이 저장되는 곳

| 대상 | 위치 |
| --- | --- |
| 오브젝트 | 선택한 데이터 폴더와 메타데이터용 숨김 폴더 `.rustfs.sys` |
| 서버 로그 | 데이터 폴더 옆의 `logs` 폴더 |
| 런처 설정 | 앱이 알아서 저장합니다. 직접 고칠 파일은 없습니다 |
| 앱 | Windows는 `%LOCALAPPDATA%\RustFS Launcher`, macOS는 `/Applications` |

## 업데이트

바닥글의 **Upgrade**에서 **Check for Updates**를 여세요. 새 릴리스가 있으면 런처가 내려받아 서명을
확인하고 설치합니다. RustFS가 실행 중이면 먼저 확인을 구한 뒤 서버를 멈추고 런처를 다시 시작합니다.
업데이트는 내장된 RustFS 서버를 포함해 앱 전체를 교체합니다.

서명과 릴리스 절차는 [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md)에 정리되어 있습니다.

## 문제가 생겼을 때

**"Data path does not exist"** — 탐색기나 Finder에서 폴더를 먼저 만드세요. 런처가 대신 만들어 주지 않습니다.

**"Port 9000 is already in use"** — 다른 프로그램(또는 예전 RustFS)이 포트를 쓰고 있습니다. **Settings**에서 API 포트를 바꾸거나 그 프로그램을 종료하세요. 콘솔 포트도 마찬가지입니다.

**이미 수신 중이라고 나옴** — 해당 호스트와 포트에서 이미 RustFS가 응답하고 있지만, 이 런처가 실행한 것이 아닙니다. 그 프로세스를 실행한 곳에서 종료하거나 **Settings**에서 포트를 바꾸세요.

**RustFS Output이 계속 비어 있음** — **View logs**를 연 뒤에 이 탭은 조용한 경우가 많습니다. 최근 RustFS 빌드는 데이터 폴더 옆 `logs` 폴더에 기록합니다. 런처의 동작은 다른 탭인 App Logs에서 확인하세요.

**브라우저에 `AccessDenied` 같은 XML이 보임** — S3 API가 브라우저에 응답한 것일 뿐 고장이 아닙니다. 웹 화면이 필요하면 **Open Console**을 쓰고, 수신 주소는 S3 클라이언트에 넣어 주세요.

**창이 사라짐** — 닫아도 숨겨질 뿐입니다. Windows에서는 알림 영역 아이콘, macOS에서는 Dock 아이콘을
사용하세요.

**macOS에서 앱을 "열 수 없습니다"라고 함** — 위 macOS 항목의 격리 속성 명령을 실행하세요.

## 직접 빌드하기

[Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), [Trunk](https://trunkrs.dev/)(`cargo install trunk`)가
필요합니다.

```bash
./build.sh          # Windows에서는 build.bat — 플랫폼에 맞는 RustFS 바이너리를 내려받습니다
cargo tauri dev     # 핫 리로드로 실행
cargo tauri build   # 설치 파일 생성
```

풀 리퀘스트를 보내기 전에 `make pre-commit`을 실행하세요. 포맷 검사, Clippy, 프런트엔드 빌드, 테스트가 함께
돌아갑니다. 자세한 내용은 [AGENTS.md](AGENTS.md), 릴리스 워크플로는 [.github/ACTIONS.md](.github/ACTIONS.md)를
참고하세요. 편집기는 [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)와
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) 확장을 설치한
VS Code가 편합니다.

## 라이선스

Apache-2.0. [LICENSE](LICENSE)를 참고하세요.
