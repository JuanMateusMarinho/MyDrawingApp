!include "MUI2.nsh"

Name "DigitalCanvas"
OutFile "DigitalCanvas-Setup.exe"
InstallDir "$PROGRAMFILES64\DigitalCanvas"
InstallDirRegKey HKCU "Software\DigitalCanvas" "InstallDir"
RequestExecutionLevel admin
CRCCheck on

!define MUI_ABORTWARNING
!define MUI_ICON "assets/icons/app.ico"
!define MUI_UNICON "assets/icons/app.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "Main" SecMain
    SetOutPath "$INSTDIR"
    
    # Main executable
    File "target/release/digital_canvas.exe"
    
    # Assets
    File /r "assets"
    
    # Default settings
    File "src/settings/default_settings.toml"
    
    # Registry
    WriteRegStr HKCU "Software\DigitalCanvas" "InstallDir" "$INSTDIR"
    WriteRegStr HKCU "Software\DigitalCanvas" "Version" "1.0.0"
    
    # Start Menu shortcuts
    CreateDirectory "$SMPROGRAMS\DigitalCanvas"
    CreateShortCut "$SMPROGRAMS\DigitalCanvas\DigitalCanvas.lnk" "$INSTDIR\digital_canvas.exe" "" "$INSTDIR\assets\icons\app.ico" 0
    CreateShortCut "$SMPROGRAMS\DigitalCanvas\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    
    # Desktop shortcut
    CreateShortCut "$DESKTOP\DigitalCanvas.lnk" "$INSTDIR\digital_canvas.exe" "" "$INSTDIR\assets\icons\app.ico" 0
    
    # Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    # File associations
    WriteRegStr HKCR ".dcanvas" "" "DigitalCanvas.Project"
    WriteRegStr HKCR "DigitalCanvas.Project" "" "DigitalCanvas Project"
    WriteRegStr HKCR "DigitalCanvas.Project\DefaultIcon" "" "$INSTDIR\assets\icons\project.ico"
    WriteRegStr HKCR "DigitalCanvas.Project\shell\open\command" "" '"$INSTDIR\digital_canvas.exe" "%1"'
    
    WriteRegStr HKCR ".dcv" "" "DigitalCanvas.Project"
    WriteRegStr HKCR "DigitalCanvas.Project\shell\open\command" "" '"$INSTDIR\digital_canvas.exe" "%1"'
SectionEnd

Section "Uninstall"
    # Remove files
    Delete "$INSTDIR\*.*"
    RMDir /r "$INSTDIR\assets"
    RMDir "$INSTDIR"
    
    # Remove shortcuts
    Delete "$SMPROGRAMS\DigitalCanvas\*.*"
    RMDir "$SMPROGRAMS\DigitalCanvas"
    Delete "$DESKTOP\DigitalCanvas.lnk"
    
    # Registry
    DeleteRegKey HKCU "Software\DigitalCanvas"
    
    # File associations
    DeleteRegKey HKCR ".dcanvas"
    DeleteRegKey HKCR ".dcv"
    DeleteRegKey HKCR "DigitalCanvas.Project"
    
    # Uninstaller
    Delete "$INSTDIR\Uninstall.exe"
SectionEnd

Function .onInit
    # Check for previous installation
    ReadRegStr $0 HKCU "Software\DigitalCanvas" "InstallDir"
    StrCmp $0 "" 0 +3
    MessageBox MB_YESNO "DigitalCanvas is already installed at $0.$\n$\nDo you want to reinstall?" IDYES +2
    Abort
FunctionEnd