SHELL := /bin/bash
.PHONY: build
OS = $(shell uname)
clean:
	rm -rf build
	#rm main

build:
	@echo The OS is $(OS)
	mkdir build
	cargo build --release
ifeq ($(OS),Linux)
	cp target/release/libgo_move.so build/
else
	cp target/release/libgo_move.dylib build/
endif
	cp -a lib/. build/
	go build -o main -ldflags="-r ./lib" cmd/main.go
run:
	export LD_LIBRARY_PATH=./build && ./main
