#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_raw_ptr_to_usize_cast)]

#[allow(non_camel_case_types, non_snake_case)]
use core::{fmt::Write, mem::MaybeUninit};

use bl602_hal as hal;
use core::panic::PanicInfo;
use hal::{
    clock::Strict,
    gpio::{Pin16, Pin7, Uart, Uart0Rx, Uart0Tx, UartMux0, UartMux7},
    pac::{self, UART},
    prelude::*,
    serial::*,
};
use smoltcp::{
    iface::{NeighborCache, Routes},
    socket::{TcpSocket, TcpSocketBuffer},
    wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address},
};

use bl602_hal::timer::TimerExt;

static mut GLOBAL_SERIAL: MaybeUninit<
    bl602_hal::serial::Serial<
        UART,
        (
            (Pin16<Uart>, UartMux0<Uart0Tx>),
            (Pin7<Uart>, UartMux7<Uart0Rx>),
        ),
    >,
> = MaybeUninit::uninit();

use bl602wifi::log::set_writer;
use bl602wifi::println;
use bl602wifi::wifi::*;
use bl602wifi::{
    compat::bl602::dispatch_irq,
    timer::{timestamp, wifi_timer_init},
};

mod wifi_config;
use wifi_config::WIFI_PASSWORD;
use wifi_config::WIFI_SSID;

#[riscv_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let parts = dp.GLB.split();

    // the wifi stuff doesn't work when touching the clock
    let clocks = Strict::boot_defaults();

    // Set up uart output. Since this microcontroller has a pin matrix,
    // we need to set up both the pins and the muxs
    let pin16 = parts.pin16.into_uart_sig0();
    let pin7 = parts.pin7.into_uart_sig7();
    let mux0 = parts.uart_mux0.into_uart0_tx();
    let mux7 = parts.uart_mux7.into_uart0_rx();

    // Configure our UART to 115200 Baud, and use the pins we configured above
    let serial = Serial::uart0(
        dp.UART,
        Config::default().baudrate(115_200.Bd()),
        ((pin16, mux0), (pin7, mux7)),
        clocks,
    );
    unsafe {
        *(GLOBAL_SERIAL.as_mut_ptr()) = serial;
    }

    set_writer(get_serial);

    println!("init");

    wifi_pre_init();

    let timers = dp.TIMER.split();
    wifi_timer_init(timers.channel0);

    let mut socket_set_entries: [_; 2] = Default::default();
    let mut sockets = smoltcp::socket::SocketSet::new(&mut socket_set_entries[..]);
    let mut neighbor_cache_storage = [None; 8];
    let neighbor_cache = NeighborCache::new(&mut neighbor_cache_storage[..]);

    let hw_address = EthernetAddress::from_bytes(&[0, 0, 0, 0, 0, 0]);
    let device = WifiDevice::new();

    let ip_addr = IpCidr::new(IpAddress::v4(192, 168, 2, 191), 24);
    let mut ip_addrs = [ip_addr];

    let mut routes_storage = [None; 1];
    let mut routes = Routes::new(&mut routes_storage[..]);
    routes
        .add_default_ipv4_route(Ipv4Address::new(192, 168, 2, 1))
        .ok();

    let mut ethernet = smoltcp::iface::EthernetInterfaceBuilder::new(device)
        .ethernet_addr(hw_address)
        .neighbor_cache(neighbor_cache)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .finalize();

    wifi_init();

    init_mac(&mut ethernet);

    println!("start connect");

    connect_sta(WIFI_SSID, WIFI_PASSWORD);

    let greet_socket = {
        static mut TCP_SERVER_RX_DATA: [u8; 32] = [0; 32];
        static mut TCP_SERVER_TX_DATA: [u8; 32] = [0; 32];

        let tcp_rx_buffer = unsafe { TcpSocketBuffer::new(&mut TCP_SERVER_RX_DATA[..]) };
        let tcp_tx_buffer = unsafe { TcpSocketBuffer::new(&mut TCP_SERVER_TX_DATA[..]) };

        TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer)
    };
    let greet_handle = sockets.add(greet_socket);

    // task should never return
    loop {
        let timestamp = timestamp();
        riscv::interrupt::free(|_| {
            ethernet.poll(&mut sockets, timestamp).ok();
        });

        trigger_transmit_if_needed();

        // Control the "greeting" socket (:4321)
        {
            let mut socket = sockets.get::<TcpSocket>(greet_handle);
            if !socket.is_open() {
                println!(
                    "Listening to port 4321 for greeting, \
                        please connect to the port"
                );
                socket.listen(4321).unwrap();
            }

            if socket.can_send() {
                println!("Send and close.");
                socket.send_slice(&b"Hello World"[..]).ok();
                socket.close();
            }
        }
    }
}

#[export_name = "ExceptionHandler"]
fn custom_exception_handler(_trap_frame: &riscv_rt::TrapFrame) -> ! {
    /*
    0 0 Instruction address misaligned
    0 1 Instruction access fault
    0 2 Illegal instruction
    0 3 Breakpoint
    0 4 Load address misaligned
    0 5 Load access fault
    0 6 Store/AMO address misaligned
    0 7 Store/AMO access fault
    0 8 Environment call from U-mode
    0 9 Environment call from S-mode
    0 10 Reserved
    0 11 Environment call from M-mode
    0 12 Instruction page fault
    0 13 Load page fault
    0 14 Reserved
    0 15 Store/AMO page fault
    */

    let mepc = riscv::register::mepc::read();
    let code = riscv::register::mcause::read().code() & 0xff;
    println!("exception code {} at {:x}", code, mepc);
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
fn DefaultHandler() {
    let irq = riscv::register::mcause::read().code() & 0xff;
    dispatch_irq(irq);
}

fn get_serial() -> &'static mut dyn core::fmt::Write {
    unsafe { &mut *GLOBAL_SERIAL.as_mut_ptr() }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let serial = unsafe { &mut *(GLOBAL_SERIAL.as_mut_ptr()) };
    write!(serial, "PANIC! {:?}", info).ok();
    loop {}
}
