@echo off
REM 无限循环模拟客户端请求

REM 定义服务地址和 proto 文件路径
SET server=127.0.0.1:50052
SET proto_path=D:\_temp
SET proto_file=service.proto

REM 初始化自增整型值
SET /A counter=0

REM 生成全局随机整型值，范围 [1, 10000000000]
FOR /F %%A IN ('powershell -Command "Get-Random -Minimum 1 -Maximum 10000000000"') DO SET global_random=%%A

:loop
REM 获取当前时间字符串
FOR /F "tokens=1-4 delims=/-: " %%A IN ("%date% %time%") DO (
    SETLOCAL ENABLEDELAYEDEXPANSION

    REM 构造消息内容
    SET message={\"message\": \"@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  !global_random!  >>>>>>  !counter!  <<<<<<<<<<<<<<<<<<\"}

    REM 发送请求
    grpcurl --plaintext --import-path "!proto_path!" --proto "!proto_file!" -d "!message!" !server! grpc_service.Greeter/SayHello

    ENDLOCAL
)

SET /A counter+=1

REM 延迟 1 秒，避免过于频繁的请求
TIMEOUT /T 1 /NOBREAK >nul
GOTO loop

