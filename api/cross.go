package api

// #include"bindings.h"
import "C"

import (
	"unsafe"
)

var GenerateCallerInfoFunc func(q unsafe.Pointer, contractAddress string) ([]byte, KVStore, Querier, GasMeter)

var GetWasmCacheInfoFunc func() (GoAPI, Cache)

func RegisterGenerateCallerInfo(fnn func(q unsafe.Pointer, contractAddress string) ([]byte, KVStore, Querier, GasMeter)) {
	GenerateCallerInfoFunc = fnn
}

func RegisterGetCacheInfo(fnn func() (GoAPI, Cache)) {
	GetWasmCacheInfoFunc = fnn
}

func GenerateCallerInfo(p unsafe.Pointer, contractAddress *C.char, resCodeHash *C.UnmanagedVector, resStore **C.Db, resQuerier **C.GoQuerier) {
	if GenerateCallerInfoFunc == nil {
		panic("the GenerateCallerInfoFunc is nil")
	}
	goContractAddress := C.GoString(contractAddress)
	codeHash, store, querier, gasMeter := GenerateCallerInfoFunc(p, goContractAddress)
	*resCodeHash = newUnmanagedVector(codeHash)
	dbstate := buildDBState(store, 0)
	rs := buildDB(&dbstate, &gasMeter)
	*resStore = &rs
	rq := buildQuerier(&querier)
	*resQuerier = &rq
}

func GetWasmCacheInfo(resGoApi **C.GoApi, resCache_t **C.cache_t) {
	if GetWasmCacheInfoFunc == nil {
		panic("the GetWasmCacheInfoFunc is nil")
	}
	api, cache := GetWasmCacheInfoFunc()
	rsap := buildAPI(&api)
	*resGoApi = &rsap
	*resCache_t = cache.ptr
}
