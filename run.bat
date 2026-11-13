@echo off
chcp 65001 > nul
echo ========================================================
echo   잠실점 에비뉴엘 시설물 및 배관 관제 시스템 (Rust FMS)
echo ========================================================
echo.
echo 서버를 시작합니다... (포트: 8080)
echo 브라우저에서 아래 주소로 접속하세요:
echo http://127.0.0.1:8080
echo.

if exist "target\debug\building-fms.exe" (
    start http://127.0.0.1:8080
    target\debug\building-fms.exe
) else if exist "building-fms.exe" (
    start http://127.0.0.1:8080
    building-fms.exe
) else (
    echo [알림] 실행 파일이 없습니다. cargo run을 실행합니다...
    start http://127.0.0.1:8080
    cargo run
)
pause
