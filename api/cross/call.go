package main

import "C"

import (
	//"github.com/CosmWasm/wasmvm/api"
	"unsafe"
)

var GenerateCallerInfoFunc func(q unsafe.Pointer /*C.GGoQuerier*/, contractAddress string) ([]byte, KVStore, Querier, GasMeter)

func RegisterGenerateCallerInfo(fnn func(q unsafe.Pointer /*C.GGoQuerier*/, contractAddress string) ([]byte, KVStore, Querier, GasMeter)) {
	GenerateCallerInfoFunc = fnn
}

////export GenerateCallerInfo
//func GenerateCallerInfo(p unsafe.Pointer, contractAddress *C.char, resCodeHash **C.char, resStore *C.Db, resQuerier *C.GoQuerier) {
//	goContractAddress := C.GoString(contractAddress)
//	codeHash, store, querier, gasMeter := GenerateCallerInfoFunc(p, goContractAddress)
//	//*resCodeHash = C.CString(codeHash)
//	*resCodeHash = (*C.char)(unsafe.Pointer(&codeHash[0]))
//	dbstate := buildDBState(store, 0)
//	resStore = buildDB(&dbstate, &gasMeter)
//	resQuerier = buildQuerier(&querier)
//}
//
//func main() {}
