; The launcher starts RustFS as a child. Closing only the launcher executable
; leaves that child running, and Windows then refuses to replace the bundled
; binary. Ask (or, in a passive update, just stop) before copying files.
!macro NSIS_HOOK_PREINSTALL
  !insertmacro CheckIfAppIsRunning "rustfs-windows-x86_64.exe" "RustFS"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro CheckIfAppIsRunning "rustfs-windows-x86_64.exe" "RustFS"
!macroend
