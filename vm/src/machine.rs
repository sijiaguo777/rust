use std::io::{self, Write};
use log::{debug, error, warn, info};

pub const MEMORY_SIZE: usize = 4096;

extern crate pretty_env_logger;
extern crate log;

//////////////////////
const NREGS: usize = 16;

const IP: usize = 0;

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Machine {
    regs: [u32; NREGS],
    memory: [u8; MEMORY_SIZE],
}

#[derive(Debug)]
pub enum Error {
    /// Attempt to create a machine with too large a memory
    MemoryOverflow,
    // Add some more entries to represent different errors
    InvalidRegister,
    BadRegisterName,
    UnknownInstruction,
    IoError(std::io::Error),
}

impl Machine {
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Errors
    /// This function returns an error when the memory exceeds `MEMORY_SIZE`.
    pub fn new(memory: &[u8]) -> Result<Self> {
        if memory.len() > MEMORY_SIZE {
            Err(Error::MemoryOverflow)
        } else {
            let mut mem : [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
            mem[0..memory.len()].copy_from_slice(memory);
            Machine {memory: mem, regs: [0; NREGS]};
            Ok(Machine { memory: mem, regs: [0; NREGS] })
        }
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<()> {
        while !self.step_on(fd)? {}
        Ok(())
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    pub fn run(&mut self) -> Result<()> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool> {
        // init_logger();
        let ip = self.regs[IP] as usize;

        if ip >= MEMORY_SIZE {
            return Err(Error::MemoryOverflow);
        }

        let opcode = self.memory[ip];
        let instr_len = match opcode {
            1 => 4,
            2 => 3,
            3 => 3,
            4 => 4,
            5 => 4,
            6 => 2,
            7 => 1,
            8 => 2,
            _ => return Err(Error::UnknownInstruction),
        };

        if ip + instr_len > MEMORY_SIZE {
            return Err(Error::MemoryOverflow);
        }

        let new_ip = (ip + instr_len) as u32;
        self.set_reg(IP, new_ip).unwrap();
        let instr: &[u8] = &(self.memory)[ip..ip+instr_len];

        match opcode {
            1 => self.move_if(instr[1].into(), instr[2].into(), instr[3].into()),
            2 => self.store(instr[1].into(), instr[2].into()),
            3 => self.load(instr[1].into(), instr[2].into()),
            4 => self.loadimm(instr[1].into(), instr[2], instr[3]),
            5 => self.sub(instr[1].into(), instr[2].into(), instr[3].into()),
            6 => self.out(instr[1].into(), fd),
            7 => Ok(true), 
            8 => self.out_number(instr[1].into(), fd),
            _ => Err(Error::UnknownInstruction),
        }
    }

    /// Similar to [`step_on`](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    #[must_use]
    pub fn regs(&self) -> &[u32] {
        &self.regs
    }

    
    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<()> {
        if reg <= NREGS {
            self.regs[reg] = value;
            Ok(())
        } else {
            Err(Error::InvalidRegister)
        }
    }


    /// Reference onto the machine current memory.
    #[must_use]
    pub fn memory(&self) -> &[u8] {
        &self.memory
    }


    pub fn move_if(&mut self, reg1: usize, reg2: usize, reg3: usize) -> Result<bool> {
        // info!("move_if input: reg1 = {}, reg2 = {}, reg3 = {}", reg1, reg2, reg3);
        if reg1 >= NREGS || reg2 >= NREGS || reg3 >= NREGS {
            return Err(Error::BadRegisterName);
        }
        let cond = self.regs[reg3];
        // writeln!(io::stdout().lock(),"cond: {}", cond);
        if cond != 0 {
            // info!("reg1 = {}, reg2 = {}", reg1, reg2);
            // info!("reg1 value = {}", self.regs[reg1]);
            // info!("reg2 value = {}", self.regs[reg2]);
            self.set_reg(reg1, self.regs[reg2] as u32)?;
            // info!("reg1 value = {}", self.regs[reg1]);
            // info!("reg2 value = {}", self.regs[reg2]);         
        }
        else {
            // do nothing
            return Ok(false)
        }
        Ok(false)
    }


    pub fn store(&mut self, reg1: usize, reg2: usize) -> Result<bool> {
        if reg1 >= NREGS || reg2 >= NREGS {
            return Err(Error::BadRegisterName);
        }
        let addr = self.regs[reg1] as usize;
        if addr + 4 > MEMORY_SIZE {
            return Err(Error::MemoryOverflow);
        }
        let data = self.regs[reg2].to_le_bytes();
        self.memory[addr..addr + 4].copy_from_slice(&data);
        Ok(false)
    }


    pub fn load(&mut self, reg1: usize, reg2: usize) -> Result<bool> {
        if (reg1 < NREGS) && (reg2 < NREGS) {
        // load content of memory starting at add stored in reg2 into reg1
        let addr = self.regs[reg2] as usize;
        if addr + 4 > MEMORY_SIZE {
            return Err(Error::MemoryOverflow);
        }
        let values: &[u8] = &self.memory[addr..addr + 4];
        let value = u32::from_le_bytes(values.try_into().unwrap());
        self.set_reg(reg1,value).unwrap();

        if reg1 == IP {
            self.set_reg(IP, value).unwrap(); // IP 变成新值
        }
        Ok(false)
    }
    else {
        Err(Error::BadRegisterName)
    }
    }


    pub fn loadimm(&mut self, reg1: usize, l: u8, h: u8) -> Result<bool> {
        if reg1 >= NREGS {
            Err(Error::BadRegisterName)
        }
        else {
            let value : i32 = i16::from_le_bytes([l, h]) as i32;
            self.set_reg(reg1, value as u32).unwrap();
            Ok(false)
    }
}


    pub fn sub(&mut self, reg1: usize, reg2: usize, reg3: usize) -> Result<bool> {
        if reg1 < NREGS && reg2 < NREGS && reg3 < NREGS {
            let result = self.regs[reg2].wrapping_sub(self.regs[reg3]);
            self.set_reg(reg1, result)?;
            Ok(false)
        } else {
            Err(Error::BadRegisterName)
        }
    }


    pub fn out<T: Write>(&mut self, reg1: usize,fd: &mut T) -> Result<bool> {
        // let content be the 8 low bits of reg1
        if reg1 < NREGS {
            let content = (self.regs[reg1] & 0xFF) as u8;
            let output : char = content as char;
            write!(fd, "{}", output).map_err(Error::IoError)?;
            Ok(false)
        } else {
            Err(Error::BadRegisterName)
        }
    }


    pub fn out_number<T: Write>(&mut self, reg1: usize, fd: &mut T) -> Result<bool> {
        // signed number in reg1 in decimal
        let signed_value = self.regs[reg1] as i32;
        write!(fd, "{}", signed_value).map_err(Error::IoError)?;
        Ok(false)
    }
}
