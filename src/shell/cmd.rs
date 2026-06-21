use std::path::Path;

use anyhow::Result;

use super::{InjectionMode, Shell, ShellKind};

pub struct Cmd;

impl Shell for Cmd {
    fn kind(&self) -> ShellKind {
        ShellKind::Cmd
    }

    fn injection_mode(&self) -> InjectionMode {
        InjectionMode::TempFileCall
    }

    fn quote(&self, arg: &str) -> String {
        // Basic cmd.exe quoting: wrap in double-quotes, escape internal double-quotes
        if arg.contains(' ') || arg.contains('"') || arg.contains('&') {
            format!("\"{}\"", arg.replace('"', "\"\""))
        } else {
            arg.to_string()
        }
    }

    fn sequence(&self, cmds: &[String]) -> String {
        cmds.join(" &\n")
    }

    fn init_script(&self, bin: &Path) -> Result<String> {
        let bin_str = bin.display();
        Ok(format!(
            r#"@echo off
:: bro wrapper for cmd.exe
:: Save this as bro.bat somewhere on PATH that appears BEFORE the directory
:: containing bro.exe (cmd prefers .bat over .exe at same PATH position).
:: To load at cmd.exe startup, set this registry value:
::   HKCU\Software\Microsoft\Command Processor\AutoRun
::   REG_SZ: call "%USERPROFILE%\bin\bro.bat"
call :_bro_run %*
exit /b %ERRORLEVEL%

:_bro_run
set "BRO_MGMT=add update set remove rm list ls info search find edit init paths run completions help"

if "%~1"=="" goto :bro_pick
if /i "%~1"=="-f" goto :bro_pick
if /i "%~1"=="pick" goto :bro_pick

for %%m in (%BRO_MGMT%) do if /i "%~1"=="%%m" (
    "{bin}" %*
    exit /b %ERRORLEVEL%
)
set "BRO_FIRST=%~1"
if "%BRO_FIRST:~0,1%"=="-" (
    "{bin}" %*
    exit /b %ERRORLEVEL%
)
set "BRO_TMP=%TEMP%\bro_%RANDOM%.bat"
"{bin}" --emit --shell-name cmd --exec-file "%BRO_TMP%" run %*
if %ERRORLEVEL% neq 0 exit /b %ERRORLEVEL%
if not exist "%BRO_TMP%" exit /b 0
call "%BRO_TMP%"
set "BRO_EXIT=%ERRORLEVEL%"
del "%BRO_TMP%" 2>nul
exit /b %BRO_EXIT%

:bro_pick
set "BRO_TMP=%TEMP%\bro_%RANDOM%.bat"
"{bin}" --emit --shell-name cmd --exec-file "%BRO_TMP%" pick
if %ERRORLEVEL% neq 0 exit /b %ERRORLEVEL%
if not exist "%BRO_TMP%" exit /b 0
call "%BRO_TMP%"
set "BRO_EXIT=%ERRORLEVEL%"
del "%BRO_TMP%" 2>nul
exit /b %BRO_EXIT%
"#,
            bin = bin_str,
        ))
    }
}
