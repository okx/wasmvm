package api

import (
	"unsafe"
)

var GenerateCallerInfoFunc func(q unsafe.Pointer /*C.GGoQuerier*/, contractAddress string) ([]byte, KVStore, Querier)

func RegisterGenerateCallerInfo(fnn func(q unsafe.Pointer /*C.GGoQuerier*/, contractAddress string) ([]byte, KVStore, Querier)) {
	GenerateCallerInfoFunc = fnn
}

//export GenerateCallerInfo
func GenerateCallerInfo(q unsafe.Pointer, contractAddress string) ([]byte, KVStore, Querier) {
	return GenerateCallerInfoFunc(q, contractAddress)
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
