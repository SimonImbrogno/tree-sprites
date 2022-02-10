@echo off

if "%1" == "enable" (
    set RUST_LOG=warn,maze=debug
) else if "%1" == "disable" (
    set RUST_LOG=off
) else if "%1" == "loud" (
    set RUST_LOG=debug
) else (
    echo "unrecognized option"
)
