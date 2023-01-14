import {keyGen1} from "../../bridge_tss_rust_ffi/index"


const keygen1_ret = keyGen1(1,1,1)

// send coificients to others
keygen1_ret.coificients
