package v1

// #include <stdlib.h>
// #include "bindings.h"
import "C"

// Value types
type (
	Cint   = C.int
	Cbool  = C.bool
	Cusize = C.size_t
	cu8    = C.uint8_t
	Cu32   = C.uint32_t
	Cu64   = C.uint64_t
	ci8    = C.int8_t
	Ci32   = C.int32_t
	ci64   = C.int64_t
)

// Pointers
type Cu8_ptr = *C.uint8_t

type Cache struct {
	Ptr *C.cache_t
}
