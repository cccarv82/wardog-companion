#ifndef AppVersion
  #define AppVersion "1.0.1"
#endif
#ifndef PayloadDir
  #define PayloadDir "..\..\output\wardog-companion-v1.0"
#endif
#ifndef ReleaseDir
  #define ReleaseDir "..\..\output"
#endif

[Setup]
AppId={{71A60EC5-9053-4606-B796-2D23E8B69089}
AppName=WARDOGS Companion
AppVersion={#AppVersion}
AppVerName=WARDOGS Companion {#AppVersion}
AppPublisher=Carlos Carvalho and contributors
AppPublisherURL=https://github.com/cccarv82/wardog-companion
AppSupportURL=https://github.com/cccarv82/wardog-companion/issues
AppUpdatesURL=https://github.com/cccarv82/wardog-companion/releases
DefaultDirName={localappdata}\Programs\WARDOG Companion
DefaultGroupName=WARDOGS Companion
DisableProgramGroupPage=yes
UsePreviousGroup=no
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
SetupIconFile=..\assets\wardog.ico
UninstallDisplayIcon={app}\wardog-companion.exe
VersionInfoDescription=WARDOGS Companion Setup
VersionInfoVersion={#AppVersion}
LicenseFile=..\LICENSE
WizardStyle=modern dark includetitlebar
WizardSizePercent=110
WizardImageFile=wizard.png
WizardSmallImageFile=wizard-small.png
WizardImageBackColor=$1D1811
DisableWelcomePage=no
DisableDirPage=auto
DisableReadyPage=no
CloseApplications=yes
CloseApplicationsFilter=wardog-companion.exe
RestartApplications=no
Compression=lzma2/max
SolidCompression=yes
OutputDir={#ReleaseDir}
OutputBaseFilename=WARDOG-Companion-{#AppVersion}-Setup
UninstallDisplayName=WARDOGS Companion
AppMutex=Local\WARDOGCompanion.71A60EC5-9053-4606-B796-2D23E8B69089
SetupLogging=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:"; Flags: unchecked

[Files]
Source: "{#PayloadDir}\wardog-companion.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\Data\catalog.json"; DestDir: "{app}\Data"; Flags: onlyifdoesntexist uninsneveruninstall
Source: "{#PayloadDir}\*.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\START_HERE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\About\*"; DestDir: "{app}\About"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#PayloadDir}\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "{#PayloadDir}\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs createallsubdirs

[InstallDelete]
Type: files; Name: "{userprograms}\WARDOG Companion\WARDOG Companion.lnk"
Type: files; Name: "{userprograms}\WARDOG Companion\Uninstall WARDOG Companion.lnk"
Type: dirifempty; Name: "{userprograms}\WARDOG Companion"
Type: files; Name: "{autodesktop}\WARDOG Companion.lnk"

[Icons]
Name: "{group}\WARDOGS Companion"; Filename: "{app}\wardog-companion.exe"; AppUserModelID: "WardogCompanion.Desktop"
Name: "{group}\Uninstall WARDOGS Companion"; Filename: "{uninstallexe}"
Name: "{autodesktop}\WARDOGS Companion"; Filename: "{app}\wardog-companion.exe"; Tasks: desktopicon; AppUserModelID: "WardogCompanion.Desktop"

[Run]
Filename: "{app}\wardog-companion.exe"; Description: "Launch WARDOGS Companion"; Flags: nowait postinstall skipifsilent

[Messages]
WelcomeLabel1=Your next setup.
WelcomeLabel2=Install WARDOGS Companion to find eligible weapon setups, compare your options and spend smarter.%n%nBuilt with Rust. Open source. Your profile stays on your PC.
FinishedHeadingLabel=Your companion is ready.
FinishedLabel=WARDOGS Companion is installed.%n%nSet your levels, pick a weapon and keep the panel out of the way with Ctrl + Shift + Space.%n%nExisting levels and preferences are preserved when updating. Find app updates in Settings.

[Code]
procedure InitializeWizard;
begin
  WizardForm.WelcomeLabel1.Font.Size := 19;
  WizardForm.FinishedHeadingLabel.Font.Size := 19;
end;

