package types

type (
	HumanizeAddress     func([]byte) (string, uint64, error)
	CanonicalizeAddress func(string) ([]byte, uint64, error)
	Contract            func(request ContractCreateRequest, gasLimit uint64) (string, uint64, error)
)

type GoAPI struct {
	HumanAddress     HumanizeAddress
	CanonicalAddress CanonicalizeAddress
	Contract         Contract
}

type ContractCreateRequest struct {
	Creator   string `json:"creator"`
	WasmCode  []byte `json:"wasm_code"`
	InitMsg   []byte `json:"init_msg"`
	AdminAddr string `json:"admin_addr"`
	Label     string `json:"label"`
	IsCreate2 bool   `json:"is_create2"`
	Salt      []byte `json:"salt"`
	//Deposit   Coins  `json:"deposit"`
}
