@echo off

:: 配置 gRPC 服务地址和 proto 文件路径
SET server=dev.vm:50052
SET proto_path=D:\_temp
SET proto_file=euclidolap.proto

:: 获取命令行参数（如果有）
SET param_type=%~1
SET param_statement=%~2

:: 默认值（如果未提供参数）
IF "%param_type%"=="" SET param_type=__MDX_QUERYING__
IF "%param_statement%"=="" SET param_statement=__________________this is a mdx statement__________________

:loop
	:: 模拟批量请求
	for /l %%i in (1,1,20) do (
		:: 构造 JSON 请求体并调用 gRPC 接口
		grpcurl --plaintext --import-path "%proto_path%" --proto "%proto_file%" -d "{\"operation_type\": \"%param_type%\", \"statement\": \"%param_statement% - %%i\"}" %server% euclidolap.OlapApi/ExecuteOperation
	)

	:: 等待 1 秒后继续
	timeout /t 1 >nul

goto loop
