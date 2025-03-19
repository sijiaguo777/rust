use interpreter::{Machine, MEMORY_SIZE};
use std::io::{self, Write};
use std::sync::Once;
use log::{debug, error, warn, info};
extern crate pretty_env_logger;
extern crate log;

static INIT: Once = Once::new();
fn init_logger() {
    INIT.call_once(|| {
        pretty_env_logger::init();
    });
}

#[test]
fn create_with_memory() {
    let memory = vec![1, 2, 3];
    let machine = Machine::new(&memory).unwrap();
    assert_eq!(&memory[..], &machine.memory()[..memory.len()],);
    assert!(machine.memory()[memory.len()..].iter().all(|b| *b == 0),);
    assert!(machine.regs().iter().all(|v| *v == 0),);
}

#[test]
fn create_with_too_large_a_memory() {
    assert!(Machine::new(&[0; MEMORY_SIZE + 1]).is_err());
}

#[test]
fn refuse_illegal_instruction() {
    let mut machine = Machine::new(&[]).unwrap();
    assert!(machine.step().is_err());
}

fn expect_on<T: Write>(machine: &mut Machine, fd: &mut T, end: bool, new_ip: usize) {
    match machine.step_on(fd) {
        Ok(r) if r == end => (), 
        _ => panic!(),
    }
    writeln!(fd,"machine.regs()[0]: {}", machine.regs()[0]);
    assert_eq!(new_ip, machine.regs()[0] as usize,);
}

fn expect(machine: &mut Machine, end: bool, new_ip: usize) {
    expect_on(machine, &mut io::stdout().lock(), end, new_ip);
}

#[test]
fn test_exit() {
    // 0: exit
    // 1:
    // init_logger(); 
    let mut machine = Machine::new(&[7]).unwrap();
    expect(&mut machine, true, 1);
}

#[test]
fn ip_reg() {
    // 0: invalid
    // 1: exit
    // 2:
    // init_logger(); 
    let mut machine = Machine::new(&[0,7]).unwrap();
    machine.set_reg(0, 1).unwrap();
    expect(&mut machine, true, 2);
}

#[test]
fn test_move_if() {
    // 0: move r1 <- r2 if r2 != 0
    // 4: move r3 <- r2 if r3 != 0
    // 8:
    let mut machine = Machine::new(&[1, 1, 2, 2, 1, 3, 2, 3]).unwrap();
    machine.set_reg(2, 42).unwrap();
    expect(&mut machine, false, 4);
    assert_eq!(42, machine.regs()[1]);
    expect(&mut machine, false, 8);
    assert_eq!(0, machine.regs()[3]);
}

#[test]
fn test_move_if_out_of_bounds() {
    // 0: move r1 <- r100 if r0 != 0
    // 4:
    let mut machine = Machine::new(&[1, 1, 100, 0]).unwrap();
    assert!(machine.step().is_err());

    // 0: move r100 <- r1 if r0 != 0
    // 4:
    let mut machine = Machine::new(&[1, 100, 1, 0]).unwrap();
    assert!(machine.step().is_err());

    // 0: move r1 <- r1 if r100 != 0
    // 4:
    let mut machine = Machine::new(&[1, 1, 1, 100]).unwrap();
    assert!(machine.step().is_err());
}

#[test]
fn early_ip_advance() {
    // 0: move r1 <- r0 if r0 != 0
    // 4: move r0 <- r1 if r0 != 0
    let mut machine = Machine::new(&[1, 1, 0, 0, 1, 0, 1, 0]).unwrap();
    expect(&mut machine, false, 4);
    assert_eq!(4, machine.regs()[1],);

    expect(&mut machine, false, 4);
    assert_eq!(4, machine.regs()[1]);

    expect(&mut machine, false, 4);
    assert_eq!(4, machine.regs()[1],);
}


#[test]
fn test_load() {
    // 0: load r1 <- [r0]
    // 3: 42
    let mut machine = Machine::new(&[3, 1, 0, 42]).unwrap();
    expect(&mut machine, false, 3);
    assert_eq!(42, machine.regs()[1]);

    // 0: load r0 <- [r0]
    // 4: 42
    let mut machine = Machine::new(&[3, 0, 0, 42]).unwrap();
    expect(&mut machine, false, 42);

    // 0: load r1 <- [r0]
    // 4: 1
    // 5: 2
    // 6: 3
    // 7: 4
    let mut machine = Machine::new(&[3, 1, 0, 1, 2, 3, 4]).unwrap();
    expect(&mut machine, false, 3);
    assert_eq!(0x0403_0201, machine.regs()[1]);
}

#[test]
fn test_load_out_of_bounds() {
    // 0: load r100 <- [r1]
    // 3:
    let mut machine = Machine::new(&[3, 100, 1]).unwrap();
    assert!(machine.step().is_err());

    // 0: load r1 <- [r100]
    // 3:
    let mut machine = Machine::new(&[3, 1, 100]).unwrap();
    assert!(machine.step().is_err());

    // 0: load r1 <- [r1] with r1 == 30000
    // 3:
    let mut machine = Machine::new(&[3, 1, 1]).unwrap();
    machine.set_reg(1, 30000).unwrap();
    assert!(machine.step().is_err());
}


#[test]
fn end_of_memory() {
    // memory_size-1: exit
    // memory_size  :
    let mut memory = [0; MEMORY_SIZE];
    memory[MEMORY_SIZE - 1] = 7;
    let mut machine = Machine::new(&memory).unwrap();
    machine.set_reg(0, (MEMORY_SIZE - 1) as u32).unwrap();
    expect(&mut machine, true, MEMORY_SIZE);
}


#[test]
fn exec_after_end_of_address_space() {
    let mut machine = Machine::new(&[]).unwrap();
    machine.set_reg(0, MEMORY_SIZE as u32).unwrap();
    assert!(machine.step().is_err());
}


#[test]
fn exec_over_end_of_memory() {
    // end-2: sub r1 <- r1 - r1
    // end+2:
    let mut memory = vec![0; MEMORY_SIZE - 3];
    memory.extend(&[5, 1, 1]);
    let mut machine = Machine::new(&memory).unwrap();
    machine.set_reg(0, MEMORY_SIZE as u32 - 3).unwrap();
    assert!(machine.step().is_err());
}

#[test]
fn no_wraparound_past_end_of_memory() {
    // memory_size-4: move r1 <- r1 if r1
    // 0:             exit
    // 1:
    let mut memory = [0; MEMORY_SIZE];
    for i in MEMORY_SIZE - 4..MEMORY_SIZE {
        memory[i] = 1;
    }
    memory[0] = 7;
    let mut machine = Machine::new(&memory).unwrap();
    machine.set_reg(0, (MEMORY_SIZE - 4) as u32).unwrap();
    expect(&mut machine, false, MEMORY_SIZE);
    assert!(machine.step().is_err());
}


#[test]
fn sub_with_wraparound() {
    // 0: sub r1 <- r2 - r1
    // 4:
    let mut machine = Machine::new(&[5, 1, 2, 1]).unwrap();
    machine.set_reg(1, 0xEEEE_EEEE).unwrap();
    expect(&mut machine, false, 4);
    assert_eq!(machine.regs()[1], !0xEEEE_EEEE + 1);
}


#[test]
fn test_out() {
    // 0: out r1
    // 2:
    let mut machine = Machine::new(&[6, 1]).unwrap();
    machine.set_reg(1, u32::from(b'A')).unwrap();
    let mut out = Vec::new();
    expect_on(&mut machine, &mut out, false, 2);
    assert_eq!("A".as_bytes(), &out[..]);
}


#[test]
fn test_out_number() {
    // 0: out_number r0
    // 2:
    let mut machine = Machine::new(&[8, 0]).unwrap();
    let mut out = Vec::new();
    expect_on(&mut machine, &mut out, false, 2);
    assert_eq!("2".as_bytes(), &out[..]);

    // 0: out_number r1
    // 2:
    let mut machine = Machine::new(&[8, 1]).unwrap();
    let mut out = Vec::new();
    machine.set_reg(1, -1234i32 as u32).unwrap();
    expect_on(&mut machine, &mut out, false, 2);
    assert_eq!("-1234".as_bytes(), &out[..]);
}


#[test]
fn test_run_on() {
    // 0: out_number r0
    // 2: out_number r0
    // 4: exit
    // 5:
    let mut machine = Machine::new(&[8, 0, 8, 0, 7]).unwrap();
    let mut out = Vec::new();
    machine.run_on(&mut out).unwrap();
    assert_eq!("24".as_bytes(), &out[..]);
}
