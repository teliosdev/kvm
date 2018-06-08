use super::Core;
use super::Result;

fn dump(core: &mut Core) -> Result<()> {
    let r = core.registers();
    println!(
        "rax: {:016x} rbx: {:016x} rcx: {:016x} rdx: {:016x}",
        r.rax, r.rbx, r.rcx, r.rdx
    );
    println!(
        "rsi: {:016x} rdi: {:016x} rsp: {:016x} rbp: {:016x}",
        r.rsi, r.rdi, r.rsp, r.rbp
    );
    println!(
        " r8: {:016x}  r9: {:016x} r10: {:016x} r11: {:016x}",
        r.r8, r.r9, r.r10, r.r11
    );
    println!(
        "r12: {:016x} r13: {:016x} r14: {:016x} r15: {:016x}",
        r.r12, r.r13, r.r14, r.r15
    );
    println!("rip: {:016x} rflags: {:016x}", r.rip, r.rflags);
}
