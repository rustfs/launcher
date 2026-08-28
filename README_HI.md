# RustFS Launcher

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README_ZH.md">简体中文</a> |
  <a href="./README_DE.md">Deutsch</a> |
  <a href="./README_FR.md">Français</a> |
  <a href="./README_JA.md">日本語</a> |
  <a href="./README_KO.md">한국어</a> |
  हिन्दी
</p>

RustFS Launcher एक छोटा डेस्कटॉप ऐप है, जिससे आप अपने कंप्यूटर पर [RustFS](https://github.com/rustfs/rustfs) चला सकते हैं। एक फ़ोल्डर चुनें, Launch दबाएँ, और इसी मशीन पर S3-compatible API आ जाता है। वेब console वैकल्पिक है।

RustFS बाइनरी इंस्टॉलर के अंदर ही है। अलग से डाउनलोड करने की ज़रूरत नहीं।

ऐप की स्क्रीन अंग्रेज़ी में है। ये पेज दूसरी भाषाओं में भी हैं।

![शुरू करने से पहले Launcher विंडो](docs/images/launcher-ready.png)

## डाउनलोड

बिल्ड [Releases](https://github.com/rustfs/launcher/releases) पर हैं।

| कंप्यूटर | फ़ाइल |
| --- | --- |
| Windows 10/11, 64-bit | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS, Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS, Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

**Source code** न लें। वो Git का सोर्स है, ऐप नहीं।

अगर नए tag में सिर्फ़ सोर्स zip हैं, Assets में `setup.exe` या `.app.zip` वाला release ढूँढें।

ARM Windows x86_64 इंस्टॉलर emulation से चलाता है। Linux पैकेज अभी नहीं है।

## Windows: डाउनलोड, इंस्टॉल, स्टार्ट

ज़्यादातर लोग यही रास्ता अपनाते हैं।

### 1. इंस्टॉलर लें

1. [github.com/rustfs/launcher/releases](https://github.com/rustfs/launcher/releases) खोलें।
2. जिस release में इंस्टॉलर हो, वो चुनें।
3. **Assets** से `rustfs-launcher-windows-x86_64-…-setup.exe` डाउनलोड करें।

### 2. इंस्टॉल करें

`.exe` पर डबल-क्लिक करें।

Windows **Windows protected your PC** दिखा सकता है। इंस्टॉलर अभी Authenticode-signed नहीं है, इसलिए SmartScreen रोकता है। फ़ाइल इसी repo के Releases से आई हो तो **More info**, फिर **Run anyway**।

NSIS विज़ार्ड पूरा करें। Start मेनू में **RustFS Launcher** आ जाता है।

### 3. पहले डेटा फ़ोल्डर बनाएँ

Launcher ये फ़ोल्डर खुद नहीं बनाता। Explorer में खाली फ़ोल्डर बनाएँ, जैसे `D:\rustfs\data`। उसमें और फ़ाइलें न रखें।

PowerShell:

```powershell
New-Item -ItemType Directory -Force -Path D:\rustfs\data
```

### 4. फ़ॉर्म भरें

Start मेनू से **RustFS Launcher** खोलें।

- **Data Path** — **Browse** दबाएँ, या फ़ोल्डर विंडो पर खींचें। ये ज़रूरी है, और फ़ोल्डर पहले से मौजूद होना चाहिए।
- **API Port** — खाली हो तो `9000`।
- **Host** — `127.0.0.1` रहने दें, तो सिर्फ़ यही कंप्यूटर जुड़ सकता है।
- **Console Endpoint** — डिफ़ॉल्ट बंद है। वेब UI चाहिए तो चालू करें। पोर्ट आम तौर पर `9001`। API और console एक ही पोर्ट नहीं हो सकते।
- **Access Key / Secret Key** — पहले से `rustfsadmin` / `rustfsadmin` भरा है। अगर और लोग ये मशीन इस्तेमाल करते हैं तो बदल दें।

फिर **Launch RustFS**।

### 5. चलने के बाद

स्टेटस **Service Online** हो जाता है, फ़ॉर्म लॉक, बटन **Stop RustFS**।

![सफल स्टार्ट के बाद Launcher](docs/images/launcher-running.png)

**API** कार्ड `http://127.0.0.1:9000` खोलता है, **Console** `http://127.0.0.1:9001`। Console में वही keys डालें जो आपने सेट कीं।

S3 क्लाइंट path-style इस्तेमाल करें, पोर्ट 9000।

सर्वर लॉग डेटा फ़ोल्डर के अंदर नहीं, उसके बगल में बनते हैं। `D:\rustfs\data` हो तो `D:\rustfs\logs`।

## विंडो में क्या है

बाएँ सेटअप है, दाएँ लॉग।

**App Logs** launcher की बात है। **RustFS Output** सर्वर प्रोसेस की। कुछ टूटे तो पहले **RustFS Output** देखें।

अपडेट वाला हिस्सा GitHub से नया launcher चेक कर सकता है। वो दो लेबल अभी चीनी में हैं (`版本与更新`, `检查更新`)। अपडेट पूरा ऐप बदल देता है, bundled RustFS सहित। डेटा फ़ोल्डर नहीं छूता। RustFS चल रहा हो तो रोकने से पहले पूछता है।

विंडो बंद करने से ऐप बंद नहीं होता। Tray में छिप जाता है, RustFS चलता रहता है। Tray आइकन पर राइट-क्लिक → **Quit** से सर्वर रुकता है और ऐप बंद होता है। **Show**, या लेफ्ट-क्लिक, विंडो वापस लाता है। ऐप दोबारा खोलने से वही पहले से खुली विंडो आगे आती है।

## macOS

अपने चिप वाला zip लें, निकालें, `RustFS Launcher.app` को Applications में रखें।

GitHub बिल्ड notarize नहीं हैं। पहली बार खोलते समय रुक सकता है:

```bash
xattr -cr "/Applications/RustFS Launcher.app"
open "/Applications/RustFS Launcher.app"
```

या ऐप पर Control-क्लिक करके **Open**।

## अगर अटक जाए

**इंस्टॉलर ब्लॉक हो।** इसी repo से डाउनलोड किया है, ये पक्का करें। Properties में **Unblock** उसके बाद ठीक है।

**Data path is required / does not exist.** पहले फ़ोल्डर बनाएँ, फिर Browse।

**पोर्ट पहले से लगा है।** दूसरा नंबर लें, या 9000 / 9001 वाला प्रोग्राम बंद करें।

**Console नहीं खुलता।** Console Endpoint ऑन होना चाहिए। पोर्ट 9001 है, 7001 नहीं।

**Detected Externally.** उस API पोर्ट पर कोई और प्रोसेस सुन रही है, इस launcher ने नहीं चलाई। वो प्रोसेस बंद करें, या पोर्ट बदलें। **Stop RustFS** सिर्फ़ उसी प्रोसेस को रोकता है जिसे इस ऐप ने शुरू किया।

**Launch के तुरंत बाद बंद।** दोनों लॉग टैब देखें। अक्सर: फ़ोल्डर नहीं, लिखने की अनुमति नहीं, पोर्ट टकराव।

## ये लोकल नोड है

Launcher RustFS को डेस्कटॉप प्रोसेस की तरह चलाता है, Windows service की तरह नहीं। आज़माने और डेवलपमेंट के लिए काफी है। प्रोडक्शन क्लस्टर Linux पर हैं: [docs.rustfs.com/en/installation](https://docs.rustfs.com/en/installation)।

लंबा Windows गाइड: [Install RustFS on Windows](https://docs.rustfs.com/en/installation/windows)।

## सोर्स से बिल्ड

[CONTRIBUTING.md](CONTRIBUTING.md) देखें।

## लाइसेंस

[Apache License 2.0](LICENSE)।
