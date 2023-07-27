package types

type (
	HumanizeAddress     func([]byte) (string, uint64, error)
	CanonicalizeAddress func(string) ([]byte, uint64, error)
	GetCallInfo         func(contractAddress, storeAddress string) ([]byte, uint64, KVStore, Querier, GasMeter, error)
	TransferCoins       func(contractAddress, caller string, coins []byte) (uint64, error)
)

type GoAPI struct {
	HumanAddress     HumanizeAddress
	CanonicalAddress CanonicalizeAddress
	GetCallInfo      GetCallInfo
	TransferCoins    TransferCoins
}
