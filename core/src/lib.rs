use rand::random;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

pub struct Cpu {
    // program counter
    pub pc: u16,
    // ram
    pub ram: [u8; RAM_SIZE],
    // screen
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HIGHT],
    // v registers
    pub v_reg: [u8; NUM_REGS],
    // index registers
    pub i_reg: u16,
    // stack pointer
    pub sp: u16,
    // stack
    pub stack: [u16; STACK_SIZE],
    // keys
    pub keys: [bool; NUM_KEYS],
    // delay timer
    pub dt: u8,
    // sound timer
    pub st: u8,
}

const START_ADDR: u16 = 0x200;

impl Cpu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & execute
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higer_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higer_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // NOP
            (0x0, 0x0, 0x0, 0x0) => return,
            // CLS
            (0x0, 0x0, 0xE, 0x0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HIGHT],
            // RET
            (0x0, 0x0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            // JMP NNN
            (0x1, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = nnn;
            }
            // CALL NNN
            (0x2, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            // SKIP IF VX == NN
            (0x3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xF00) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            // SKIP IF VX != NN
            (0x4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xF00) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }
            // SKIP IF VX == VY
            (0x5, _, _, 0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // VX = NN
            (0x6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }
            // VX += NN
            (0x7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            // VX = VY
            (0x8, _, _, 0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }
            // VX |= VY
            (0x8, _, _, 0x1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            // VX &= VY
            (0x8, _, _, 0x2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }
            // VX ^= VY
            (0x8, _, _, 0x3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }
            // VX += VY
            (0x8, _, _, 0x4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            // VX -= VY
            (0x8, _, _, 0x5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            // VX >>= 1
            (0x8, _, _, 0x6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 0x1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            }
            // VX = VY - VX
            (0x8, _, _, 0x7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            // VX <<= 1
            (0x8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] & 0x80) >> 7;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            }
            // SKIP IF VX != VY
            (0x9, _, _, 0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
            }
            // JUMP TO V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = self.v_reg[0] as u16 + nnn;
            }
            // VX = rand() & NN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rand_byte = random::<u8>();
                self.v_reg[x] = rand_byte & nn;
            }
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                let num_rows = digit4;
                let mut flipped = false;
                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        if (pixels & (0x80 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HIGHT;
                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }
            // (0xE, _, 0x9, 0xE) => self.skp_vx(op),
            // (0xE, _, 0xA, 0x1) => return,
            (_, _, _, _) => unimplemented!("Unknown opcode: {:#06x}", op),
        }
    }

    pub fn tick_timer(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }
}

