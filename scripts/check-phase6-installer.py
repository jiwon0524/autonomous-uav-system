#!/usr/bin/env python3
"""Static guard and optional NSIS page harness; this is not a Windows installer test."""
import argparse
import json
import subprocess
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / 'src-tauri/installer/firescene.nsi').read_text()
cfg = json.loads((ROOT / 'src-tauri/tauri.conf.json').read_text())
nsis = cfg['bundle']['windows']['nsis']
version = cfg['version']
assert version == tomllib.loads((ROOT / 'src-tauri/Cargo.toml').read_text())['package']['version']
assert version == json.loads((ROOT / 'frontend/package.json').read_text())['version']
lock = json.loads((ROOT / 'frontend/package-lock.json').read_text())
assert version == lock['version'] == lock['packages']['']['version']
assert nsis['template'] == 'installer/firescene.nsi'
assert nsis['installMode'] == 'currentUser'
assert nsis['languages'] == ['Korean', 'English']
assert cfg['identifier'] == 'org.firescene.gcs'
assert cfg['bundle']['windows']['webviewInstallMode']['type'] == 'downloadBootstrapper'
for image in ('sidebarImage', 'headerImage'):
    assert (ROOT / 'src-tauri' / nsis[image]).is_file()
for item in ('Page custom FireSceneWelcome', 'Page custom PageReinstall PageLeaveReinstall',
             'MUI_PAGE_DIRECTORY', 'MUI_PAGE_STARTMENU', 'MUI_PAGE_INSTFILES',
             'MUI_PAGE_FINISH', 'MUI_UNPAGE_CONFIRM', 'MUI_UNPAGE_INSTFILES',
             'Section WebView2', 'WriteUninstaller', 'CreateOrUpdateStartMenuShortcut',
             'DeleteAppDataCheckboxState'):
    assert item in source, item
assert '${LANG_KOREAN}' not in source[source.index('Function FireSceneWelcome'):source.index('FunctionEnd', source.index('Function FireSceneWelcome'))]
for key in ('DirectoryTitle', 'DirectorySubtitle', 'ShortcutTitle', 'ShortcutSubtitle',
            'InstallTitle', 'InstallSubtitle', 'FinishTitle', 'FinishText',
            'UninstallTitle', 'UninstallSubtitle', 'RemoveTitle', 'RemoveSubtitle'):
    for language in ('ENGLISH', 'KOREAN'):
        assert f'LangString Fire{key} ${{LANG_{language}}}' in source
print('Static configuration and page guards: PASS')

parser = argparse.ArgumentParser()
parser.add_argument('--makensis')
args = parser.parse_args()
if args.makensis:
    welcome = source[source.index('Page custom FireSceneWelcome'):source.index('; 2. License Page')]
    welcome = welcome.replace('${SIDEBARIMAGE}', str(ROOT / 'src-tauri/installer/sidebar.bmp'))
    strings = '\n'.join(line for line in source.splitlines() if line.startswith('LangString Fire'))
    with tempfile.TemporaryDirectory(prefix='firescene-nsis-harness-') as temp:
        out = Path(temp)
        script = '''Unicode true
!include MUI2.nsh
Name "FireScene harness"
OutFile "@OUT@"
!define VERSION "@VERSION@"
!define MUI_BGCOLOR "0B0F12"
!define MUI_TEXTCOLOR "E9EEF2"
@WELCOME@
!define MUI_PAGE_HEADER_TEXT "$(FireDirectoryTitle)"
!define MUI_PAGE_HEADER_SUBTEXT "$(FireDirectorySubtitle)"
!insertmacro MUI_PAGE_DIRECTORY
Var AppStartMenuFolder
!define MUI_PAGE_HEADER_TEXT "$(FireShortcutTitle)"
!define MUI_PAGE_HEADER_SUBTEXT "$(FireShortcutSubtitle)"
!insertmacro MUI_PAGE_STARTMENU Application $AppStartMenuFolder
!define MUI_PAGE_HEADER_TEXT "$(FireInstallTitle)"
!define MUI_PAGE_HEADER_SUBTEXT "$(FireInstallSubtitle)"
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_TITLE "$(FireFinishTitle)"
!define MUI_FINISHPAGE_TEXT "$(FireFinishText)"
!insertmacro MUI_PAGE_FINISH
!define MUI_PAGE_HEADER_TEXT "$(FireUninstallTitle)"
!define MUI_PAGE_HEADER_SUBTEXT "$(FireUninstallSubtitle)"
!insertmacro MUI_UNPAGE_CONFIRM
!define MUI_PAGE_HEADER_TEXT "$(FireRemoveTitle)"
!define MUI_PAGE_HEADER_SUBTEXT "$(FireRemoveSubtitle)"
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "Korean"
!insertmacro MUI_LANGUAGE "English"
@STRINGS@
Function SkipIfPassive
FunctionEnd
Section
WriteUninstaller "$INSTDIR\\uninstall.exe"
SectionEnd
Section Uninstall
SectionEnd
'''.replace('@OUT@', str(out / 'harness.exe')).replace('@WELCOME@', welcome).replace('@STRINGS@', strings).replace('@VERSION@', version)
        path = out / 'harness.nsi'
        path.write_text(script)
        result = subprocess.run([args.makensis, '-WX', str(path)], text=True, capture_output=True)
        print(result.stdout)
        if result.returncode:
            print(result.stderr)
            raise SystemExit(result.returncode)
        print('NSIS page harness compile: PASS; full Tauri bundle and Windows execution NOT TESTED')
