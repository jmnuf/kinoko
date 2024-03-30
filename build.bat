@ECHO OFF
SETLOCAL
SET TARGET_DIR=.\build
SET TARGET=%TARGET_DIR%\kinoko.exe
IF EXIST %TARGET% (
	echo MOVE /Y %TARGET% %TARGET_DIR%\kinoko.old.exe
	MOVE /Y %TARGET% kinoko.old.exe
)

echo rustc -o %TARGET% .\src\main.rs -C opt-level=3
rustc -o %TARGET% .\src\main.rs -C opt-level=3 && echo Compiled succesfully

ENDLOCAL
