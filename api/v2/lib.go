package v2

import "C"
import (
	"fmt"
	"github.com/CosmWasm/wasmvm/api/utils"
	"github.com/CosmWasm/wasmvm/types"
	"runtime"
	"syscall"
)

// #include <stdlib.h>
// #include "bindings.h"
import "C"

type Querier = types.Querier

func InitCache(dataDir string, supportedFeatures string, cacheSize uint32, instanceMemoryLimit uint32) (Cache, error) {
	dataDirBytes := []byte(dataDir)
	supportedFeaturesBytes := []byte(supportedFeatures)

	d := MakeView(dataDirBytes)
	defer runtime.KeepAlive(dataDirBytes)
	f := MakeView(supportedFeaturesBytes)
	defer runtime.KeepAlive(supportedFeaturesBytes)

	errmsg := NewUnmanagedVector(nil)

	ptr, err := C.init_cache(d, f, Cu32(cacheSize), Cu32(instanceMemoryLimit), &errmsg)
	if err != nil {
		return Cache{}, errorWithMessage(err, errmsg)
	}
	return Cache{Ptr: ptr}, nil
}

func ReleaseCache(cache Cache) {
	C.release_cache(cache.Ptr)
}

func Create(cache Cache, wasm []byte) ([]byte, error) {
	w := MakeView(wasm)
	defer runtime.KeepAlive(wasm)
	errmsg := NewUnmanagedVector(nil)
	checksum, err := C.save_wasm(cache.Ptr, w, &errmsg)
	if err != nil {
		return nil, errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(checksum), nil
}

func GetCode(cache Cache, checksum []byte) ([]byte, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	errmsg := NewUnmanagedVector(nil)
	wasm, err := C.load_wasm(cache.Ptr, cs, &errmsg)
	if err != nil {
		return nil, errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(wasm), nil
}

func Pin(cache Cache, checksum []byte) error {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	errmsg := NewUnmanagedVector(nil)
	_, err := C.pin(cache.Ptr, cs, &errmsg)
	if err != nil {
		return errorWithMessage(err, errmsg)
	}
	return nil
}

func Unpin(cache Cache, checksum []byte) error {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	errmsg := NewUnmanagedVector(nil)
	_, err := C.unpin(cache.Ptr, cs, &errmsg)
	if err != nil {
		return errorWithMessage(err, errmsg)
	}
	return nil
}

func AnalyzeCode(cache Cache, checksum []byte) (*types.AnalysisReport, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	errmsg := NewUnmanagedVector(nil)
	report, err := C.analyze_code(cache.Ptr, cs, &errmsg)
	if err != nil {
		return nil, errorWithMessage(err, errmsg)
	}
	res := types.AnalysisReport{
		HasIBCEntryPoints: bool(report.has_ibc_entry_points),
		RequiredFeatures:  string(CopyAndDestroyUnmanagedVector(report.required_features)),
	}
	return &res, nil
}

func GetMetrics(cache Cache) (*types.Metrics, error) {
	errmsg := NewUnmanagedVector(nil)
	metrics, err := C.get_metrics(cache.Ptr, &errmsg)
	if err != nil {
		return nil, errorWithMessage(err, errmsg)
	}

	return &types.Metrics{
		HitsPinnedMemoryCache:     uint32(metrics.hits_pinned_memory_cache),
		HitsMemoryCache:           uint32(metrics.hits_memory_cache),
		HitsFsCache:               uint32(metrics.hits_fs_cache),
		Misses:                    uint32(metrics.misses),
		ElementsPinnedMemoryCache: uint64(metrics.elements_pinned_memory_cache),
		ElementsMemoryCache:       uint64(metrics.elements_memory_cache),
		SizePinnedMemoryCache:     uint64(metrics.size_pinned_memory_cache),
		SizeMemoryCache:           uint64(metrics.size_memory_cache),
	}, nil
}

func Instantiate(
	cache Cache,
	checksum []byte,
	env []byte,
	info []byte,
	msg []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	i := MakeView(info)
	defer runtime.KeepAlive(info)
	m := MakeView(msg)
	defer runtime.KeepAlive(msg)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.instantiate(cache.Ptr, cs, e, i, m, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func Execute(
	cache Cache,
	checksum []byte,
	env []byte,
	info []byte,
	msg []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	fmt.Println("---zjg------------Execute--v2")

	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	i := MakeView(info)
	defer runtime.KeepAlive(info)
	m := MakeView(msg)
	defer runtime.KeepAlive(msg)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.execute(cache.Ptr, cs, e, i, m, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func Migrate(
	cache Cache,
	checksum []byte,
	env []byte,
	msg []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	m := MakeView(msg)
	defer runtime.KeepAlive(msg)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.migrate(cache.Ptr, cs, e, m, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func Sudo(
	cache Cache,
	checksum []byte,
	env []byte,
	msg []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	m := MakeView(msg)
	defer runtime.KeepAlive(msg)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.sudo(cache.Ptr, cs, e, m, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func Reply(
	cache Cache,
	checksum []byte,
	env []byte,
	reply []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	r := MakeView(reply)
	defer runtime.KeepAlive(reply)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.reply(cache.Ptr, cs, e, r, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func Query(
	cache Cache,
	checksum []byte,
	env []byte,
	msg []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	m := MakeView(msg)
	defer runtime.KeepAlive(msg)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.query(cache.Ptr, cs, e, m, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func IBCChannelOpen(
	cache Cache,
	checksum []byte,
	env []byte,
	msg []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	m := MakeView(msg)
	defer runtime.KeepAlive(msg)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.ibc_channel_open(cache.Ptr, cs, e, m, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func IBCChannelConnect(
	cache Cache,
	checksum []byte,
	env []byte,
	msg []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	m := MakeView(msg)
	defer runtime.KeepAlive(msg)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.ibc_channel_connect(cache.Ptr, cs, e, m, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func IBCChannelClose(
	cache Cache,
	checksum []byte,
	env []byte,
	msg []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	m := MakeView(msg)
	defer runtime.KeepAlive(msg)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.ibc_channel_close(cache.Ptr, cs, e, m, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func IBCPacketReceive(
	cache Cache,
	checksum []byte,
	env []byte,
	packet []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	pa := MakeView(packet)
	defer runtime.KeepAlive(packet)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.ibc_packet_receive(cache.Ptr, cs, e, pa, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func IBCPacketAck(
	cache Cache,
	checksum []byte,
	env []byte,
	ack []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	ac := MakeView(ack)
	defer runtime.KeepAlive(ack)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.ibc_packet_ack(cache.Ptr, cs, e, ac, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

func IBCPacketTimeout(
	cache Cache,
	checksum []byte,
	env []byte,
	packet []byte,
	gasMeter *utils.GasMeter,
	store utils.KVStore,
	api *utils.GoAPI,
	querier *Querier,
	gasLimit uint64,
	printDebug bool,
) ([]byte, uint64, error) {
	cs := MakeView(checksum)
	defer runtime.KeepAlive(checksum)
	e := MakeView(env)
	defer runtime.KeepAlive(env)
	pa := MakeView(packet)
	defer runtime.KeepAlive(packet)

	callID := StartCall()
	defer EndCall(callID)

	dbState := BuildDBState(store, callID)
	db := BuildDB(&dbState, gasMeter)
	a := BuildAPI(api)
	q := BuildQuerier(querier)
	var gasUsed Cu64
	errmsg := NewUnmanagedVector(nil)

	res, err := C.ibc_packet_timeout(cache.Ptr, cs, e, pa, db, a, q, Cu64(gasLimit), Cbool(printDebug), &gasUsed, &errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}
	return CopyAndDestroyUnmanagedVector(res), uint64(gasUsed), nil
}

/**** To error module ***/

func errorWithMessage(err error, b C.UnmanagedVector) error {
	// this checks for out of gas as a special case
	if errno, ok := err.(syscall.Errno); ok && int(errno) == 2 {
		return types.OutOfGasError{}
	}
	msg := CopyAndDestroyUnmanagedVector(b)
	if msg == nil {
		return err
	}
	return fmt.Errorf("%s", string(msg))
}
