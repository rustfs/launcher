# RustFS Launcher design

The visual source of truth is the RustFS Design System: <https://design.rustfs.com/en/agents/> (also the typography, color, logo, and web pages on that site). Tokens below mirror that system for the Launcher. When this file and the design site disagree, the design site wins.

Do not introduce one-off colors, radii, fonts, gradients, or shadows. Product UI stays handwritten CSS on these tokens. Do not add a component library or a design-system runtime package.

## Color

| Role | Value | Use |
| --- | --- | --- |
| Signal Blue | `#0062FF` | Primary actions, brand, links. Hover `#014FDD`. |
| Oxide | `#FF5E14` | Parity only. At most 10% of a screen, never on a Signal surface, never for errors or warnings. |
| Ink | `#0B0E14` | Primary text. |
| Paper | `#F4F6F9` | Page background. Cards sit on white `#FFFFFF`. |
| Warning | `#A06604` | Warnings. |
| Error | `#C51B32` | Errors and destructive actions. |
| Success | `#037E3F` | Success text. |
| Status online | `#02A855` | Online marker fill. Offline marker fill is `#E42E41`. |

Neutrals, borders, and the focus ring come from the same scale: border `#DCE0E6`, strong/input border `#C4C9D2`, muted text `#5A616D`, focus ring `#0062FF`.

## Typography

- Display, headlines, and large numerals: Archivo, width axis 112% (headlines) or 125% (display and numerals), weights 700–800. Summary-card values stay Geist 16/600 so short words such as "Disabled" fit the column.
- UI body, titles, and buttons: Geist 400 / 500 / 600.
- Labels that are uppercase, plus code, paths, directory lists, and log lines: Geist Mono. Uppercase labels use `letter-spacing: 0.08em`.
- Paths, directory lists, and log lines stay Geist Mono and `dir="ltr"` so a later RTL locale does not reorder them.

The official Archivo release has no Cyrillic. Russian characters in an Archivo style fall through to Geist, which carries Cyrillic. Do not add a second Latin face to paper over that.

## Corners, elevation, focus

- Every corner is square (`border-radius: 0`). No pills, no circular status dots, no 4–22px radii.
- Status markers are 10px squares.
- Surfaces separate with a 1px border. No glow, gradient fill, glass, or backdrop blur.
- The only shadow is the overlay shadow, on toasts: `0 24px 48px -16px #0B0E1447`.
- Keyboard focus is a 2px Signal ring with a 2px offset (`:focus-visible` only).

## Buttons

Existing classes map onto the system. They are 40px tall, square, and verb-first. Do not add a second filled primary.

| Control | Class | Variant |
| --- | --- | --- |
| Launch | `.launch-btn` | Primary, Signal Blue `#0062FF`, hover `#014FDD` |
| Stop | `.launch-btn.stop-btn` | Destructive, error `#C51B32` |
| Browse | `.browse-btn` | Outline |
| Check for Updates | `.update-check-btn` | Outline |
| Update Launcher / Update RustFS | `.update-install-btn` | Outline |

## Font packaging

Fonts are local `woff2` files in `public/fonts/`. `@font-face` lives in `fonts.css`. Trunk copies `public/` into the app bundle; nothing is loaded from a CDN (`default-src 'self'`).

Packaged cuts, and only these:

- `Archivo.woff2` — variable, `wdth` 112–125, `wght` 700–800. Latin, Latin Extended (including Vietnamese), no italic.
- `Geist.woff2` — variable, `wght` 400–600. Latin, Latin Extended, Cyrillic. No italic.
- `GeistMono.woff2` — variable, `wght` 400–500. Latin, Latin Extended, Cyrillic. No italic.

CJK and Arabic are not packaged. `html` `lang`, and `lang` on a subtree, switch the sans and display stacks. Geist, Geist Mono, and Archivo are not first in those stacks.

- `zh`, `zh-Hans`, `zh-CN`: `HarmonyOS Sans SC`, `Source Han Sans SC`, `Noto Sans SC`, `PingFang SC`, `Microsoft YaHei`, `sans-serif`
- `zh-Hant`, `zh-TW`: `HarmonyOS Sans TC`, `Source Han Sans TC`, `Noto Sans TC`, `PingFang TC`, `Microsoft JhengHei`, `sans-serif`
- `ja`: `Hiragino Sans`, `Yu Gothic`, `Noto Sans JP`, `sans-serif`
- `ko`: `Apple SD Gothic Neo`, `Malgun Gothic`, `Noto Sans KR`, `sans-serif`
- `ar`: `Segoe UI`, `Tahoma`, `Noto Naskh Arabic`, `Noto Sans Arabic`, `sans-serif`

## Marks

`public/rustfs-wordmark.svg` and `public/rustfs-symbol.svg` are the official Signal files from the design site. The wordmark is the in-app brand. The Flow S is not placed beside it. `public/rustfs-stripe.svg` is the EC 4+2 stripe signature (four Signal squares, two Oxide squares, gap 32% of the square), used once in the header. Do not retype the wordmark, and do not replace the OS icon packs in this UI pass.
