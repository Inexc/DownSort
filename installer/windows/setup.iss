; Script generated for DownSort Inno Setup Installer
#define MyAppName "DownSort"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "DownSort"
#define MyAppURL "https://github.com/Inexc/DownSort"
#define MyAppExeName "downsort.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
AppId={{9F5F6C62-1082-4EBE-9A2E-C246E563065A}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={localappdata}\Programs\{#MyAppName}
DisableProgramGroupPage=yes
; PrivilegesRequired=lowest allows installation without administrator privileges (UAC prompt)
PrivilegesRequired=lowest
OutputDir=..\..\dist
OutputBaseFilename=DownSort-Setup-v{#MyAppVersion}
SetupIconFile=..\..\crates\app\downsort.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\crates\app\downsort.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\downsort.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\downsort.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
