package api

// #include"bindings.h"
import "C"

import (
	"github.com/CosmWasm/wasmvm/types"
	"unsafe"
)

var GetCallInfoFunc func(q unsafe.Pointer, contractAddress, storeAddress string) ([]byte, types.KVStore, types.Querier, types.GasMeter, error)

var GetWasmCacheInfoFunc func() (types.GoAPI, Cache)

func RegisterGetWasmCallInfo(fnn func(q unsafe.Pointer, contractAddress, storeAddress string) ([]byte, types.KVStore, types.Querier, types.GasMeter, error)) {
	GetCallInfoFunc = fnn
}

func RegisterGetWasmCacheInfo(fnn func() (types.GoAPI, Cache)) {
	GetWasmCacheInfoFunc = fnn
}

func GetCallInfo(p unsafe.Pointer, contrAddr C.U8SliceView, storeAddr C.U8SliceView, resCodeHash *C.UnmanagedVector, resStore **C.Db, resQuerier **C.GoQuerier, errOut *C.UnmanagedVector) (ret C.GoError) {
	if GetCallInfoFunc == nil {
		*errOut = newUnmanagedVector([]byte("the GetCallInfoFunc is nil"))
		return C.GoError_Other
	}
	cAddr := copyU8Slice(contrAddr)
	sAddr := copyU8Slice(storeAddr)
	codeHash, store, querier, gasMeter, err := GetCallInfoFunc(p, string(cAddr), string(sAddr))
	if err != nil {
		*errOut = newUnmanagedVector([]byte(err.Error()))
		return C.GoError_Other
	}
	*resCodeHash = newUnmanagedVector(codeHash)
	dbstate := buildDBState(store, startCall())
	rs := buildDB(&dbstate, &gasMeter)
	*resStore = &rs
	rq := buildQuerier(&querier)
	*resQuerier = &rq
	return C.GoError_None
}

func GetWasmCacheInfo(resGoApi **C.GoApi, resCache_t **C.cache_t, errOut *C.UnmanagedVector) (ret C.GoError) {
	if GetWasmCacheInfoFunc == nil {
		*errOut = newUnmanagedVector([]byte("the GetWasmCacheInfoFunc is nil"))
		return C.GoError_Other
	}
	api, cache := GetWasmCacheInfoFunc()
	rsap := buildAPI(&api)
	*resGoApi = &rsap
	*resCache_t = cache.ptr
	return C.GoError_None
}

func Release(ptr *C.db_t) (ret C.GoError) {
	if ptr == nil {
		return C.GoError_None
	}
	state := (*DBState)(unsafe.Pointer(ptr))
	endCall(state.CallID)
	return C.GoError_None
}
