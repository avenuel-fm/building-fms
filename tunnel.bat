@echo off
chcp 65001 > nul
echo ========================================================
echo   잠실점 에비뉴엘 관제 - 외부망(LTE/5G) 접속 터널 실행
echo ========================================================
echo.
echo 1. Rust 백엔드 서버(포트 8080) 확인 중...

REM 서버가 이미 실행 중인지 확인
netstat -ano | findstr ":8080" | findstr "LISTENING" > nul
if %errorlevel% neq 0 (
    echo [알림] FMS 서버를 백그라운드에서 먼저 시작합니다...
    if exist "target\debug\building-fms.exe" (
        start "" /b target\debug\building-fms.exe
    ) else if exist "building-fms.exe" (
        start "" /b building-fms.exe
    ) else (
        start "" /b cargo run
    )
    timeout /t 3 > nul
)

echo.
echo 2. Cloudflare 보안 터널을 시작합니다...
echo 아래에 생성되는 'https://....trycloudflare.com' 주소를
echo 안드로이드 태블릿이나 스마트폰 주소창에 입력하세요!
echo.
"C:\Program Files (x86)\cloudflared\cloudflared.exe" tunnel --url http://127.0.0.1:8080
pause
