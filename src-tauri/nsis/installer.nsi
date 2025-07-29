; BestMe NSIS Installer Script
; Custom installer configuration

!include "MUI2.nsh"

; Define application information
!define PRODUCT_NAME "BestMe"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "BestMe Team"
!define PRODUCT_WEB_SITE "https://bestme.app"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"

; MUI Settings
!define MUI_ABORTWARNING
!define MUI_ICON "..\icons\icon.ico"
!define MUI_UNICON "..\icons\icon.ico"

; Welcome page
!insertmacro MUI_PAGE_WELCOME
; License page
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE.txt"
; Directory page
!insertmacro MUI_PAGE_DIRECTORY
; Instfiles page
!insertmacro MUI_PAGE_INSTFILES
; Finish page
!define MUI_FINISHPAGE_RUN "$INSTDIR\BestMe.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch BestMe"
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_INSTFILES

; Language files
!insertmacro MUI_LANGUAGE "English"

; Custom messages
LangString DESC_CreateDesktopShortcut ${LANG_ENGLISH} "Create desktop shortcut"
LangString DESC_CreateStartMenuShortcut ${LANG_ENGLISH} "Create Start Menu shortcuts"

; Installer sections
Section "BestMe Core Files" SEC_CORE
  SectionIn RO
  
  ; Install WebView2 if needed
  ExecWait '"$INSTDIR\resources\WebView2Setup.exe" /silent /install'
  
  ; Check for microphone permissions
  ; This is handled by the app itself on first run
SectionEnd

Section "Desktop Shortcut" SEC_DESKTOP
  CreateShortcut "$DESKTOP\BestMe.lnk" "$INSTDIR\BestMe.exe"
SectionEnd

Section "Start Menu Shortcuts" SEC_STARTMENU
  CreateDirectory "$SMPROGRAMS\BestMe"
  CreateShortcut "$SMPROGRAMS\BestMe\BestMe.lnk" "$INSTDIR\BestMe.exe"
  CreateShortcut "$SMPROGRAMS\BestMe\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

; Post-installation
Function .onInstSuccess
  ; Register URL protocol for bestme://
  WriteRegStr HKCR "bestme" "" "URL:BestMe Protocol"
  WriteRegStr HKCR "bestme" "URL Protocol" ""
  WriteRegStr HKCR "bestme\shell\open\command" "" '"$INSTDIR\BestMe.exe" "%1"'
FunctionEnd