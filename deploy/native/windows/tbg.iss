; Per-user installer from the same immutable portable package inventory.
#ifndef PackageDir
#error PackageDir is required
#endif
#ifndef BuildVersion
#error BuildVersion is required
#endif
#ifndef OutputDir
#error OutputDir is required
#endif
[Setup]
AppId={{B580602D-A93E-4B46-A3A3-450649AC42B5}
AppName=TBG
AppVersion={#BuildVersion}
AppPublisher=Alexandre Zanata
AppPublisherURL=https://github.com/AlexandreZanata
AppSupportURL=https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds
DefaultDirName={localappdata}\Programs\TBG
DefaultGroupName=TBG
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.22000
WizardStyle=modern
SetupIconFile={#PackageDir}\assets\branding\tbg.ico
UninstallDisplayIcon={app}\tbg-client.exe
OutputDir={#OutputDir}
OutputBaseFilename=tupi-battle-grounds-{#BuildVersion}-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
CloseApplications=no
RestartApplications=no
AlwaysRestart=no
DisableProgramGroupPage=yes
[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
[Files]
Source: "{#PackageDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
[Icons]
Name: "{userprograms}\TBG"; Filename: "{app}\tbg-client.exe"
Name: "{userdesktop}\TBG"; Filename: "{app}\tbg-client.exe"; Tasks: desktopicon
[Run]
Filename: "{app}\tbg-client.exe"; Description: "{cm:LaunchProgram,TBG}"; Flags: nowait postinstall skipifsilent
[CustomMessages]
english.CloseFirst=Close TBG normally and try again. The installer will not stop a running game.
brazilianportuguese.CloseFirst=Feche o TBG normalmente e tente novamente. O instalador não encerra uma partida em andamento.
[Code]
function PrepareToInstall(var NeedsRestart: Boolean): String;
var Probe: TFileStream;
begin
  { A locked executable is a normal-close request, never permission to kill a match. }
  Result := '';
  if FileExists(ExpandConstant('{app}\tbg-client.exe')) then
  begin
    try
      Probe := TFileStream.Create(ExpandConstant('{app}\tbg-client.exe'), fmOpenReadWrite or fmShareExclusive);
      Probe.Free;
    except
      Result := CustomMessage('CloseFirst');
    end;
  end;
end;
