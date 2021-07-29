#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_raw_ptr_to_usize_cast)]

#[allow(non_camel_case_types, non_snake_case)]
use core::{fmt::Write, mem::MaybeUninit};

use bl602_hal as hal;
use core::panic::PanicInfo;
use hal::{
    clock::{Strict, SysclkFreq, UART_PLL_FREQ},
    gpio::{Pin16, Pin7, Uart, Uart0Rx, Uart0Tx, UartMux0, UartMux7},
    pac::{self, UART},
    prelude::*,
    serial::*,
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

use bl602wifi::println;
use bl602wifi::timer::wifi_timer_init;
use bl602wifi::wifi::*;
use bl602wifi::{compat::common::StrBuf, log::set_writer};

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

    let scan_result = wifi_scan();

    match scan_result {
        Ok(scan_result) => {
            for item in &scan_result {
                match item {
                    Some(item) => {
                        let ssid = unsafe { StrBuf::from(&item.ssid as *const u8) };
                        println!("SSID: {}", ssid.as_str_ref());
                        println!("BSSID: {:x?}", item.bssid);
                        println!("CHANNEL: {}", item.channel);
                        println!("RSSI: {}", item.rssi);
                        println!("");
                    }
                    None => (),
                }
            }
        }
        Err(_) => {
            println!("Some error occured")
        }
    }

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
    write!(serial, "PANIC! {:?}", info).ok();
    loop {}
}
