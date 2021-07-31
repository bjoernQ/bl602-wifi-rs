use embedded_time::duration::Milliseconds;
use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken};
use smoltcp::wire::EthernetAddress;

use crate::binary::wifi_mgmr::{self, wifi_mgmr_drv_init, CODE_ON_GOT_IP};
use crate::binary::wifi_mgmr_api;
use crate::compat::bl602::{bl602_set_em_sel_bl602_glb_em_8kb, hbn_config_aon_pad_input_and_smt};
use crate::compat::common::EMULATED_TIMER;
use crate::compat::{common::EmulatedTimer, get_time, work_queue::do_work};
use crate::{binary::bl_wifi, compat::queue::SimpleQueue};
use crate::{log, print, println};

extern "C" {
    static mut __wifi_bss_start: u32;

    static mut __wifi_bss_end: u32;

    pub fn wifi_main_init();

    pub fn ipc_emb_notify();

    pub fn wifi_mgmr_tsk_init();

    pub fn bl602_ef_ctrl_read_mac_address(mac: &mut [u8; 6]);

    pub fn bl_free_rx_buffer(p: *const u8);

    pub fn bl_irq_handler();

    pub fn bl_output(bl_hw: *const bl_wifi::bl_hw, p: *mut u8, tot_len: usize, is_sta: i32) -> i32;

    pub static wifiMgmr: wifi_mgmr::wifi_mgmr;

    pub static bl606a0_sta: bl_wifi::net_device;
}

pub static mut WIFI_CONNECTED: bool = false;
struct DataFrame {
    len: usize,
    data: *mut u8,
}

static mut DATA_QUEUE_RX: SimpleQueue<DataFrame> = SimpleQueue::new();

#[link_section = ".wifi_ram.txbuff"]
static mut TX_BUFFER: [u8; 1650] = [0u8; 1650]; // should be a queue
pub static mut TX_QUEUED: bool = false;

static mut SCAN_IN_PROGRESS: bool = false;
static mut LAST_SCAN_RESULT: [Option<ScanItem>; 50] = [None; 50];

pub fn wifi_pre_init() {
    unsafe {
        use core::{mem, ptr};

        let mut sbss = &mut __wifi_bss_start as *mut u32;
        let ebss = &mut __wifi_bss_end as *mut u32;
        while sbss < ebss {
            ptr::write_volatile(sbss, mem::zeroed());
            sbss = sbss.offset(1);
        }
    }

    hbn_config_aon_pad_input_and_smt();

    bl602_set_em_sel_bl602_glb_em_8kb();
}

pub fn wifi_init() {
    let mut mac = get_mac();
    println!("MAC address");
    for x in mac.iter() {
        print!("{:2x} ", *x);
    }
    print!("\r\n");

    let mut conf = crate::binary::wifi_mgmr::wifi_conf_t {
        country_code: [b'E', b'U', 0],
        channel_nums: 13,
    };

    unsafe {
        crate::binary::bl_wifi::bl_wifi_ap_mac_addr_set(&mut mac as *mut _);
        crate::binary::bl_wifi::bl_wifi_sta_mac_addr_set(&mut mac as *mut _);

        let mut my_ssid = [b't', b'e', b's', b't', 0];
        crate::binary::wifi_mgmr_api::wifi_mgmr_sta_ssid_set(&mut my_ssid as *mut _);
        crate::binary::wifi_mgmr_api::wifi_mgmr_sta_mac_set(&mut mac as *mut _);

        wifi_main_init();
        ipc_emb_notify();
        wifi_mgmr_drv_init(&mut conf);

        for _ in 0..250000 {}

        wifi_mgmr_tsk_init();

        crate::binary::wifi_mgmr_api::wifi_mgmr_sta_autoconnect_disable();
    }
}

pub fn get_mac() -> [u8; 6] {
    let mut mac = [0u8; 6];
    unsafe {
        bl602_ef_ctrl_read_mac_address(&mut mac);
    }
    mac
}

pub fn wifi_scan() -> core::result::Result<[Option<ScanItem>; 50], ()> {
    unsafe {
        if SCAN_IN_PROGRESS {
            return Err(());
        }

        SCAN_IN_PROGRESS = true;
        wifi_mgmr::wifi_mgmr_scan(core::ptr::null_mut(), Some(scan_cb));
        while SCAN_IN_PROGRESS {}

        Ok(LAST_SCAN_RESULT.clone())
    }
}

pub fn connect_sta(arg_ssid: &str, arg_psk: &str) {
    let mut ssid = [0u8; 33];
    let mut psk = [0u8; 64];

    ssid[0..(arg_ssid.len())].copy_from_slice(arg_ssid.as_bytes());
    psk[0..(arg_psk.len())].copy_from_slice(arg_psk.as_bytes());

    unsafe {
        wifi_mgmr_api::wifi_mgmr_api_connect(
            &mut ssid as *mut _,
            &mut psk as *mut _,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
        );

        while !WIFI_CONNECTED {
            // wait until we are connected
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn bl602_net_notify(event: u32, data: *mut u8, len: usize) -> i32 {
    // event: notify type, tx done or received new data
    // data: The data of the event, may be NULL
    // len: data length

    println!("bl602_net_notify {} {:p} {}", event, data, len);

    let is_rx = (event & 0x2) != 0;
    let is_tx_done = (event & 0x1) != 0;

    riscv::interrupt::free(|_| {
        if is_rx {
            if !DATA_QUEUE_RX.is_full() {
                DATA_QUEUE_RX.enqueue(DataFrame {
                    len: len,
                    data: data,
                });
            }
        } else if is_tx_done {
            // nothing here
        }
    });

    0
}

#[no_mangle]
pub unsafe extern "C" fn bl602_netdev_free_txbuf(_buf: *mut u8) {
    println!("bl602_netdev_free_txbuf called");
}

#[no_mangle]
pub unsafe extern "C" fn bl602_net_event(evt: u32, val: u32) {
    // evt e.g. CODE_WIFI_ON_CONNECTED, CODE_WIFI_ON_GOT_IP, ...

    println!("bl602_net_event called {} {}", evt, val);

    if evt == CODE_ON_GOT_IP {
        WIFI_CONNECTED = true;
    }
}

pub unsafe extern "C" fn scan_cb(
    _data: *mut crate::binary::c_types::c_void,
    _param: *mut crate::binary::c_types::c_void,
) {
    println!("SCAN CALLBACK");

    for i in 0..50 {
        let item = wifiMgmr.scan_items[i];
        if item.is_used != 0 {
            LAST_SCAN_RESULT[i] = Some(ScanItem {
                ssid: item.ssid.clone(),
                channel: item.channel,
                rssi: item.rssi,
                bssid: item.bssid.clone(),
            });
        } else {
            LAST_SCAN_RESULT[i] = None;
        }
    }

    SCAN_IN_PROGRESS = false;
}

pub fn init_mac(ethernet: &mut smoltcp::iface::EthernetInterface<WifiDevice>) {
    let mac = get_mac();
    let addr = EthernetAddress::from_bytes(&mac);
    ethernet.set_ethernet_addr(addr);
}

pub struct WifiDevice {}

impl WifiDevice {
    pub fn new() -> WifiDevice {
        WifiDevice {}
    }
}

// see https://docs.rs/smoltcp/0.7.1/smoltcp/phy/index.html
impl<'a> Device<'a> for WifiDevice {
    type RxToken = WifiRxToken;

    type TxToken = WifiTxToken;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let available = unsafe { !DATA_QUEUE_RX.is_empty() };

        if available {
            Some((WifiRxToken::default(), WifiTxToken::default()))
        } else {
            None
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(WifiTxToken::default())
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;
        caps.max_burst_size = Some(1);
        caps
    }
}

#[derive(Debug, Default)]
pub struct WifiRxToken {}

impl RxToken for WifiRxToken {
    fn consume<R, F>(self, _timestamp: smoltcp::time::Instant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        unsafe {
            while !DATA_QUEUE_RX.is_empty() {
                let element = DATA_QUEUE_RX.dequeue();

                match element {
                    Some(data) => {
                        let mut buffer = core::slice::from_raw_parts_mut(data.data, data.len);

                        dump_packet_info(&buffer);

                        let res = f(&mut buffer);
                        bl_free_rx_buffer(data.data);
                        return res;
                    }
                    None => {}
                }
            }
        }

        Err(smoltcp::Error::Exhausted)
    }
}

#[derive(Debug, Default)]
pub struct WifiTxToken {}

impl TxToken for WifiTxToken {
    fn consume<R, F>(
        self,
        _timestamp: smoltcp::time::Instant,
        len: usize,
        f: F,
    ) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        // there are 128 bytes needed in front of the data
        let res = unsafe { f(&mut TX_BUFFER[128..(128 + len)]) };

        unsafe {
            dump_packet_info(&TX_BUFFER[128..(128 + len)]);
        }

        match res {
            Ok(_) => {
                let is_sta = 1; // for now we are always STA
                unsafe {
                    if TX_QUEUED {
                        panic!("already some tx pending");
                    }

                    bl_output(
                        bl606a0_sta.bl_hw,
                        (&mut TX_BUFFER as *mut u8).offset(128),
                        len,
                        is_sta,
                    );
                    TX_QUEUED = true;
                }
            }
            Err(_) => {}
        }

        res
    }
}

fn dump_packet_info(buffer: &[u8]) {
    let ef = smoltcp::wire::EthernetFrame::new_unchecked(buffer);
    println!(
        "src={:x?} dst={:x?} type={:x?}",
        ef.src_addr(),
        ef.dst_addr(),
        ef.ethertype()
    );
    match ef.ethertype() {
        smoltcp::wire::EthernetProtocol::Ipv4 => {
            let ip = smoltcp::wire::Ipv4Packet::new_unchecked(ef.payload());
            println!(
                "src={:?} dst={:?} proto={:x?}",
                ip.src_addr(),
                ip.dst_addr(),
                ip.protocol()
            );

            match ip.protocol() {
                smoltcp::wire::IpProtocol::HopByHop => {}
                smoltcp::wire::IpProtocol::Icmp => {}
                smoltcp::wire::IpProtocol::Igmp => {}
                smoltcp::wire::IpProtocol::Tcp => {
                    let tp = smoltcp::wire::TcpPacket::new_unchecked(ip.payload());
                    println!("src={:?} dst={:?}", tp.src_port(), tp.dst_port());
                }
                smoltcp::wire::IpProtocol::Udp => {
                    let up = smoltcp::wire::UdpPacket::new_unchecked(ip.payload());
                    println!("src={:?} dst={:?}", up.src_port(), up.dst_port());
                }
                smoltcp::wire::IpProtocol::Ipv6Route => {}
                smoltcp::wire::IpProtocol::Ipv6Frag => {}
                smoltcp::wire::IpProtocol::Icmpv6 => {}
                smoltcp::wire::IpProtocol::Ipv6NoNxt => {}
                smoltcp::wire::IpProtocol::Ipv6Opts => {}
                smoltcp::wire::IpProtocol::Unknown(_) => {}
            }
        }
        smoltcp::wire::EthernetProtocol::Arp => {
            let ap = smoltcp::wire::ArpPacket::new_unchecked(ef.payload());
            println!(
                "src={:x?} dst={:x?} src proto addr={:x?}",
                ap.source_hardware_addr(),
                ap.target_hardware_addr(),
                ap.source_protocol_addr()
            );
        }
        smoltcp::wire::EthernetProtocol::Ipv6 => {}
        smoltcp::wire::EthernetProtocol::Unknown(_) => {}
    }
}

pub fn trigger_transmit_if_needed() {
    unsafe {
        let trigger = riscv::interrupt::free(|_| {
            if TX_QUEUED {
                TX_QUEUED = false;
                true
            } else {
                false
            }
        });

        if trigger {
            bl_irq_handler();
        }
    }
}

pub extern "C" fn wifi_worker_task1() {
    unsafe {
        loop {
            do_work();

            riscv::interrupt::free(|_| {
                for i in 0..EMULATED_TIMER.len() {
                    EMULATED_TIMER[i] = match &EMULATED_TIMER[i] {
                        Some(old) => {
                            if old.next_notify != 0 && (get_time().0 >= old.next_notify) {
                                log!("trigger timer....");

                                (old.notify_function)();
                                Some(EmulatedTimer {
                                    notify_function: old.notify_function,
                                    interval_secs: old.interval_secs,
                                    next_notify: (get_time()
                                        + Milliseconds::new(old.interval_secs * 1000))
                                    .0,
                                })
                            } else {
                                Some(EmulatedTimer { ..*old })
                            }
                        }
                        None => None,
                    };
                }
            });
        }
    }
}

pub extern "C" fn wifi_worker_task2() {
    loop {
        do_work();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScanItem {
    pub ssid: [u8; 32],
    pub channel: u8,
    pub rssi: i8,
    pub bssid: [u8; 6],
}
