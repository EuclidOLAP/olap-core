@echo off

SET server=127.0.0.1:50052
SET proto_path=D:\_temp
SET proto_file=service.proto

SET param=%~1

:loop

	for /l %%i in (1,1,20) do (
		grpcurl --plaintext --import-path "%proto_path%" --proto "%proto_file%" -d "{\"message\": \"%param% - %%i\"}" %server% grpc_service.Greeter/SayHello
	)

timeout /t 1 >nul

goto loop
