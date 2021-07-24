use crate::log;

extern "C" {
    static _sheap: u8;
}

#[derive(Debug, Copy, Clone)]
struct Allocation {
    address: *const u8,
    size: usize,
    free: bool,
}

static mut ALLOCATIONS: [Option<Allocation>; 128] = [None; 128];
static mut ALLOC_INDEX: isize = -1;

#[no_mangle]
pub unsafe extern "C" fn malloc(size: u32) -> *const u8 {
    log!("malloc called {}", size);

    let mut candidate_addr = &_sheap as *const u8;

    riscv::interrupt::free(|_critical_section| {
        let aligned_size = size + if size % 8 != 0 { 8 - size % 8 } else { 0 };

        // try to find a previously freed block
        let mut reused = 0 as *const u8;
        for allocation in ALLOCATIONS.iter_mut() {
            match allocation {
                Some(ref mut allocation) => {
                    if allocation.free && aligned_size <= allocation.size as u32 {
                        allocation.free = false;
                        reused = allocation.address;
                        break;
                    }
                }
                None => {}
            }
        }

        if reused.is_null() {
            // otherwise allocate after the highest allocated block
            if ALLOC_INDEX != -1 {
                candidate_addr = ALLOCATIONS[ALLOC_INDEX as usize]
                    .unwrap()
                    .address
                    .offset(ALLOCATIONS[ALLOC_INDEX as usize].unwrap().size as isize);
            }

            ALLOC_INDEX += 1;

            ALLOCATIONS[ALLOC_INDEX as usize] = Some(Allocation {
                address: candidate_addr,
                size: aligned_size as usize,
                free: false,
            });
            log!("new allocation idx = {}", ALLOC_INDEX);
        } else {
            log!("new allocation at reused block");
            candidate_addr = reused;
        }

        log!("malloc at {:p}", candidate_addr);
    });

    return candidate_addr;
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *const u8) {
    log!("free called {:p}", ptr);

    if ptr.is_null() {
        return;
    }

    riscv::interrupt::free(|_critical_section| {
        let alloced_idx = ALLOCATIONS
            .iter()
            .enumerate()
            .find(|(_, allocation)| allocation.is_some() && allocation.unwrap().address == ptr);

        if alloced_idx.is_some() {
            let alloced_idx = alloced_idx.unwrap().0;
            log!("free idx {}", alloced_idx);

            if alloced_idx as isize == ALLOC_INDEX {
                ALLOCATIONS[alloced_idx] = None;
                ALLOC_INDEX -= 1;
            } else {
                ALLOCATIONS[alloced_idx] = ALLOCATIONS[alloced_idx as usize]
                    .take()
                    .and_then(|v| Some(Allocation { free: true, ..v }));
            }
        } else {
            panic!("freeing a memory area we don't know of");
        }
    });
}
