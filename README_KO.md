# RustFS Launcher

[English](README.md) · [Deutsch](README_DE.md) · [Français](README_FR.md) · [日本語](README_JA.md) · **한국어** · [简体中文](README_ZH.md) · [हिन्दी](README_HI.md)

RustFS Launcher는 S3 호환 오브젝트 스토리지 서버인 [RustFS](https://github.com/rustfs/rustfs)를 내 컴퓨터에서
실행해 주는 작은 데스크톱 앱입니다. 폴더를 하나 고르고 버튼을 한 번 누르면 `http://127.0.0.1:9000`에 S3
엔드포인트가 생깁니다.

서버는 앱 안에 함께 들어 있습니다. 따로 설치할 것도, 터미널을 열 일도, 설정 파일을 고칠 일도 없습니다.

![RustFS가 온라인이다. Data Path는 첫 화면에 남고 로그는 숨겨져 있다](docs/images/launcher-running.png)

## 이런 데 쓸 수 있습니다

- Docker 없이 노트북에서 오브젝트 스토리지를 돌려 개발·테스트하거나 개인 백업 대상으로 사용하기.
- 아무 S3 클라이언트나 SDK를 `127.0.0.1`로 향하게 해서 클라우드의 버킷처럼 쓰기.
- 서버 상태와 사용 중인 포트를 확인하기. 로그는 직접 열기 전까지 숨겨져 있습니다.
- 원할 때 RustFS를 시작하고 멈추기. 서버가 도는 동안 런처는 트레이에 남아 있습니다.
- 런처와 그 안에 들어 있는 RustFS를 앱에서 바로 업데이트하기.

## 내려받기

[릴리스 페이지](https://github.com/rustfs/launcher/releases)에서 사용 중인 환경에 맞는 파일을 받으세요.
현재 최신 릴리스는 **1.0.6**입니다.

| 사용 환경 | 파일 |
| --- | --- |
| Windows 10 / 11 (64비트) | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS (Apple Silicon) | `rustfs-launcher-macos-aarch64-<version>.dmg` 및 `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS (Intel, x86_64) | 릴리스 1.0.6에는 없습니다 |

1.0.6의 파일 이름은 `rustfs-launcher-windows-x86_64-1.0.6-setup.exe`,
`rustfs-launcher-macos-aarch64-1.0.6.dmg`, `rustfs-launcher-macos-aarch64-1.0.6.app.zip`입니다.
`rustfs-launcher-macos-x86_64-*` 패키지는 없으므로 Intel Mac에서는 이 릴리스를 설치할 수 없습니다.

Windows on ARM에서도 동작합니다. x86_64 빌드가 에뮬레이션으로 실행됩니다. 리눅스용 런처 패키지는 없으며,
리눅스에서는 [RustFS 바이너리](https://github.com/rustfs/rustfs/releases)를 직접 실행하면 됩니다.

파일 용량이 큰 이유는 RustFS 서버 전체가 설치 파일에 들어 있기 때문입니다.

같은 릴리스에서 이름에 `-latest`가 들어간 파일은 버전 번호가 붙은 설치 파일과 바이트 단위로 같은 복사본입니다.
`rustfs-launcher-windows-x86_64-latest-setup.exe`와 `rustfs-launcher-windows-x86_64-latest.exe`는 그 Windows
설치 파일을 고정된 이름으로 둔 것입니다. macOS의 `-latest.dmg`와 `-latest.app.zip`도 같은 종류의 복사본입니다.
버전 번호가 붙은 `-setup.exe` 또는 `.dmg`를 받으세요.

### 내려받은 파일 확인

같은 릴리스 페이지의 **Assets**에서 설치 파일과 함께 `SHA256SUMS`를 받으세요. 각 줄은 SHA-256 해시와 파일 이름입니다.

Windows에서는 설치 파일이 있는 폴더에서 PowerShell을 엽니다.

```powershell
Get-FileHash .\rustfs-launcher-windows-x86_64-1.0.6-setup.exe -Algorithm SHA256
```

macOS에서는 터미널에서 실행합니다.

```bash
shasum -a 256 rustfs-launcher-macos-aarch64-1.0.6.dmg
```

해시는 `SHA256SUMS`에서 그 파일 이름 줄과 같아야 합니다. 영문 대소문자는 구분하지 않습니다. `.app.zip`을
받았다면 그 파일 이름으로 같은 방식으로 확인하세요. 이후 릴리스에서는 그 릴리스의 파일 이름과 `SHA256SUMS`를
사용하세요.

## Windows, 하나씩 따라 하기

### 1. 설치 파일 내려받기

[릴리스 페이지](https://github.com/rustfs/launcher/releases)를 열고 최신 릴리스의 **Assets**를 펼친 뒤
`rustfs-launcher-windows-x86_64-<version>-setup.exe`를 받습니다(1.0.6은
`rustfs-launcher-windows-x86_64-1.0.6-setup.exe`). 위에서 설명한 대로 `SHA256SUMS`와 비교하세요. Edge나
Chrome은 흔치 않은 설치 파일에 경고를 띄우기도 합니다. 물어보면 **유지**를 선택하세요.

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

### 5. 항목 입력하기

![설정 화면](docs/images/launcher-config.png)

| 항목 | 설명 |
| --- | --- |
| **Data Path** | 4단계에서 만든 폴더입니다. **Browse**로 고르거나 창에 폴더를 끌어다 놓으세요. 유일한 필수 항목입니다. |
| **API Port** | S3 엔드포인트 포트입니다. 다른 프로그램이 쓰고 있지 않다면 `9000` 그대로 두면 됩니다. |
| **Host** | `127.0.0.1`이면 이 컴퓨터 안에서만 쓸 수 있습니다. `0.0.0.0`으로 두면 같은 네트워크의 다른 기기에서도 접속할 수 있습니다. |
| **Console Endpoint** | RustFS 웹 콘솔을 쓰려면 켜세요. 기본값 `9001`의 별도 포트를 사용합니다. |
| **Access Key** / **Secret Key** | S3 클라이언트가 사용할 자격 증명입니다. 기본값은 `rustfsadmin` / `rustfsadmin`이며 폼의 다른 항목과 함께 저장됩니다. 루프백이 아닌 호스트에서는 둘 다 기본값에서 바꿀 때까지 런처가 시작을 거부합니다. 비워 두어도 이 공개된 쌍으로 돌아갑니다. |

입력한 값은 저장되므로 다음부터는 클릭 한 번으로 시작할 수 있습니다.

### 6. Launch 누르기

창은 **Data Path**와 **Launch RustFS**에서 열립니다. 스크롤하지 않아도 보입니다. 로그는 **Show logs**를
누르기 전까지 숨겨져 있습니다. **Hide logs**는 패널을 다시 닫습니다. 시작 점검이나 업데이트 메시지는 패널을
열지 않습니다.

**Launch RustFS**를 클릭합니다. 1~2초 안에 상태가 **Service Online / Managed by Launcher**로 바뀌고 입력
폼이 잠깁니다.

시작, 중지 또는 업데이트가 실패하면 첫 화면에 이유와 고치는 방법이 나옵니다. 로그 패널은 닫힌 채로 있습니다.
데이터 폴더가 없거나 포트가 이미 사용 중인 경우가 가장 흔합니다. RustFS는 데이터 폴더 옆 `logs` 폴더에도
자세한 내용을 씁니다. 줄 단위 출력이 필요할 때만 **Show logs**를 누르세요.

### 7. 스토리지 사용하기

서비스가 온라인인 동안에는 실행 버튼 아래의 **API**와 **Console** 카드를 클릭할 수 있습니다. **Console**은 웹 화면을 열고,
**API**는 S3 엔드포인트 자체를 엽니다. 브라우저로 열면 XML 응답만 보이는데, 이 주소는 S3 클라이언트용이기
때문입니다.

S3 클라이언트를 엔드포인트로 향하게 합니다.

```bash
aws --endpoint-url http://127.0.0.1:9000 s3 mb s3://demo
aws --endpoint-url http://127.0.0.1:9000 s3 cp report.pdf s3://demo/
```

자격 증명은 액세스 키와 시크릿 키, 리전은 `us-east-1`, 주소 방식은 path-style을 사용하세요.

콘솔에는 같은 액세스 키와 시크릿 키로 로그인합니다.

![RustFS 웹 콘솔의 버킷 목록](docs/images/rustfs-console.png)

### 8. 멈추거나, 계속 켜 두기

**Stop RustFS**를 누르면 서버가 멈추고 입력 폼의 잠금도 풀립니다.

창을 닫아도 종료되지 않습니다. 런처는 알림 영역으로 숨고 RustFS는 계속 동작합니다. 트레이 아이콘을 클릭하면
창이 다시 나오고, 오른쪽 클릭 후 **Quit**을 고르면 서버를 멈추고 앱을 종료합니다.

### 9. 제거하기

**설정 → 앱 → 설치된 앱**에서 **RustFS Launcher**를 제거하거나, 시작 메뉴 폴더의 제거 프로그램을 사용하세요.
데이터 폴더는 그대로 남으니 필요 없으면 직접 지우면 됩니다.

## macOS

1. `.dmg`를 열고 **RustFS Launcher**를 **응용 프로그램**으로 끌어다 놓습니다. `.app.zip`을 받았다면 압축을
   풀고 같은 방식으로 옮기세요.
2. 이 빌드는 공증되지 않아 첫 실행이 차단됩니다. 앱을 오른쪽 클릭하고 **열기**를 선택한 뒤 확인하세요.
   터미널에서 격리 속성을 지우는 것은 선택 사항입니다.

   ```bash
   xattr -cr "/Applications/RustFS Launcher.app"
   open "/Applications/RustFS Launcher.app"
   ```

3. 이후는 4~8단계와 같습니다. 창을 닫은 뒤에는 Dock 아이콘을 클릭하면 다시 나타납니다.

## 화면 살펴보기

![시작 전 런처 화면](docs/images/launcher-ready.png)

첫 화면은 **Data Path**, **Browse**, **Launch RustFS**이고, 제목 옆에 서비스 상태가 있습니다.
로그는 열기 전까지 보이지 않습니다.

### 언어

상단 바의 언어 버튼(**中** / **EN** / **FR** / **日**)으로 화면 언어를 바꿀 수 있습니다.

아직 언어를 고르지 않은 첫 실행에서는 운영 체제 언어를 따릅니다(`zh-CN`, `zh-Hant-TW`, `fr-CA`, `ja_JP`
등 기본 언어 태그 기준). 그 언어가 앱에 들어 있는 4개 언어 중 하나이면 그 언어를 쓰고, 아니면 영어를 씁니다.

앱 내 4개 언어는 简体中文(`zh-CN`), English(`en`), Français(`fr`), 日本語(`ja`)입니다.

언어를 고르면 바로 적용되고 기억됩니다. 그 뒤에는 운영 체제 언어가 이 선택을 바꾸지 않습니다.

한국어는 현재 앱 내 언어 선택에 포함되어 있지 않습니다. 언어를 직접 고르지 않으면 위 규칙에 따라 시스템 언어가
지원되는 4개 언어 중 하나일 때만 해당 언어를 쓰고, 그렇지 않으면 영어를 씁니다.

트레이 메뉴(**Show**, **Quit**), Windows 설치 마법사, 백엔드 오류 메시지 및 로그 내용은 영어입니다. macOS
파일은 응용 프로그램으로 끌어 놓는 디스크 이미지이며, 번역된 설치 마법사가 아닙니다. 언어 버튼은 런처 창의 언어를
바꿉니다.

**상태 배지.** 첫 번째는 서버 상태입니다. *Service Online*은 설정한 호스트와 포트에서 무언가 응답하고 있다는
뜻입니다. 두 번째는 그 프로세스의 주인이 누구인지 알려 줍니다.

| 배지 | 의미 |
| --- | --- |
| Ready to Launch | 해당 포트에서 실행 중인 것이 없습니다. |
| Managed by Launcher | 런처가 RustFS를 실행했고, 멈출 수도 있습니다. |
| Detected Externally | 포트는 응답하지만 여기서 실행한 프로세스가 아닙니다(예: 터미널에서 띄운 RustFS). 포트를 바꿀 수 있도록 폼은 편집할 수 있습니다. 이 런처가 그 프로세스를 소유하지 않으므로 정지 버튼은 비활성으로 남습니다. |

**요약 카드.** API와 Console은 포트를 보여 주고, 서비스가 온라인이면 브라우저로 열어 줍니다. Mode는 폼이
이 런처가 직접 RustFS를 실행하는 동안에만 *Locked*입니다. 외부에서 감지된 서버는 폼을 *Editable*로 둡니다.

**Version & Updates.** 런처 버전과 내장된 RustFS 버전을 보여 주고, 요청하면 새 릴리스를 확인합니다.

**로그.** 시작할 때 숨겨져 있습니다. **Show logs**로 패널을 엽니다. 시작 점검과 업데이트 메시지는 패널을 열지
않습니다. *App Logs*는 런처 자신의 기록입니다. 무엇을 찾았고, 무엇을 실행했고, 왜 실패했는지가 나옵니다.
*RustFS Output*은 서버가 콘솔에 출력한 내용인데, 최근 RustFS 빌드는 상세 로그를 파일에 쓰기 때문에 연 뒤에도 이 탭은
비어 있는 경우가 많습니다. Auto-scroll은 새 줄을 따라가고, Clear는 두 탭을 모두 비웁니다.

## 파일이 저장되는 곳

| 대상 | 위치 |
| --- | --- |
| 오브젝트 | 선택한 데이터 폴더와 메타데이터용 숨김 폴더 `.rustfs.sys` |
| 서버 로그 | 데이터 폴더 옆의 `logs` 폴더 |
| 런처 설정 | 앱이 알아서 저장합니다. 직접 고칠 파일은 없습니다 |
| 앱 | Windows는 `%LOCALAPPDATA%\RustFS Launcher`, macOS는 `/Applications` |

## 업데이트

Version & Updates 카드에서 **Check for Updates**를 누르세요. 새 릴리스가 있으면 런처가 내려받아 서명을
확인하고 설치합니다. RustFS가 실행 중이면 먼저 확인을 구한 뒤 서버를 멈추고 런처를 다시 시작합니다.
업데이트는 내장된 RustFS 서버를 포함해 앱 전체를 교체합니다.

서명과 릴리스 절차는 [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md)에 정리되어 있습니다.

## 문제가 생겼을 때

**"Data path does not exist"** — 탐색기나 Finder에서 폴더를 먼저 만드세요. 런처가 대신 만들어 주지 않습니다.

**"Port 9000 is already in use"** — 다른 프로그램(또는 예전 RustFS)이 포트를 쓰고 있습니다. API 포트를
바꾸거나 그 프로그램을 종료하세요. 콘솔 포트도 마찬가지입니다.

**Detected Externally로 표시됨** — 해당 호스트와 포트에서 이미 RustFS가 응답하고 있지만, 이 런처가 실행한
것이 아닙니다. 그 프로세스를 실행한 곳에서 종료하거나, 런처의 포트를 바꾸세요.

**RustFS Output이 계속 비어 있음** — 로그를 연 뒤에 이 탭은 조용한 경우가 많습니다. 최근 RustFS 빌드는 데이터 폴더 옆 `logs`
폴더에 기록합니다. 런처의 동작은 다른 탭인 App Logs에서 확인하세요.

**브라우저에 `AccessDenied` 같은 XML이 보임** — S3 API가 브라우저에 응답한 것일 뿐 고장이 아닙니다. 웹 화면이
필요하면 **Console** 카드를 쓰고, API 주소는 S3 클라이언트에 넣어 주세요.

**창이 사라짐** — 닫아도 숨겨질 뿐입니다. Windows에서는 알림 영역 아이콘, macOS에서는 Dock 아이콘을
사용하세요.

**macOS에서 앱을 "열 수 없습니다"라고 함** — 앱을 오른쪽 클릭하고 **열기**를 선택한 뒤 확인하세요. macOS 항목의 `xattr` 명령은 선택적 대안입니다.

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
