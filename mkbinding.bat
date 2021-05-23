REM Problem: pub type WIFI_MGMR_CONNECTION_STATUS = crate::c_types::c_int; should be a u8
REM To match what the real representation is
REM There are some options but none seem to fit?
REM Maybe just do some regex?
REM

bindgen --raw-line "#![allow(non_camel_case_types,non_snake_case,non_upper_case_globals,dead_code)]" --use-core --ctypes-prefix "crate::binary::c_types" --no-layout-tests nuttx\wifi_manager\bl_wifi.h >src\binary\bl_wifi.rs
bindgen --raw-line "#![allow(non_camel_case_types,non_snake_case,non_upper_case_globals,dead_code)]" --use-core --ctypes-prefix "crate::binary::c_types" --no-layout-tests nuttx\wifi_manager\wifi_mgmr.h >src\binary\wifi_mgmr.rs -- -I./nuttx/
bindgen --raw-line "#![allow(non_camel_case_types,non_snake_case,non_upper_case_globals,dead_code)]" --use-core --ctypes-prefix "crate::binary::c_types" --no-layout-tests nuttx\wifi_manager\wifi_mgmr_api.h >src\binary\wifi_mgmr_api.rs -- -I./nuttx/
