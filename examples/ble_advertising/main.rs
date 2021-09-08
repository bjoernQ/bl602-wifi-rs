#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_raw_ptr_to_usize_cast)]

#[allow(non_camel_case_types, non_snake_case)]
use core::{fmt::Write, mem::MaybeUninit};

use bl602_hal as hal;
use ble_hci::{Ble, Data, acl::{encode_acl_packet, BoundaryFlag, HostBroadcastFlag}, ad_structure::{AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE, create_advertising_data}, att::{ATT_READ_BY_GROUP_TYPE_REQUEST_OPCODE, ATT_READ_BY_TYPE_REQUEST_OPCODE, AttErrorCode, AttributeData, AttributePayloadData, Uuid, att_encode_error_response, att_encode_read_by_group_type_response, att_encode_read_by_type_response, att_encode_read_response, att_encode_write_response, parse_att}, attribute_server::{ATT_READABLE, ATT_WRITEABLE, AttributeServer, Service}, l2cap::{encode_l2cap, parse_l2cap}};
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

use bl602wifi::log::set_writer;
use bl602wifi::wifi::*;
use bl602wifi::{ble::ble_init, println};
use bl602wifi::{
    ble::{controller::BleConnector, send_hci},
    timer::wifi_timer_init,
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

    let connector = BleConnector {};
    let mut ble = Ble::new(&connector);

    println!("{:?}", ble.init());
    println!("{:?}", ble.cmd_set_le_advertising_parameters());
    println!(
        "{:?}",
        ble.cmd_set_le_advertising_data(
            create_advertising_data(
                &[
                    AdStructure::Flags(LE_GENERAL_DISCOVERABLE|BR_EDR_NOT_SUPPORTED),
                    AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                    AdStructure::CompleteLocalName("BL602 BLE"),
                ]
            )            
        )
    );
    println!("{:?}", ble.cmd_set_le_advertise_enable(true));

    println!("started advertising");

    let mut rf = || Data::new(&[b'H',b'e',b'l',b'l',b'o',]);
    let mut wf = |data: Data| {
        println!("{:x?}", data.to_slice());
    };

    let srv1 = Service::new(
        Uuid::Uuid128([
            0xC9, 0x15, 0x15, 0x96, 0x54, 0x56, 0x64, 0xB3, 0x38, 0x45, 0x26, 0x5D, 0xF1, 0x62,
            0x6A, 0xA8,
        ]),
        ATT_READABLE | ATT_WRITEABLE,
        &mut rf,
        &mut wf,
    );

    let mut rf2 = || Data::default();
    let mut wf2 = |_data| {};

    let srv2 = Service::new(
        Uuid::Uuid128([
            0xC8, 0x15, 0x15, 0x96, 0x54, 0x56, 0x64, 0xB3, 0x38, 0x45, 0x26, 0x5D, 0xF1, 0x62,
            0x6A, 0xA8,
        ]),
        ATT_WRITEABLE,
        &mut rf2,
        &mut wf2,
    );

    let services = &mut [srv1, srv2];
    let mut srv = AttributeServer::new(&mut ble, services);

    loop {
        match srv.do_work() {
            Ok(_) => (),
            Err(err) => { println!("{:?}", err); },
        }

        for _ in 0..10000 {
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

fn get_serial() -> &'static mut dyn core::fmt::Write {
    unsafe { &mut *GLOBAL_SERIAL.as_mut_ptr() }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let serial = unsafe { &mut *(GLOBAL_SERIAL.as_mut_ptr()) };
    write!(serial, "PANIC! {:?}\r\n", info).ok();
    loop {}
}
