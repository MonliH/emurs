use crate::disasm;

struct ConditionCodes {
    z: bool,
    s: bool,
    p: bool,
    cy: bool,
    ac: bool,
    pad: bool
}

pub struct State<'a> {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    g: u8,
    h: u8,
    l: u8,

    sp: u16,
    pc: usize,

    cc: ConditionCodes,
    mem: &'a mut [u8],

    int_enable: u16
}

impl State<'_> {
    pub fn new(mem: &mut [u8]) -> State {
        State {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            g: 0,
            h: 0,
            l: 0,

            sp: 0,
            pc: 0,

            cc: ConditionCodes {
                z:   true,
                s:   true,
                p:   true,
                cy:  true,
                ac:  true,
                pad: true
            },

            mem,

            int_enable: 0
        }
    }

    pub fn start(&mut self) {
        let mut running = true;
        while running {
            running = self.step();
        }
    }

    fn extend(first: u8, second: u8) -> u16 {
        ((first as u16) << 8) | second as u16
    }

    fn seperate(value: u16) -> (u8, u8) {
        (((value & 0xFF00) >> 8) as u8, (value & 0x00FF) as u8)
    }

    fn zero_flag(&mut self, answer: u16) {
        self.cc.z = answer == 0;
    }

    fn sign_flag(&mut self, answer: u16) {
        self.cc.s = (answer & 0b10000000) == 0b10000000;
    }

    fn carry_flag(&mut self, answer: u16) {
        self.cc.cy = answer > 0xFF;
    }

    fn parity(mut x: u16) -> bool {
        x ^= x >> 8;
        x ^= x >> 4;
        x ^= x >> 2;
        x ^= x >> 1;
        ((!x) & 1) != 0
    }

    fn parity_flag(&mut self, answer: u16) {
        self.cc.p = Self::parity(answer);
    }

    fn arith_flags(&mut self, answer: u16) {
        self.zero_flag(answer);
        self.sign_flag(answer);
        self.parity_flag(answer & 0xFF);
    }

    fn assign_ref(assigns: (&mut u8, &mut u8), value: (u8, u8)) {
        *assigns.0 = value.1;
        *assigns.1 = value.0;
    }

    fn step(&mut self) -> bool {
        match self.mem[self.pc] {
            0x00 => {} // NOP
            0x01 => {
                // LXI B, word
                self.c = self.mem[self.pc + 1];
                self.b = self.mem[self.pc + 2];
                self.pc += 2;
            }

            0x02 => {
                // STAX B
                let offset = Self::extend(self.b, self.c) as usize;
                self.mem[offset] = self.a;
            }

            0x03 => {
                // INX B
                let answer = Self::extend(self.b, self.c) + 1;
                
                Self::assign_ref((&mut self.b, &mut self.c), Self::seperate(answer));
            }

            0x04 => {
                // INR B
                let answer = self.b as u16 + 1; 
                self.arith_flags(answer);
                self.b = (answer & 0xff) as u8;
            }

            0x05 => {
                // DCR B
                let answer = self.b as u16 - 1; 
                self.arith_flags(answer);
                self.b = (answer & 0xff) as u8;
            }

            0x06 => {
                // MVI B, byte
                self.b = self.mem[self.pc + 1];
                self.pc += 1;
            }

            0x07 => {
                // RLC
                self.a = self.a.rotate_left(1);
                self.cc.cy = (self.a & 0x01) == 0x01;
            }

            0x08 => {
                // DAD B
                let hl = Self::extend(self.h, self.l);
                let bc = Self::extend(self.b, self.c);

                let answer: u16 = hl + bc;

                Self::assign_ref((&mut self.h, &mut self.l), Self::seperate(answer));
                
                self.carry_flag(answer);
            }

            0x0A => {
                // LDAX B
                let offset = Self::extend(self.b, self.c) as usize;
                self.a = self.mem[offset];
            }

            0x0B => {
                // INX B
                let answer = Self::extend(self.b, self.c) - 1;
                
                Self::assign_ref((&mut self.b, &mut self.c), Self::seperate(answer));
            }

            0x0C => {
                // INR C
                let answer = self.c as u16 + 1; 
                self.arith_flags(answer);
                self.c = (answer & 0xff) as u8;
            }

            0x0D => {
                // DCR C
                let answer = self.c as u16 - 1; 
                self.arith_flags(answer);
                self.c = (answer & 0xff) as u8;
            }

            0x0E => {
                // MVI C, byte
                self.c = self.mem[self.pc + 1];
                self.pc += 1;
            }

            0x0F => {
                // RRC
                self.a = self.a.rotate_right(1);
                self.cc.cy = (self.a & 0x01) == 0x01;
            }

            0x40 => { self.b = self.b; } // MOV B,B
            0x41 => { self.b = self.c; } // MOV B,C
            0x42 => { self.b = self.d; } // MOV B,D
            0x43 => { self.b = self.e; } // MOV B,E
            0x44 => { self.b = self.h; } // MOV B,H
            0x45 => { self.b = self.l; } // MOV B,L
            0x46 => {                    // MOV B,M
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.b = self.mem[offset];
            }
            0x47 => { self.b = self.a; } // MOV B,A

            0x48 => { self.c = self.b; } // MOV C,B
            0x49 => { self.c = self.c; } // MOV C,C
            0x4A => { self.c = self.d; } // MOV C,D
            0x4B => { self.c = self.e; } // MOV C,E
            0x4C => { self.c = self.h; } // MOV C,H
            0x4D => { self.c = self.l; } // MOV C,L
            0x4E => {                    // MOV C,M
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.b = self.mem[offset];
            }
            0x4F => { self.c = self.a; } // MOV C,A
            
            0x50 => { self.d = self.b; } // MOV D,B
            0x51 => { self.d = self.c; } // MOV D,C
            0x52 => { self.d = self.d; } // MOV D,D
            0x53 => { self.d = self.e; } // MOV D,E
            0x54 => { self.d = self.h; } // MOV D,H
            0x55 => { self.d = self.l; } // MOV D,L
            0x56 => {                    // MOV D,M
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.d = self.mem[offset];
            }
            0x57 => { self.d = self.a; } // MOV D,A

            0x58 => { self.e = self.b; } // MOV E,B
            0x59 => { self.e = self.c; } // MOV E,C
            0x5A => { self.e = self.d; } // MOV E,D
            0x5B => { self.e = self.e; } // MOV E,E
            0x5C => { self.e = self.h; } // MOV E,H
            0x5D => { self.e = self.l; } // MOV E,L
            0x5E => {                    // MOV E,M
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.e = self.mem[offset];
            }
            0x5F => { self.e = self.a; } // MOV E,A

            0x60 => { self.h = self.b; } // MOV H,B
            0x61 => { self.h = self.c; } // MOV H,C
            0x62 => { self.h = self.d; } // MOV H,D
            0x63 => { self.h = self.e; } // MOV H,E
            0x64 => { self.h = self.h; } // MOV H,H
            0x65 => { self.h = self.l; } // MOV H,L
            0x66 => {                    // MOV H,M
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.h = self.mem[offset];
            }
            0x67 => { self.h = self.a; } // MOV H,A

            0x68 => { self.l = self.b; } // MOV L,B
            0x69 => { self.l = self.c; } // MOV L,C
            0x6A => { self.l = self.d; } // MOV L,D
            0x6B => { self.l = self.e; } // MOV L,E
            0x6C => { self.l = self.h; } // MOV L,H
            0x6D => { self.l = self.l; } // MOV L,L
            0x6E => {                    // MOV L,M
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.l = self.mem[offset];
            }
            0x6F => { self.l = self.a; } // MOV L,A
            
            0x70 => {                    // MOV M,B
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.b;
            }
            0x71 => {                    // MOV M,C
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.c;
            }
            0x72 => {                    // MOV M,D
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.d;
            }
            0x73 => {                    // MOV M,E
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.e;
            }
            0x74 => {                    // MOV M,H
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.h;
            }
            0x75 => {                    // MOV M,L
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.l;
            }
            0x77 => {                    // MOV M,A
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.a;
            }

            0x78 => { self.a = self.b; } // MOV A,B
            0x79 => { self.a = self.c; } // MOV A,C
            0x7A => { self.a = self.d; } // MOV A,D
            0x7B => { self.a = self.e; } // MOV A,E
            0x7C => { self.a = self.h; } // MOV A,H
            0x7D => { self.a = self.l; } // MOV A,L
            0x7E => {                    // MOV A,M
                let offset: usize = Self::extend(self.h, self.l) as usize;
                self.a = self.mem[offset];
            }
            0x7F => { self.a = self.a; } // MOV A,A

            op => {
                let mut friendly_name = String::new();
                disasm::disasm_single(&mut friendly_name, self.mem, self.pc).expect("Faild to write to string");
                friendly_name.pop();
                panic!("Opcode {:x} (aka. `{}`) not implemented!", op, friendly_name)
            }
        } 

        self.pc += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn zero_flag_true() {
        let mut emu = State::new(&mut []);

        emu.zero_flag(0x00);

        assert_eq!(emu.cc.z, true);
    }

    #[test]
    fn zero_flag_false() {
        let mut emu = State::new(&mut []);

        emu.zero_flag(0xFF);

        assert_eq!(emu.cc.z, false);
    }

    #[test]
    fn sign_flag_false() {
        let mut emu = State::new(&mut []);

        emu.sign_flag(0b01101111);

        assert_eq!(emu.cc.s, false);
    }

    #[test]
    fn sign_flag_true() {
        let mut emu = State::new(&mut []);

        emu.sign_flag(0b11101011);

        assert_eq!(emu.cc.s, true);
    }

    #[test]
    fn carry_flag_false() {
        let mut emu = State::new(&mut []);

        emu.carry_flag(0x0001);

        assert_eq!(emu.cc.cy, false);
    }


    #[test]
    fn carry_flag_true() {
        let mut emu = State::new(&mut []);

        emu.carry_flag(0xFF01);

        assert_eq!(emu.cc.cy, true);
    }

    #[test]
    fn parity_true1() {
        assert_eq!(State::parity(0b1111111111111111), true);
    }

    #[test]
    fn parity_true2() {
        assert_eq!(State::parity(0b1111111111110000), true);
    }
    
    #[test]
    fn parity_true3() {
        assert_eq!(State::parity(0b1010101010100000), true);
    }

    #[test]
    fn parity_false1() {
        assert_eq!(State::parity(0b1111101111111111), false);
    }

    #[test]
    fn parity_false2() {
        assert_eq!(State::parity(0b1111101111110000), false);
    }
    
    #[test]
    fn parity_false3() {
        assert_eq!(State::parity(0b1010100010100000), false);
    }
    
    #[test]
    fn extend1() {
        assert_eq!(State::extend(0x01, 0xF0), 0x01F0);
    }

    #[test]
    fn extend2() {
        assert_eq!(State::extend(0xFF, 0xFF), 0xFFFF);
    }
    
    #[test]
    fn seperate1() {
        assert_eq!((0x01, 0xF0), State::seperate(0x01F0));
    }

    #[test]
    fn seperate2() {
        assert_eq!((0xFF, 0xFF), State::seperate(0xFFFF));
    }

}

