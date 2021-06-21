/* automatically generated by rust-bindgen 0.58.1 */

#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code
)]

pub const _SAL_VERSION: u32 = 20;
pub const __SAL_H_VERSION: u32 = 180000000;
pub const _USE_DECLSPECS_FOR_SAL: u32 = 0;
pub const _USE_ATTRIBUTES_FOR_SAL: u32 = 0;
pub const _CRT_PACKING: u32 = 8;
pub const _HAS_EXCEPTIONS: u32 = 1;
pub const WCHAR_MIN: u32 = 0;
pub const WCHAR_MAX: u32 = 65535;
pub const WINT_MIN: u32 = 0;
pub const WINT_MAX: u32 = 65535;
pub type va_list = *mut crate::binary::c_types::c_char;
extern "C" {
    pub fn __va_start(arg1: *mut *mut crate::binary::c_types::c_char, ...);
}
pub type size_t = crate::binary::c_types::c_ulonglong;
pub type __vcrt_bool = bool;
pub type wchar_t = crate::binary::c_types::c_ushort;
extern "C" {
    pub fn __security_init_cookie();
}
extern "C" {
    pub fn __security_check_cookie(_StackCookie: usize);
}
extern "C" {
    pub fn __report_gsfailure(_StackCookie: usize);
}
extern "C" {
    pub static mut __security_cookie: usize;
}
pub type int_least8_t = crate::binary::c_types::c_schar;
pub type int_least16_t = crate::binary::c_types::c_short;
pub type int_least32_t = crate::binary::c_types::c_int;
pub type int_least64_t = crate::binary::c_types::c_longlong;
pub type uint_least8_t = crate::binary::c_types::c_uchar;
pub type uint_least16_t = crate::binary::c_types::c_ushort;
pub type uint_least32_t = crate::binary::c_types::c_uint;
pub type uint_least64_t = crate::binary::c_types::c_ulonglong;
pub type int_fast8_t = crate::binary::c_types::c_schar;
pub type int_fast16_t = crate::binary::c_types::c_int;
pub type int_fast32_t = crate::binary::c_types::c_int;
pub type int_fast64_t = crate::binary::c_types::c_longlong;
pub type uint_fast8_t = crate::binary::c_types::c_uchar;
pub type uint_fast16_t = crate::binary::c_types::c_uint;
pub type uint_fast32_t = crate::binary::c_types::c_uint;
pub type uint_fast64_t = crate::binary::c_types::c_ulonglong;
pub type intmax_t = crate::binary::c_types::c_longlong;
pub type uintmax_t = crate::binary::c_types::c_ulonglong;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bl_wifi_ap_info {
    pub ssid: [u8; 33usize],
    pub psk: [u8; 65usize],
    pub chan: u8,
}
pub type bl_wifi_ap_info_t = bl_wifi_ap_info;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _bl_wifi_env {
    pub sta_mac_addr_board: [u8; 6usize],
    pub sta_mac_addr_usr: [u8; 6usize],
    pub ap_mac_addr_board: [u8; 6usize],
    pub ap_mac_addr_usr: [u8; 6usize],
    pub country_code: u8,
    pub ap_info: bl_wifi_ap_info_t,
    pub ap_info_en: u8,
    pub sta_info: bl_wifi_ap_info_t,
    pub sta_info_en: u8,
}
pub type bl_wifi_env_t = _bl_wifi_env;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct net_device {
    pub bl_hw: *mut bl_hw,
}
extern "C" {
    pub fn bl_wifi_enable_irq() -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_clock_enable() -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_sta_mac_addr_set(mac: *mut u8) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_ap_mac_addr_set(mac: *mut u8) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_ap_mac_addr_get(mac: *mut u8) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_mac_addr_set(mac: *mut u8) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_country_code_set(country_code: u8) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_ap_info_set(
        ssid: *mut u8,
        ssid_len: u8,
        psk: *mut u8,
        psk_len: u8,
        chan: u8,
    ) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_mac_addr_get(mac: *mut u8) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_ap_info_get(ap_info: *mut bl_wifi_ap_info_t) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_sta_info_set(
        ssid: *mut u8,
        ssid_len: u8,
        psk: *mut u8,
        psk_len: u8,
        autoconnect: crate::binary::c_types::c_int,
    ) -> crate::binary::c_types::c_int;
}
extern "C" {
    pub fn bl_wifi_sta_info_get(sta_info: *mut bl_wifi_ap_info_t) -> crate::binary::c_types::c_int;
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bl_hw {
    pub _address: u8,
}
