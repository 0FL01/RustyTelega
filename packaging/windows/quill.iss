; Quill's Windows installer (Inno Setup 6): a per-user install, no
; administrator rights, into the same folder the one-line installer uses
; (%LOCALAPPDATA%\Programs\Quill), with a Start menu shortcut and a Settings >
; Apps entry. The Chocolatey package installs this file. docs/decisions/distribution-channels.md
;
;   iscc /DVersion=2026.1103.4940 /DPackage=dist\windows\quill-windows-x86_64 ^
;        /DOutputDir=dist\windows packaging\windows\quill.iss

#ifndef Version
  #error Pass /DVersion=YEAR.MONTHDAY.COMMITS
#endif
#ifndef Package
  #error Pass /DPackage=<the unpacked quill-windows-x86_64 folder>
#endif
#ifndef OutputDir
  #define OutputDir "."
#endif

[Setup]
; Inno keeps its uninstall entry under "Quill_is1". The one-line installer's
; own "Quill" entry is removed below, so Settings > Apps lists Quill once.
AppId=Quill
AppName=Quill
AppVersion={#Version}
AppVerName=Quill {#Version}
AppPublisher=Quill (open source)
AppPublisherURL=https://github.com/shinycake/quill
AppSupportURL=https://github.com/shinycake/quill/issues
AppUpdatesURL=https://github.com/shinycake/quill/releases
AppCopyright=MIT License
VersionInfoVersion={#Version}
VersionInfoProductName=Quill
VersionInfoDescription=Quill installer
PrivilegesRequired=lowest
DefaultDirName={userpf}\Quill
DisableDirPage=yes
DisableProgramGroupPage=yes
DisableReadyPage=yes
UsePreviousAppDir=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
SetupIconFile=..\..\assets\icons\Quill.ico
UninstallDisplayIcon={app}\quill.exe
UninstallDisplayName=Quill
; Quill updates itself in place file by file, which leaves unins000.exe alone.
CloseApplications=force
RestartApplications=no
Compression=lzma2/ultra64
SolidCompression=yes
OutputDir={#OutputDir}
OutputBaseFilename=quill-windows-x86_64-setup

[Tasks]
Name: desktopicon; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#Package}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
; The AppUserModelID matches the one Quill's notifications use (src/main.rs).
Name: "{userprograms}\Quill"; Filename: "{app}\quill.exe"; WorkingDir: "{app}"; AppUserModelID: "org.shinycake.quill"; Comment: "Quill, a Telegram client"
Name: "{userdesktop}\Quill"; Filename: "{app}\quill.exe"; WorkingDir: "{app}"; AppUserModelID: "org.shinycake.quill"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Uninstall\Quill"; ValueType: none; Flags: deletekey
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\org.shinycake.quill"; ValueType: none; Flags: uninsdeletekey

[Run]
Filename: "{app}\quill.exe"; Description: "{cm:LaunchProgram,Quill}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\quill.exe"; Parameters: "--quit"; Flags: runhidden waituntilterminated; RunOnceId: "QuitQuill"

[UninstallDelete]
; Files Quill's own updates leave behind (*.old, the update staging folder).
Type: filesandordirs; Name: "{app}"

[Code]
// Quill writes the tg: link handler and the start-at-login entry itself
// (Settings); remove them only when they still point at this install.
procedure RemoveIfOurs(const Key, Name: String);
var
  Value: String;
begin
  if RegQueryStringValue(HKCU, Key, Name, Value) and
     (Pos(Lowercase(ExpandConstant('{app}')), Lowercase(Value)) > 0) then
  begin
    if Name = '' then
      RegDeleteKeyIncludingSubkeys(HKCU, 'Software\Classes\tg')
    else
      RegDeleteValue(HKCU, Key, Name);
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemoveIfOurs('Software\Classes\tg\shell\open\command', '');
    RemoveIfOurs('Software\Microsoft\Windows\CurrentVersion\Run', 'Quill');
  end;
end;
