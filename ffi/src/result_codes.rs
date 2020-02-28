pub type ResultCode = u32;

#[no_mangle] pub static ALASS_SUCCESS:                  ResultCode = 0;
#[no_mangle] pub static ALASS_INTERNAL_ERROR:           ResultCode = 1;
#[no_mangle] pub static ALASS_INVALID_PARAMS:           ResultCode = 2;
#[no_mangle] pub static ALASS_SINK_CLOSED:              ResultCode = 3;
#[no_mangle] pub static ALASS_UNSUPPORTED_FORMAT:       ResultCode = 4;
#[no_mangle] pub static ALASS_READ_ERROR:               ResultCode = 5;
#[no_mangle] pub static ALASS_FILE_DOES_NOT_EXIST:      ResultCode = 6;
#[no_mangle] pub static ALASS_PERMISSION_DENIED:        ResultCode = 7;
#[no_mangle] pub static ALASS_PARSE_ERROR:              ResultCode = 8;
#[no_mangle] pub static ALASS_WRITE_ERROR:              ResultCode = 9;
#[no_mangle] pub static ALASS_SERIALIZE_ERROR:          ResultCode = 10;
#[no_mangle] pub static ALASS_LOG_ALREADY_CONFIGURED:   ResultCode = 11;