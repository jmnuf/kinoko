@ECHO OFF
SETLOCAL
SET TARGET=.\build\kinoko.exe
IF EXIST %TARGET% (
	echo REN %TARGET% kinoko.old.exe
	REN %TARGET% kinoko.old.exe
)

echo rustc -o %TARGET% .\src\main.rs -C opt-level=3
rustc -o %TARGET% .\src\main.rs -C opt-level=3 && echo Compiled succesfully

ENDLOCAL
