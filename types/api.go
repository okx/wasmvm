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

type ContractCreateRequest struct {
	Creator   string `json:"creator"`
	WasmCode  []byte `json:"wasm_code"`
	CodeID    uint64 `json:"code_id"`
	InitMsg   []byte `json:"init_msg"`
	AdminAddr string `json:"admin_addr"`
	Label     string `json:"label"`
	IsCreate2 bool   `json:"is_create2"`
	Salt      []byte `json:"salt"`
	//Deposit   Coins  `json:"deposit"`
}
