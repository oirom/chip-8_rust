pub struct Memory {
  pub reserved: [u8; 512],
  pub rom: [u8; 3584],
}

pub struct Keypad {
    pub keys: [bool; 16],
}

pub struct Display {
    pub memory: [u8; 2048],
}

pub struct Cpu {
    // index register
    pub i: u16,
    // program counter
    pub pc: u16,
    // memory
    pub memory: Memory,
    // registers
    pub v: [u8; 16],
    // th
    pub keypad: Keypad,
    // display
    pub display: Display,
    // stack
    pub stack: [u16; 16],
    // stack pointer
    pub sp: u8,
    // delay timer
    pub dt: u8,
}

fn read_word(memory: Memory, index: u16) -> u16 {
    (memory[index as usize] as u16) << 8 | memory[(index + 1) as usize] as u16
}

fn main() {
    println!("Hello, world!");
}
