@echo off
set BRO_EXEC=%TEMP%\bro_%RANDOM%.bat
python R:\bro-cli\bro.py --exec-file "%BRO_EXEC%" %*
if exist "%BRO_EXEC%" (
    call "%BRO_EXEC%"
    del "%BRO_EXEC%"
)
