#ifndef AppVersion
  #error AppVersion must be supplied with /DAppVersion
#endif

[Setup]
AppId={{BBDB5DC2-8D44-4D5C-AAD6-6028C4507B0A}
AppName=Windows Mouse Fix
AppVersion={#AppVersion}
AppPublisher=Windows Mouse Fix contributors
AppPublisherURL=https://github.com/jeckleee/windows-mouse-fix
DefaultDirName={autopf}\Windows Mouse Fix
DisableDirPage=yes
UsePreviousAppDir=no
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
AppMutex=Local\WindowsMouseFix.Instance
CloseApplications=no
RestartApplications=no
UninstallDisplayIcon={app}\windows-mouse-fix.exe
SetupIconFile=..\assets\mouse.ico
LicenseFile=..\LICENSE
OutputDir=..\dist\installer
OutputBaseFilename=windows-mouse-fix-{#AppVersion}-windows-x64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "..\dist\package\windows-mouse-fix.exe"; DestDir: "{app}"; Flags: ignoreversion; AfterInstall: SetExecutableIntegrity
Source: "..\dist\package\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\package\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\package\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\package\THIRD_PARTY_LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\package\assets\mouse.png"; DestDir: "{app}\assets"; Flags: ignoreversion

[Icons]
Name: "{commonprograms}\Windows Mouse Fix"; Filename: "{app}\windows-mouse-fix.exe"; WorkingDir: "{app}"
Name: "{commondesktop}\Windows Mouse Fix"; Filename: "{app}\windows-mouse-fix.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Code]
function SetMediumIntegrity(const Path, Level: String): Boolean;
var
  ExitCode: Integer;
begin
  Result := Exec(ExpandConstant('{sys}\icacls.exe'),
    '"' + Path + '" /setintegritylevel ' + Level, '', SW_HIDE,
    ewWaitUntilTerminated, ExitCode);
  if Result then
    Result := ExitCode = 0;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Result := '';
  if not ForceDirectories(ExpandConstant('{app}')) then
    Result := 'Unable to create the application directory.';
  if Result = '' then
    if not SetMediumIntegrity(ExpandConstant('{app}'), '(OI)(CI)M') then
      Result := 'Unable to configure application directory permissions. Installation cannot continue.';
end;

procedure SetExecutableIntegrity;
begin
  if not SetMediumIntegrity(ExpandConstant('{app}\windows-mouse-fix.exe'), 'M') then
    RaiseException('Unable to configure application permissions. Installation cannot continue.');
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Command: String;
begin
  // Remove only this installation's startup entry for the uninstalling user.
  // User configuration is intentionally retained for a future reinstall.
  if CurUninstallStep = usUninstall then
    if RegQueryStringValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Run',
      'WindowsMouseFix', Command) then
      if CompareText(Command, '"' + ExpandConstant('{app}\windows-mouse-fix.exe') + '" --startup') = 0 then
        RegDeleteValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Run', 'WindowsMouseFix');
end;
