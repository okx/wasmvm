package api

//import (
//	"github.com/CosmWasm/wasmvm/api/utils"
//	"github.com/CosmWasm/wasmvm/api/v1"
//	"github.com/CosmWasm/wasmvm/api/v2"
//	"github.com/CosmWasm/wasmvm/types"
//)
//
//type Querier = types.Querier
//
//func InitCache(dataDir string, supportedFeatures string, cacheSize uint32, instanceMemoryLimit uint32) (v1.Cache, error) {
//	return v1.InitCache(dataDir, supportedFeatures, cacheSize, instanceMemoryLimit)
//}
//
//func InitCacheV2(dataDir string, supportedFeatures string, cacheSize uint32, instanceMemoryLimit uint32) (v2.Cache, error) {
//	return v2.InitCache(dataDir, supportedFeatures, cacheSize, instanceMemoryLimit)
//}
//
//func ReleaseCache(cache v1.Cache) {
//	v1.ReleaseCache(cache)
//}
//
//func Create(cache v1.Cache, wasm []byte) ([]byte, error) {
//	return v1.Create(cache, wasm)
//}
//
//func GetCode(cache v1.Cache, checksum []byte) ([]byte, error) {
//	return v1.GetCode(cache, checksum)
//}
//
//func Pin(cache v1.Cache, checksum []byte) error {
//	return v1.Pin(cache, checksum)
//}
//
//func Unpin(cache v1.Cache, checksum []byte) error {
//	return Unpin(cache, checksum)
//}
//
//func AnalyzeCode(cache v1.Cache, checksum []byte) (*types.AnalysisReport, error) {
//	return v1.AnalyzeCode(cache, checksum)
//}
//
//func GetMetrics(cache v1.Cache) (*types.Metrics, error) {
//	return v1.GetMetrics(cache)
//}
//
//func Instantiate(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	info []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.Instantiate(cache, checksum, env, info, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func Execute(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	info []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.Execute(cache, checksum, env, info, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func Execute2(
//	cache v2.Cache,
//	checksum []byte,
//	env []byte,
//	info []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v2.Execute(cache, checksum, env, info, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func Migrate(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.Migrate(cache, checksum, env, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func Sudo(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.Sudo(cache, checksum, env, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func Reply(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	reply []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.Reply(cache, checksum, env, reply, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func Query(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.Query(cache, checksum, env, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func IBCChannelOpen(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.IBCChannelOpen(cache, checksum, env, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func IBCChannelConnect(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.IBCChannelConnect(cache, checksum, env, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func IBCChannelClose(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	msg []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.IBCChannelClose(cache, checksum, env, msg, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func IBCPacketReceive(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	packet []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.IBCPacketReceive(cache, checksum, env, packet, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func IBCPacketAck(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	ack []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.IBCPacketAck(cache, checksum, env, ack, gasMeter, store, api, querier, gasLimit, printDebug)
//}
//
//func IBCPacketTimeout(
//	cache v1.Cache,
//	checksum []byte,
//	env []byte,
//	packet []byte,
//	gasMeter *utils.GasMeter,
//	store utils.KVStore,
//	api *utils.GoAPI,
//	querier *Querier,
//	gasLimit uint64,
//	printDebug bool,
//) ([]byte, uint64, error) {
//	return v1.IBCPacketTimeout(cache, checksum, env, packet, gasMeter, store, api, querier, gasLimit, printDebug)
//}

///**** To error module ***/
//
//func errorWithMessage(err error, b C.UnmanagedVector) error {
//	// this checks for out of gas as a special case
//	if errno, ok := err.(syscall.Errno); ok && int(errno) == 2 {
//		return types.OutOfGasError{}
//	}
//	msg := v1.CopyAndDestroyUnmanagedVector(b)
//	if msg == nil {
//		return err
//	}
//	return fmt.Errorf("%s", string(msg))
//}
