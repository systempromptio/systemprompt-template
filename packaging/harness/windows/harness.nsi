; harness.nsi: the per-user Windows installer of the harness desktop app.
;
; One script serves every brand and channel; package.ps1 passes the identity of the build it packs, taken from the
; same values as crates/app/brand (sp-brand), so each (brand, channel) installs into its own directory, under its own
; uninstall entry, with its own Start menu shortcut and URL scheme, and never replaces another.
;
; Required defines (makensis /D<name>=<value>):
;   STAGE        directory holding harness.exe (and the icon when APP_ICON is defined)
;   OUTFILE      the installer to write
;   VERSION      product version, MAJOR.MINOR.PATCH (sp-brand's VERSION)
;   PRODUCT      display name, e.g. "systemprompt Harness Canary"
;   VENDOR       publisher shown in Apps and features
;   APP_ID       app id, e.g. "io.systemprompt.harness.canary"; keys the uninstall entry and the AppUserModelID
;   SCHEME       URL scheme the app answers, e.g. "sp-harness-canary"
;   INSTALL_NAME directory name under %LOCALAPPDATA%\Programs
;
; Optional define:
;   APP_ICON     file name of the icon in STAGE; without it the installer keeps NSIS's own icon, installs no icon
;                file and points the shortcut at the executable
;
; Command line: /S silent, /D=<dir> another install directory, /FILESONLY copies and removes files only (no
; shortcut, no registry), which is what package.ps1 -TestInstall uses so a test leaves the machine untouched.
;
; User data is not under the install directory: it lives in %LOCALAPPDATA%\<vendor>\<product> (sp-brand
; data_root) and the uninstaller never touches it, so a reinstall finds the same accounts, sessions and settings.
;
; Icon pipeline: app.ico, when present, is staged by package.ps1 from crates/app/desktop/assets/icons/icon.ico, the
; same file the desktop crate embeds as icon resource 1 (window and taskbar). It should hold 16, 24, 32, 48, 64 and 256 px images;
; to change it, export those sizes from the brand's square master SVG as PNG and join them into one .ico (for example
; `magick icon-16.png icon-24.png icon-32.png icon-48.png icon-64.png icon-256.png icon.ico`). A per-brand or
; per-channel tint goes in a sibling file (icon-<brand>-<channel>.ico) that package.ps1 prefers when it exists.

Unicode true
ManifestDPIAware true
RequestExecutionLevel user
SetCompressor /SOLID lzma

!macro RequireDefine NAME
  !ifndef ${NAME}
    !error "harness.nsi: define ${NAME} (see the header of this script)"
  !endif
!macroend
!insertmacro RequireDefine STAGE
!insertmacro RequireDefine OUTFILE
!insertmacro RequireDefine VERSION
!insertmacro RequireDefine PRODUCT
!insertmacro RequireDefine VENDOR
!insertmacro RequireDefine APP_ID
!insertmacro RequireDefine SCHEME
!insertmacro RequireDefine INSTALL_NAME

!define EXE "harness.exe"
!ifdef APP_ICON
  !define SHORTCUT_ICON "$INSTDIR\${APP_ICON}"
!else
  !define SHORTCUT_ICON "$INSTDIR\${EXE}"
!endif
!define REG_UNINSTALL "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_ID}"
!define REG_SCHEME "Software\Classes\${SCHEME}"
!define REG_AUMID "Software\Classes\AppUserModelId\${APP_ID}"
!define SCHEME_COMMAND '"$INSTDIR\${EXE}" "%1"'
!define SHORTCUT "$SMPROGRAMS\${PRODUCT}.lnk"

!include "FileFunc.nsh"
!include "LogicLib.nsh"

Name "${PRODUCT}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\${INSTALL_NAME}"
InstallDirRegKey HKCU "${REG_UNINSTALL}" "InstallLocation"
!ifdef APP_ICON
  Icon "${STAGE}\${APP_ICON}"
  UninstallIcon "${STAGE}\${APP_ICON}"
!endif
BrandingText "${PRODUCT} ${VERSION}"
ShowInstDetails nevershow
ShowUninstDetails nevershow

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${PRODUCT}"
VIAddVersionKey "CompanyName" "${VENDOR}"
VIAddVersionKey "FileDescription" "${PRODUCT} setup"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"

Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Var FilesOnly

!macro ReadFilesOnly
  StrCpy $FilesOnly 0
  ${GetParameters} $R0
  ClearErrors
  ${GetOptions} $R0 "/FILESONLY" $R1
  ${IfNot} ${Errors}
    StrCpy $FilesOnly 1
  ${EndIf}
!macroend

Function .onInit
  !insertmacro ReadFilesOnly
FunctionEnd

Function un.onInit
  !insertmacro ReadFilesOnly
FunctionEnd

Section "Install"
  SetShellVarContext current
  SetOutPath "$INSTDIR"
  File "${STAGE}\${EXE}"
  !ifdef APP_ICON
    File "${STAGE}\${APP_ICON}"
  !endif
  WriteUninstaller "$INSTDIR\uninstall.exe"
  ${If} $FilesOnly == 1
    Return
  ${EndIf}

  ; The shortcut starts the app in the user's profile directory, so a first session opens somewhere familiar.
  SetOutPath "$PROFILE"
  CreateShortcut "${SHORTCUT}" "$INSTDIR\${EXE}" "" "${SHORTCUT_ICON}" 0

  ; <scheme>:// links (sign-in callbacks, deep links) open this install. Each channel has its own scheme, so this
  ; never takes over another build's links.
  WriteRegStr HKCU "${REG_SCHEME}" "" "URL:${PRODUCT}"
  WriteRegStr HKCU "${REG_SCHEME}" "URL Protocol" ""
  WriteRegStr HKCU "${REG_SCHEME}\DefaultIcon" "" "$INSTDIR\${EXE},0"
  WriteRegStr HKCU "${REG_SCHEME}\shell\open\command" "" '${SCHEME_COMMAND}'

  ; Name and icon Windows shows on the app's notifications.
  WriteRegStr HKCU "${REG_AUMID}" "DisplayName" "${PRODUCT}"
  !ifdef APP_ICON
    WriteRegStr HKCU "${REG_AUMID}" "IconUri" "$INSTDIR\${APP_ICON}"
  !endif

  WriteRegStr HKCU "${REG_UNINSTALL}" "DisplayName" "${PRODUCT}"
  WriteRegStr HKCU "${REG_UNINSTALL}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${REG_UNINSTALL}" "Publisher" "${VENDOR}"
  WriteRegStr HKCU "${REG_UNINSTALL}" "DisplayIcon" "$INSTDIR\${EXE},0"
  WriteRegStr HKCU "${REG_UNINSTALL}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${REG_UNINSTALL}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${REG_UNINSTALL}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKCU "${REG_UNINSTALL}" "NoModify" 1
  WriteRegDWORD HKCU "${REG_UNINSTALL}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $R0 $R1 $R2
  IntFmt $R0 "0x%08X" $R0
  WriteRegDWORD HKCU "${REG_UNINSTALL}" "EstimatedSize" "$R0"
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  Delete "$INSTDIR\${EXE}"
  !ifdef APP_ICON
    Delete "$INSTDIR\${APP_ICON}"
  !endif
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  ${If} $FilesOnly == 1
    Return
  ${EndIf}

  Delete "${SHORTCUT}"

  ; Only a scheme registration that still points at this install is removed.
  ReadRegStr $R0 HKCU "${REG_SCHEME}\shell\open\command" ""
  ${If} $R0 == '${SCHEME_COMMAND}'
    DeleteRegKey HKCU "${REG_SCHEME}"
  ${EndIf}
  DeleteRegKey HKCU "${REG_AUMID}"
  DeleteRegKey HKCU "${REG_UNINSTALL}"
SectionEnd
