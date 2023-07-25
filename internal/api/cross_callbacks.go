package api

// #include"bindings.h"
import "C"

import (
	"github.com/CosmWasm/wasmvm/types"
	"unsafe"
)

var GetWasmCacheInfoFunc func() Cache

func RegisterGetWasmCacheInfo(fnn func() Cache) {
	GetWasmCacheInfoFunc = fnn
}

func GetCallInfo(p unsafe.Pointer, usedGas *cu64, contrAddr C.U8SliceView, storeAddr C.U8SliceView, resCodeHash *C.UnmanagedVector, resStore **C.Db, resQuerier **C.GoQuerier, callID *cu64, errOut *C.UnmanagedVector) (ret C.GoError) {
	cAddr := copyU8Slice(contrAddr)
	sAddr := copyU8Slice(storeAddr)
	api := (*types.GoAPI)(p)
	if api.GetCallInfo == nil {
		*errOut = newUnmanagedVector([]byte("the GetCallInfo is nil"))
		return C.GoError_Other
	}
	codeHash, gas, store, querier, gasMeter, err := api.GetCallInfo(string(cAddr), string(sAddr))
	*usedGas = (cu64)(gas)
	if err != nil {
		*errOut = newUnmanagedVector([]byte(err.Error()))
		return C.GoError_Other
	}
	*resCodeHash = newUnmanagedVector(codeHash)
	scallID := startCall()
	*callID = (cu64)(scallID)
	dbstate := buildDBState(store, scallID)
	rs := buildDB(&dbstate, &gasMeter)
	*resStore = &rs
	rq := buildQuerier(&querier)
	*resQuerier = &rq
	return C.GoError_None
}

func GetWasmCacheInfo(resCache_t **C.cache_t, errOut *C.UnmanagedVector) (ret C.GoError) {
	if GetWasmCacheInfoFunc == nil {
		*errOut = newUnmanagedVector([]byte("the GetWasmCacheInfoFunc is nil"))
		return C.GoError_Other
	}
	cache := GetWasmCacheInfoFunc()
	*resCache_t = cache.ptr
	return C.GoError_None
}

func Release(callID cu64) (ret C.GoError) {
	endCall(uint64(callID))
	return C.GoError_None
}

func TransferCoins(p unsafe.Pointer, usedGas *cu64, contrAddr C.U8SliceView, caller C.U8SliceView, coins C.U8SliceView, errOut *C.UnmanagedVector) (ret C.GoError) {
	cAddr := copyU8Slice(contrAddr)
	ccaller := copyU8Slice(caller)
	ccoins := copyU8Slice(coins)
	api := (*types.GoAPI)(p)
	if api.TransferCoins == nil {
		*errOut = newUnmanagedVector([]byte("the TransferCoins is nil"))
		return C.GoError_Other
	}
	gas, err := api.TransferCoins(string(cAddr), string(ccaller), ccoins)
	*usedGas = (cu64)(gas)
	if err != nil {
		*errOut = newUnmanagedVector([]byte(err.Error()))
		return C.GoError_Other
	}
	return C.GoError_None
}
