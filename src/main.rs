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

use bl602_hal::interrupts::*;
use bl602_hal::timer::TimerExt;
use bl602_hal::timer::*;
use embedded_time::duration::Milliseconds;

static mut GLOBAL_SERIAL: MaybeUninit<
    bl602_hal::serial::Serial<
        UART,
        (
            (Pin16<Uart>, UartMux0<Uart0Tx>),
            (Pin7<Uart>, UartMux7<Uart0Rx>),
        ),
    >,
> = MaybeUninit::uninit();
static mut CH0: MaybeUninit<ConfiguredTimerChannel0> = MaybeUninit::uninit();
static mut CH1: MaybeUninit<ConfiguredTimerChannel1> = MaybeUninit::uninit();

mod preemt;
use preemt::*;

mod compat;
use compat::bl602::dispatch_irq;

mod wifi;
use wifi::*;

mod log;

mod binary;

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

    println!("start");

    wifi_pre_init();

    let timers = dp.TIMER.split();
    let ch0 = timers
        .channel0
        .set_clock_source(ClockSource::Clock1Khz, 1_000u32.Hz());
    ch0.enable_match0_interrupt();
    ch0.set_preload_value(Milliseconds::new(0));
    ch0.set_preload(hal::timer::Preload::PreloadMatchComparator0);
    ch0.set_match0(Milliseconds::new(10u32));

    hal::interrupts::enable_interrupt(hal::interrupts::Interrupt::TimerCh0);
    unsafe {
        *(CH0.as_mut_ptr()) = ch0;
    }

    let ch1 = timers
        .channel1
        .set_clock_source(ClockSource::Clock1Khz, 1_000u32.Hz());
    ch1.free_running_mode();
    unsafe {
        *(CH1.as_mut_ptr()) = ch1;
    }
    get_ch1().enable(); // start timer
    compat::set_time_source(get_time);

    println!("done");

    task_create(wifi_worker_task1);
    task_create(wifi_worker_task2);

    get_ch0().enable(); // start timer for tasks

    unsafe {
        riscv::interrupt::enable();
    }

    let mut socket_set_entries: [_; 2] = Default::default();
    let mut sockets = smoltcp::socket::SocketSet::new(&mut socket_set_entries[..]);
    let mut neighbor_cache_storage = [None; 8];
    let neighbor_cache = NeighborCache::new(&mut neighbor_cache_storage[..]);

    let hw_address = EthernetAddress::from_bytes(&[0, 0, 0, 0, 0, 0]);
    let device = WifiDevice {};

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

    println!("let's go..");

    wifi_init();

    let mac = get_mac();
    let addr = EthernetAddress::from_bytes(&mac);
    ethernet.set_ethernet_addr(addr);

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
        let timestamp = smoltcp::time::Instant::from_millis(get_ch1().current_time().0);
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

#[allow(non_snake_case)]
#[no_mangle]
fn TimerCh0(trap_frame: &mut TrapFrame) {
    get_ch0().clear_match0_interrupt();
    task_switch(trap_frame);
}

fn get_ch0() -> &'static mut ConfiguredTimerChannel0 {
    unsafe { &mut *CH0.as_mut_ptr() }
}

fn get_ch1() -> &'static mut ConfiguredTimerChannel1 {
    unsafe { &mut *CH1.as_mut_ptr() }
}

fn get_time() -> Milliseconds {
    get_ch1().current_time()
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
