@echo off

SET server=dev.vm:50052
SET proto_path=D:\_temp
SET proto_file=euclidolap.proto

SET param_type=%~1
SET param_statement=%~2

IF "%param_type%"=="" SET param_type=__MDX_QUERYING__
IF "%param_statement%"=="" SET param_statement=__________________this is a mdx statement__________________

:loop
	for /l %%i in (1,1,20) do (
		grpcurl --plaintext --import-path "%proto_path%" --proto "%proto_file%" -d "{\"operation_type\": \"%param_type%\", \"statement\": \"%param_statement% - %%i\"}" %server% euclidolap.OlapApi/ExecuteOperation
	)

	timeout /t 1 >nul

goto loop
