; RuneDesk installer (Inno Setup 6).
;
; RuneDesk installs itself: `runedesk.exe --silent-install` copies its own
; folder to Program Files\RuneDesk, registers the service, shortcuts and the
; Add/Remove Programs entry (with its own uninstaller). This wrapper only
; unpacks the files to a temp folder and runs that, so it registers no
; uninstaller of its own.
;
;   iscc /DAppVersion=0.1.0 /DSourceDir=<dir with runedesk.exe> /DOutputDir=dist res\windows\runedesk.iss

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SourceDir
  #define SourceDir "..\..\target\release"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\dist"
#endif

[Setup]
AppId={{6C5E1F8A-2B7D-4E0C-9A51-5D3F0B8E7A21}
AppName=RuneDesk
AppVersion={#AppVersion}
AppPublisher=runetale
AppPublisherURL=https://runetale.com
DefaultDirName={autopf}\RuneDesk
DisableDirPage=yes
DisableProgramGroupPage=yes
DisableReadyPage=yes
Uninstallable=no
CreateUninstallRegKey=no
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutputDir}
OutputBaseFilename=RuneDesk-{#AppVersion}-windows-x64-setup
SetupIconFile=..\icon.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=force

[Files]
Source: "{#SourceDir}\runedesk.exe"; DestDir: "{tmp}\runedesk"; Flags: ignoreversion
Source: "{#SourceDir}\sciter.dll"; DestDir: "{tmp}\runedesk"; Flags: ignoreversion

[Run]
Filename: "{tmp}\runedesk\runedesk.exe"; Parameters: "--silent-install"; StatusMsg: "Installing RuneDesk..."; Flags: runhidden waituntilterminated
Filename: "{autopf}\RuneDesk\runedesk.exe"; Description: "Launch RuneDesk"; Flags: postinstall nowait skipifsilent runasoriginaluser
