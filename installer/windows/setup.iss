; ==============================================================================
; DownSort - Официальный установщик Windows (Inno Setup)
; ==============================================================================
#define MyAppName "DownSort"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "DownSort Team"
#define MyAppURL "https://github.com/Inexc/DownSort"
#define MyAppExeName "downsort.exe"

[Setup]
AppId={{9F5F6C62-1082-4EBE-9A2E-C246E563065A}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={localappdata}\Programs\{#MyAppName}
DisableProgramGroupPage=yes
; Установка для текущего пользователя без назойливых запросов UAC / прав администратора
PrivilegesRequired=lowest
OutputDir=..\..\dist
OutputBaseFilename=DownSort-Setup
SetupIconFile=..\..\crates\app\downsort.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}
AppComments=Умный анализатор и уборщик папки Загрузки
VersionInfoVersion={#MyAppVersion}
VersionInfoDescription=Установщик DownSort
VersionInfoProductName={#MyAppName}

[Languages]
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\crates\app\downsort.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\downsort.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\downsort.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
