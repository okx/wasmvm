package api

/*
#include "bindings.h"
#include <stdio.h>

// imports (db)
GoError cSet(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, U8SliceView val, UnmanagedVector *errOut);
GoError cGet(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *val, UnmanagedVector *errOut);
GoError cDelete(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *errOut);
GoError cScan(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView start, U8SliceView end, int32_t order, GoIter *out, UnmanagedVector *errOut);
// imports (iterator)
GoError cNext(iterator_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *key, UnmanagedVector *val, UnmanagedVector *errOut);
// imports (api)
GoError cHumanAddress(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas);
GoError cCanonicalAddress(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas);
GoError cGetCallInfo(api_t *ptr, uint64_t *used_gas, U8SliceView contractAddress, U8SliceView storeAddress, UnmanagedVector *resCodeHash, Db **resStore, GoQuerier **resQuerier, uint64_t *call_id, UnmanagedVector *errOut);
GoError cGetWasmInfo(cache_t **resCache_t, UnmanagedVector *errOut);
GoError cRelease(uint64_t call_id);
GoError cTransferCoins(api_t *ptr, uint64_t *used_gas, U8SliceView contractAddress, U8SliceView caller, U8SliceView coins, UnmanagedVector *errOut);
// imports (querier)
GoError cQueryExternal(querier_t *ptr, uint64_t gas_limit, uint64_t *used_gas, U8SliceView request, UnmanagedVector *result, UnmanagedVector *errOut);

// Gateway functions (db)
GoError cGet_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *val, UnmanagedVector *errOut) {
	return cGet(ptr, gas_meter, used_gas, key, val, errOut);
}
GoError cSet_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, U8SliceView val, UnmanagedVector *errOut) {
	return cSet(ptr, gas_meter, used_gas, key, val, errOut);
}
GoError cDelete_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *errOut) {
	return cDelete(ptr, gas_meter, used_gas, key, errOut);
}
GoError cScan_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView start, U8SliceView end, int32_t order, GoIter *out, UnmanagedVector *errOut) {
	return cScan(ptr, gas_meter, used_gas, start, end, order, out, errOut);
}

// Gateway functions (iterator)
GoError cNext_cgo(iterator_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *key, UnmanagedVector *val, UnmanagedVector *errOut) {
	return cNext(ptr, gas_meter, used_gas, key, val, errOut);
}

// Gateway functions (api)
GoError cCanonicalAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas) {
    return cCanonicalAddress(ptr, src, dest, errOut, used_gas);
}
GoError cHumanAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas) {
    return cHumanAddress(ptr, src, dest, errOut, used_gas);
}

GoError cGetCallInfo_cgo(api_t *ptr, uint64_t *used_gas, U8SliceView contractAddress, U8SliceView storeAddress, UnmanagedVector *resCodeHash, Db **resStore, GoQuerier **resQuerier, uint64_t *call_id, UnmanagedVector *errOut) {
    return cGetCallInfo(ptr, used_gas, contractAddress, storeAddress, resCodeHash, resStore, resQuerier, call_id, errOut);
}

GoError cGetWasmInfo_cgo(cache_t **resCache_t, UnmanagedVector *errOut) {
    return cGetWasmInfo(resCache_t, errOut);
}

GoError cRelease_cgo(uint64_t call_id) {
   return cRelease(call_id);
}

GoError cTransferCoins_cgo(api_t *ptr, uint64_t *used_gas, U8SliceView contractAddress, U8SliceView caller, U8SliceView coins, UnmanagedVector *errOut) {
   return cTransferCoins(ptr, used_gas, contractAddress, caller, coins, errOut);
}

// Gateway functions (querier)
GoError cQueryExternal_cgo(querier_t *ptr, uint64_t gas_limit, uint64_t *used_gas, U8SliceView request, UnmanagedVector *result, UnmanagedVector *errOut) {
    return cQueryExternal(ptr, gas_limit, used_gas, request, result, errOut);
}

*/
import "C"

// We need these gateway functions to allow calling back to a go function from the c code.
// At least I didn't discover a cleaner way.
// Also, this needs to be in a different file than `callbacks.go`, as we cannot create functions
// in the same file that has //export directives. Only import header types
