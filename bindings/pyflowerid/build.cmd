@echo off
setlocal
python setup.py test && ^
python setup.py bdist_wheel && ^

REM set SRC_DIR=%cd%
REM set CARGO_TARGET_DIR=%SRC_DIR%/target
REM mkdir %CARGO_TARGET_DIR%
REM cd ../flowerid_c
REM cargo build --release && ^
REM cd %SRC_DIR% && ^
REM cython pyflowerid.pyx -o target/pyflowerid.c && ^
REM cd %CARGO_TARGET_DIR% && ^
REM clang -shared -I C:\Python36\include -I "../../flowerid_c/include/" -L C:\Python36\libs pyflowerid.c -L ./release -lflowerid_c -lWs2_32 -lUserenv -lAdvapi32 -lShell32 -o pyflowerid.pyd && ^
REM copy pyflowerid.pyd ..\pyflowerid.pyd
