package main

/*
#cgo LDFLAGS: -L./../build -lgo_move
#include <stdlib.h>
#include "./../lib/move.h"
*/
import "C"

func main() {
	C.rustdemo()
}