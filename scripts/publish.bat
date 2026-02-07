@echo off

set ALLOW_DIRTY=0

:parse_args
if "%~1"=="" goto args_done

if "%~1"=="--allow-dirty" (
	set ALLOW_DIRTY=1
	shift
	goto parse_args
)

if "%~1"=="-h" (
	echo Usage: %~nx0 [--allow-dirty]
	exit /b 0
)

if "%~1"=="--help" (
	echo Usage: %~nx0 [--allow-dirty]
	exit /b 0
)

echo Error: unknown argument: %~1
echo Usage: %~nx0 [--allow-dirty]
exit /b 2

:args_done

rem Require a clean git checkout (no staged/unstaged changes; untracked OK)
if %ALLOW_DIRTY%==0 (
	git diff-index --quiet HEAD --
	if not %errorlevel%==0 (
		echo Error: git working tree is dirty ^(staged or unstaged changes^).
		echo Re-run with --allow-dirty to bypass this check.
		echo ^(Tip: untracked files are already allowed.^)
		exit /b 1
	)
)

rem Clean the publish directory
rmdir /S /Q target\publish
mkdir target\publish

rem Build the executables
cargo build --release --bin chipplay
cargo build --release --bin chipedit

rem Copy the executables to the publish directory
copy target\release\chipplay.exe target\publish
copy target\release\chipedit.exe target\publish

rem Copy the config
copy chipgame.ini target\publish

rem Package the assets
pakscmd target/publish/data.paks 0 new
pakscmd target/publish/data.paks 0 copy "" data

rem Package the levelsets
mkdir target\publish\levelsets
cargo run --bin packset levelsets\cclp1 target\publish\levelsets\cclp1.paks
cargo run --bin packset levelsets\cclp2 target\publish\levelsets\cclp2.paks
cargo run --bin packset levelsets\cclp3 target\publish\levelsets\cclp3.paks
cargo run --bin packset levelsets\cclp4 target\publish\levelsets\cclp4.paks
cargo run --bin packset levelsets\cclp5 target\publish\levelsets\cclp5.paks

rem Create the save dir
mkdir target\publish\save

rem Generate the documentation
cargo run --bin makedocs

rem Zip it all up
del /Q target\chipgame.zip 2> NUL
pushd target\publish
powershell -NoProfile -Command "Compress-Archive -Path * -DestinationPath ..\chipgame.zip -Force"
popd
