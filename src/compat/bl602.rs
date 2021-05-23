use crate::log;

pub static mut ATTACHED_IRQ_HANDLER: [Option<InterruptAttach>; 64 + 16] = [None; 64 + 16];

#[derive(Clone, Copy)]
pub struct InterruptAttach {
    pub isr: extern "C" fn(*const u8, *const u8),
    pub arg: *const u8,
}

pub fn hbn_config_aon_pad_input_and_smt() {
    // HBN Config AON pad input and SMT
    // see bl602_start.c
    unsafe {
        const BL602_HBN_BASE: u32 = 0x4000f000;
        const BL602_HBN_IRQ_MODE_OFFSET: u32 = 0x000014;
        const BL602_HBN_IRQ_MODE: u32 = BL602_HBN_BASE + BL602_HBN_IRQ_MODE_OFFSET;
        const HBN_IRQ_MODE_REG_AON_PAD_IE_SMT: u32 = 1 << 8;

        let ptr = BL602_HBN_IRQ_MODE as *mut u32;
        let mut regval = ptr.read_volatile();
        regval &= !0;
        regval |= HBN_IRQ_MODE_REG_AON_PAD_IE_SMT;
        ptr.write_volatile(regval);
    }
}

pub fn bl602_set_em_sel_bl602_glb_em_8kb() {
    // see bl602_bringup.c
    // Set how much wifi ram is allocated to ble.
    unsafe {
        const BL602_GLB_EM_8KB: u32 = 0x3 /* 8KB */;
        const BL602_GLB_BASE: u32 = 0x40000000;
        const BL602_SEAM_MISC_OFFSET: u32 = 0x00007c;
        const BL602_SEAM_MISC: u32 = BL602_GLB_BASE + BL602_SEAM_MISC_OFFSET;
        const SEAM_MISC_EM_SEL_MASK: u32 = 0x0f;
        let ptr = BL602_SEAM_MISC as *mut u32;
        let mut regval = ptr.read_volatile();
        regval &= !SEAM_MISC_EM_SEL_MASK;
        regval |= BL602_GLB_EM_8KB;
        ptr.write_volatile(regval);
    }
}

#[no_mangle]
pub unsafe extern "C" fn bl602_aon_pad_iesmt_cfg(pad_cfg: u8) {
    log!("bl602_aon_pad_iesmt_cfg called {}", pad_cfg);

    const BL602_HBN_BASE: u32 = 0x4000f000;
    const BL602_HBN_IRQ_MODE_OFFSET: u32 = 0x000014;
    const BL602_HBN_IRQ_MODE: u32 = BL602_HBN_BASE + BL602_HBN_IRQ_MODE_OFFSET;
    const HBN_IRQ_MODE_REG_AON_PAD_IE_SMT: u32 = 1 << 8;

    let ptr = BL602_HBN_IRQ_MODE as *mut u32;
    let mut regval = ptr.read_volatile();
    regval &= HBN_IRQ_MODE_REG_AON_PAD_IE_SMT;
    regval |= (pad_cfg as u32) << 8;
    ptr.write_volatile(regval);
}

#[no_mangle]
pub unsafe extern "C" fn up_enable_irq(irq: i32) {
    // Enable the IRQ specified by 'irq'

    log!("up_enable_irq called for irq {}", irq);

    ((0x02800000 + 0x400 + irq - 16) as *mut u8).write_volatile(1);
}

#[no_mangle]
pub unsafe extern "C" fn irq_attach(
    irq: i32,
    isr: extern "C" fn(*const u8, *const u8),
    arg: *const u8,
) -> i32 {
    // Configure the IRQ subsystem so that IRQ number 'irq' is dispatched to 'isr'

    log!("irq_attach called {} {:p} {:p}", irq, isr, arg);

    ATTACHED_IRQ_HANDLER[(irq - 16) as usize] = Some(InterruptAttach { isr, arg });

    0
}

#[no_mangle]
pub unsafe extern "C" fn up_irq_save() -> u32 {
    // this is not what the original function does but seems to be good enough

    log!("up_irq_save");

    let res = riscv::register::mstatus::read().mie();
    riscv::register::mstatus::clear_mie();

    if res {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn up_irq_restore(flags: u32) {
    // this is not what the original function does but seems to be good enough
    log!("up_irq_restore {}", flags);

    if flags == 1 {
        riscv::register::mstatus::set_mie();
    } else {
        riscv::register::mstatus::clear_mie();
    };
}

pub fn dispatch_irq(irq: usize) {
    unsafe {
        match ATTACHED_IRQ_HANDLER[irq] {
            core::option::Option::Some(data) => {
                let f = data.isr;
                log!("Handling interrupt {} @ {:p} {:p}", irq, f, data.arg);
                f(core::ptr::null(), data.arg);

                log!("Handling interrupt done");
            }
            core::option::Option::None => {
                log!("ooops! unhandled interrupt cause={}", irq);
                loop {}
            }
        }
    }
}
