@echo off
echo Building renderer...
cd renderer
call npm run build
cd ..
echo Starting Electron app...
call npx electron .