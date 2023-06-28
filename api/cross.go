package api

// #include"bindings.h"
import "C"

import (
	"fmt"
	"unsafe"
)

var GenerateCallerInfoFunc func(q unsafe.Pointer, contractAddress string) ([]byte, KVStore, Querier, GasMeter)

func RegisterGenerateCallerInfo(fnn func(q unsafe.Pointer, contractAddress string) ([]byte, KVStore, Querier, GasMeter)) {
	GenerateCallerInfoFunc = fnn
}

//func GenerateCallerInfo(p unsafe.Pointer, contractAddress *C.char, resCodeHash **C.char, resStore *C.Db, resQuerier *C.GoQuerier) {
//	if GenerateCallerInfoFunc == nil {
//		panic("the GenerateCallerInfoFunc is nil")
//	}
//	goContractAddress := C.GoString(contractAddress)
//	fmt.Println("the contract address is", goContractAddress)
//	codeHash, store, querier, gasMeter := GenerateCallerInfoFunc(p, goContractAddress)
//	*resCodeHash = (*C.char)(unsafe.Pointer(&codeHash[0]))
//	dbstate := buildDBState(store, 0)
//	rs := buildDB(&dbstate, &gasMeter)
//	resStore = &rs
//	rq := buildQuerier(&querier)
//	resQuerier = &rq
//}

func GenerateCallerInfo(p unsafe.Pointer, contractAddress *C.char, resCodeHash **C.char, resStore **C.Db, resQuerier **C.GoQuerier) {
	if GenerateCallerInfoFunc == nil {
		panic("the GenerateCallerInfoFunc is nil")
	}
	goContractAddress := C.GoString(contractAddress)
	fmt.Println("the contract address is", goContractAddress)
	codeHash, store, querier, gasMeter := GenerateCallerInfoFunc(p, goContractAddress)
	*resCodeHash = (*C.char)(unsafe.Pointer(&codeHash[0]))
	dbstate := buildDBState(store, 0)
	rs := buildDB(&dbstate, &gasMeter)
	*resStore = &rs
	rq := buildQuerier(&querier)
	*resQuerier = &rq
}

//func CovertToGGoQuerier(q C.GoQuerier) C.GGoQuerier {
//	return C.GGoQuerier{
//		State: q.state,
//	}
//}

//var GenerateCallerInfoFunc1 func(q, contractAddress string) ([]byte, KVStore, Querier)
//
//func RegisterGenerateCallerInfo1(fnn func(ctx types.Context, contractAddress string) ([]byte, KVStore, Querier)) {
//	GenerateCallerInfoFunc = fnn
//}

//func GenerateCallerInfo(q C.GoQuerier, contractAddress string) ([]byte, wasmvmtypes.Env, wasmvm.KVStore, wasmvm.Querier) {
//	goQuerier := (*keeper.QueryHandler)(unsafe.Pointer((q.state)))
//	return keeper.GenerateCallerInfo(goQuerier.Ctx, contractAddress)
//}
