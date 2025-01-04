@echo off
Rem this creates a zip file with the voxelworld executable inside

set VOXELWORLD_DIR=voxelworld-windows

rmdir /S release\
mkdir release\%VOXELWORLD_DIR%
copy target\release\voxelworld.exe release\%VOXELWORLD_DIR%
copy cfg.impfile release\%VOXELWORLD_DIR%
Xcopy /E /I assets\ release\%VOXELWORLD_DIR%\assets\
cd release\
Rem NOTE: You need 7-zip installed to use this script
"C:\Program Files\7-Zip\7z.exe" a -tzip voxelworld-windows.zip %VOXELWORLD_DIR%
cd ..
