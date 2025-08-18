; installer.iss - Windows installer for Multicode Desktop

[Setup]
AppName=Multicode
AppVersion=0.1.0
DefaultDirName={pf}\Multicode
OutputBaseFilename=MulticodeSetup
OutputDir=dist
DisableProgramGroupPage=yes

[Files]
Source: "..\\target\\release\\desktop.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\\Multicode"; Filename: "{app}\\desktop.exe"

; Placeholder for future WinSparkle auto-updater integration
; #define EnableWinSparkle
; If defined EnableWinSparkle then
;   ; WinSparkle setup tasks would go here
; endif
