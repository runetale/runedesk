
.PHONY: capture
capture:
	VCPKG_ROOT=/Users/shinta/.ghq/github.com/microsoft/vcpkg cargo run --package scrap --example record-screen capture.webm
