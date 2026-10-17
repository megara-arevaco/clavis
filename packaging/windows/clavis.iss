#define AppVersion GetEnv("CLAVIS_VERSION")

[Setup]
AppId={{DE2D34AE-CB94-49EE-B74C-55CA27171C42}
AppName=Clavis
AppVersion={#AppVersion}
AppPublisher=Baul
DefaultDirName={localappdata}\Programs\Clavis
DefaultGroupName=Clavis
PrivilegesRequired=lowest
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=..\..\dist\windows
OutputBaseFilename=Clavis-{#AppVersion}-setup-x64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\clavis.exe
VersionInfoVersion={#AppVersion}
VersionInfoProductName=Clavis
VersionInfoDescription=Terminal shortcut and command reference

[Tasks]
Name: "autostart"; Description: "Start Clavis when I sign in"; Flags: unchecked

[Files]
Source: "..\..\dist\windows\stage\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Clavis"; Filename: "{app}\clavis.exe"; Parameters: "--show"
Name: "{userstartup}\Clavis"; Filename: "{app}\clavis.exe"; Tasks: autostart

[Run]
Filename: "{app}\clavis.exe"; Parameters: "--show"; Description: "Open Clavis"; Flags: postinstall nowait skipifsilent
