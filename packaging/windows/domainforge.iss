; Per-user installer for DomainForge (Inno Setup 6).
; Unsigned: no Authenticode certificate is used.
; Driven by build.ps1, which defines AppVersion, BinDir, LicenseFile,
; ExampleFile, OutDir, and SetupBase.

#ifndef AppVersion
  #error AppVersion must be defined with /DAppVersion=<version>
#endif
#ifndef BinDir
  #error BinDir must be defined with /DBinDir=<directory containing domainforge.exe>
#endif
#ifndef LicenseFile
  #error LicenseFile must be defined with /DLicenseFile=<path to LICENSE>
#endif
#ifndef ExampleFile
  #error ExampleFile must be defined with /DExampleFile=<path to basic.sea>
#endif
#ifndef OutDir
  #define OutDir "."
#endif
#ifndef SetupBase
  #define SetupBase "domainforge-" + AppVersion + "-x86_64-pc-windows-msvc-setup"
#endif

[Setup]
AppId={{6F1C2A64-9D3B-4E7A-8C21-5B0D7E4F9A13}
AppName=DomainForge
AppVersion={#AppVersion}
AppVerName=DomainForge {#AppVersion}
AppPublisher=GodSpeedAI
AppPublisherURL=https://github.com/GodSpeedAI/DomainForge
DefaultDirName={localappdata}\Programs\DomainForge
DefaultGroupName=DomainForge
DisableProgramGroupPage=yes
DisableDirPage=yes
DisableReadyPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
OutputDir={#OutDir}
OutputBaseFilename={#SetupBase}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName=DomainForge {#AppVersion}
LicenseFile={#LicenseFile}

[Files]
Source: "{#BinDir}\domainforge.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#LicenseFile}"; DestDir: "{app}"; DestName: "LICENSE"; Flags: ignoreversion
Source: "{#ExampleFile}"; DestDir: "{app}\examples"; DestName: "basic.sea"; Flags: ignoreversion

[Code]
// Removes every PATH entry equal (case-insensitively) to Entry and returns the rest.
function RemovePathEntry(const PathValue, Entry: string): string;
var
  Remaining, Item: string;
  P: Integer;
begin
  Result := '';
  Remaining := PathValue;
  while Remaining <> '' do
  begin
    P := Pos(';', Remaining);
    if P = 0 then
    begin
      Item := Remaining;
      Remaining := '';
    end
    else
    begin
      Item := Copy(Remaining, 1, P - 1);
      Remaining := Copy(Remaining, P + 1, Length(Remaining) - P);
    end;
    if (Item <> '') and (CompareText(Item, Entry) <> 0) then
    begin
      if Result <> '' then
        Result := Result + ';';
      Result := Result + Item;
    end;
  end;
end;

// Appends {app} to the per-user PATH (HKCU\Environment), once.
procedure AddAppToUserPath();
var
  PathValue, AppDir, NewValue: string;
begin
  AppDir := ExpandConstant('{app}');
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', PathValue) then
    PathValue := '';
  NewValue := RemovePathEntry(PathValue, AppDir);
  if NewValue <> '' then
    NewValue := NewValue + ';';
  NewValue := NewValue + AppDir;
  RegWriteExpandStringValue(HKCU, 'Environment', 'Path', NewValue);
end;

// Removes {app} from the per-user PATH on uninstall.
procedure RemoveAppFromUserPath();
var
  PathValue, NewValue: string;
begin
  if RegQueryStringValue(HKCU, 'Environment', 'Path', PathValue) then
  begin
    NewValue := RemovePathEntry(PathValue, ExpandConstant('{app}'));
    if NewValue <> PathValue then
      RegWriteExpandStringValue(HKCU, 'Environment', 'Path', NewValue);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    AddAppToUserPath();
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemoveAppFromUserPath();
end;
