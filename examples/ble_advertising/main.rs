#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_raw_ptr_to_usize_cast)]

#[allow(non_camel_case_types, non_snake_case)]
use core::{fmt::Write, mem::MaybeUninit};

use bl602_hal as hal;
use bluetooth_hci::{
    host::{AdvertisingParameters, Channels},
    BdAddr,
};
use core::{panic::PanicInfo, time::Duration};
use hal::{
    clock::{Strict, SysclkFreq, UART_PLL_FREQ},
    gpio::{Pin16, Pin7, Uart, Uart0Rx, Uart0Tx, UartMux0, UartMux7},
    pac::{self, UART},
    prelude::*,
    serial::*,
};
use nb::block;

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

use bl602wifi::timer::wifi_timer_init;
use bl602wifi::wifi::*;
use bl602wifi::{ble::ble_init, println};
use bl602wifi::{
    ble::controller::{Bl602Event, BleController, BusError},
    log::set_writer,
};

#[riscv_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut parts = dp.GLB.split();

    let clocks = Strict::new()
        .use_pll(40_000_000u32.Hz())
        .sys_clk(SysclkFreq::Pll160Mhz)
        .uart_clk(UART_PLL_FREQ.Hz())
        .freeze(&mut parts.clk_cfg);

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
    wifi_timer_init(timers.channel0, dp.HBN);

    wifi_init();

    for _ in 0..20000 {}

    ble_init();

    let mut ble_controller = BleController::new();
    let hci = &mut ble_controller
        as &mut dyn bluetooth_hci::host::uart::Hci<BusError, Bl602Event, BusError, VS = _>;

    block!(hci.reset()).unwrap();
    let res = block!(hci.read()).unwrap();
    println!("{:?}", res);

    let params = AdvertisingParameters {
        advertising_interval: bluetooth_hci::host::AdvertisingInterval::for_type(
            bluetooth_hci::host::AdvertisingType::ConnectableUndirected,
        )
        .with_range(Duration::from_millis(250), Duration::from_millis(500))
        .unwrap(),
        own_address_type: bluetooth_hci::host::OwnAddressType::Public,
        peer_address: bluetooth_hci::BdAddrType::Random(BdAddr([0, 0, 0, 0, 0, 0])),
        advertising_channel_map: Channels::CH_37,
        advertising_filter_policy:
            bluetooth_hci::host::AdvertisingFilterPolicy::AllowConnectionAndScan,
    };
    block!(hci.le_set_advertising_parameters(&params)).unwrap();
    let res = block!(hci.read()).unwrap();
    println!("{:?}", res);

    let data = [
        0x02, 0x01, 0x06, 0x03, 0x03, 0x09, 0x18, 0x14, 0x09, 
        0x42, 0x4C, 0x2D, 0x36, 0x30, 0x32, 0x20, 0x42, 0x6C, 0x65, 0x2D, 0x45, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, 0x21,  
    ];
    block!(hci.le_set_advertising_data(&data)).unwrap();
    let res = block!(hci.read()).unwrap();
    println!("{:?}", res);

    block!(hci.le_set_advertise_enable(true)).unwrap();
    let res = block!(hci.read()).unwrap();
    println!("{:?}", res);

    // loop {
    //     let res = block!(hci.read());
    //     println!("{:?}", res);
    // }

    println!("done.");

    loop {}
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

fn get_serial() -> &'static mut dyn core::fmt::Write {
    unsafe { &mut *GLOBAL_SERIAL.as_mut_ptr() }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let serial = unsafe { &mut *(GLOBAL_SERIAL.as_mut_ptr()) };
    write!(serial, "PANIC! {:?}\r\n", info).ok();
    loop {}
}
