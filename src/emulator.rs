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

    sp: usize,
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

            sp: 0xf000,
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

    pub fn steps(&mut self) {
        loop {
            let mut command = String::new();
            std::io::stdin().read_line(&mut command).expect("Did not enter a correct string");
            let mut rep = String::new();
            disasm::disasm_single(&mut rep, self.mem, self.pc).expect("Failed to write");
            rep.pop();
            println!("af: {:02x}{:02x}, bc: {:02x}{:02x}, de: {:02x}{:02x}, hl:{:02x}{:02x}, pc: {:04x}, sp: {:04x}\n{} opcode: {:02x}",self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.pc, self.sp, rep, self.mem[self.pc]);
            self.step();
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

    fn assign_ref<T>(assigns: (&mut T, &mut T), value: (T, T)) {
        *assigns.0 = value.1;
        *assigns.1 = value.0;
    }

    fn add(&mut self, value: u8) {
        let result = self.a as u16 + value as u16;
        self.carry_flag(result);
        self.arith_flags(result);
        self.a = result as u8;
    }

    fn add_cy(&mut self, value: u8) {
        let result = self.a as u16 + value as u16 + self.cc.cy as u16;
        self.carry_flag(result);
        self.arith_flags(result);
        self.a = result as u8;
    }

    fn sub(&mut self, value: u8) {
        let result = self.a as u16 - value as u16;
        self.carry_flag(result);
        self.arith_flags(result);
        self.a = result as u8;
    }

    fn sub_cy(&mut self, value: u8) {
        let result = self.a as u16 - value as u16 - self.cc.cy as u16;
        self.carry_flag(result);
        self.arith_flags(result);
        self.a = result as u8;
    }

    fn and(&mut self, value: u8) {
        let result = self.a & value;
        self.cc.cy = false;
        self.arith_flags(result as u16);

        self.a = result as u8;
    }

    fn xor(&mut self, value: u8) {
        let result = self.a ^ value;
        self.cc.cy = false;
        self.arith_flags(result as u16);

        self.a = result as u8;
    }
    
    fn or(&mut self, value: u8) {
        let result = self.a | value;
        self.cc.cy = false;
        self.arith_flags(result as u16);

        self.a = result as u8;
    }
    
    fn cmp(&mut self, value: u8) {
        let result = self.a as u16 - value as u16;
        self.cc.z = self.a == value;
        self.cc.cy = self.a < value;
        self.sign_flag(result);
        self.parity_flag(result);
    }

    fn ret(&mut self) {
        self.pc = Self::extend(self.mem[self.sp], self.mem[self.sp + 1]) as usize;
        self.sp += 2;
    }

    fn jmp(&mut self) {
        self.pc = Self::extend(self.mem[self.pc + 2], self.mem[self.pc + 1]) as usize;
        // Counteract the dding in each opcode
        self.pc -= 1;
    }

    fn call_jmp(&mut self) {
        let split = Self::seperate(self.pc as u16);

        self.mem[self.sp - 1] = split.0;
        self.mem[self.sp - 2] = split.1;

        self.sp -= 2;
        self.pc -= 1;
    }

    fn call(&mut self) {
        let bytes = Self::extend(self.mem[self.pc + 2], self.mem[self.pc + 1]);
        self.pc = bytes as usize;
        self.call_jmp();
    }

    fn step(&mut self) -> bool {
        match self.mem[self.pc] {
            0x00 => {} // NOP
            0x01 => {
                // LXI B, word
                self.c = self.mem[self.pc + 2];
                self.b = self.mem[self.pc + 1];
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
                self.a = (self.a >> 7) | self.a << 1;
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
                // DCX B
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
                let previous = self.a;
                self.a = (self.a << 7) | self.a >> 1;
                self.cc.cy = (previous & 0x01) == 0x01;
            }

            0x10 => {} // NOP
            0x11 => {
                // LXI D, D16
                self.d = self.mem[self.pc + 2];
                self.e = self.mem[self.pc + 1];
                self.pc += 2;
            }
            
            0x12 => {
                // STAX D
                let offset = Self::extend(self.d, self.e) as usize;
                self.mem[offset] = self.a;
            }

            0x13 => {
                // INX D
                let answer = Self::extend(self.d, self.e) + 1;
                
                Self::assign_ref((&mut self.d, &mut self.e), Self::seperate(answer));
            }

            0x14 => {
                // INR D
                let answer = self.d as u16 + 1; 
                self.arith_flags(answer);
                self.d = (answer & 0xff) as u8;
            }

            0x15 => {
                // DCR D
                let answer = self.d as u16 - 1; 
                self.arith_flags(answer);
                self.d = (answer & 0xff) as u8;
            }

            0x16 => {
                // MVI D, byte
                self.d = self.mem[self.pc + 1];
                self.pc += 1;
            }

            0x17 => {
                // RAL
                let previous = self.a;
                self.a = (self.a << 1) | (self.cc.cy as u8);
                self.cc.cy = (previous & 0b10000000) == 0b10000000;
            }

            0x18 => {} // NOP

            0x19 => {
                // DAD D
                let hl = Self::extend(self.h, self.l);
                let de = Self::extend(self.d, self.e);

                let answer: u16 = hl + de;

                Self::assign_ref((&mut self.h, &mut self.l), Self::seperate(answer));
                
                self.carry_flag(answer);
            }

            0x1A => {
                // LDAX D
                let offset = Self::extend(self.d, self.e) as usize;
                self.a = self.mem[offset];
            }

            0x1B => {
                // DCX D
                let answer = Self::extend(self.d, self.e) - 1;
                
                Self::assign_ref((&mut self.d, &mut self.e), Self::seperate(answer));
            }

            0x1C => {
                // INR E
                let answer = self.e as u16 + 1; 
                self.arith_flags(answer);
                self.e = (answer & 0xff) as u8;
            }

            0x1D => {
                // DCR E
                let answer = self.e as u16 - 1; 
                self.arith_flags(answer);
                self.e = (answer & 0xff) as u8;
            }

            0x1E => {
                // MVI E, byte
                self.e = self.mem[self.pc + 1];
                self.pc += 1;
            }

            0x1F => {
                // RAR
                let previous = self.a;
                self.a = (self.a >> 1) | ((self.cc.cy as u8) << 7);
                self.cc.cy = (previous & 0b10000000) != 0b10000000;
            }
            
            0x20 => {} // NOP

            0x21 => {
                // LXI H, D16
                self.h = self.mem[self.pc + 2];
                self.l = self.mem[self.pc + 1];
                self.pc += 2;
            }
            
            0x22 => {
                // SHLD
                let offset = Self::extend(self.mem[self.pc + 1], self.mem[self.pc + 2]) as usize;
                self.mem[offset] = self.l;
                self.mem[offset + 1] = self.h;

                self.pc += 2;
            }

            0x23 => {
                // INX H
                let answer = Self::extend(self.h, self.l) + 1;
                
                Self::assign_ref((&mut self.h, &mut self.l), Self::seperate(answer));
            }

            0x24 => {
                // INR H
                let answer = self.h as u16 + 1; 
                self.arith_flags(answer);
                self.h = (answer & 0xff) as u8;
            }

            0x25 => {
                // DCR H
                let answer = self.h as u16 - 1; 
                self.arith_flags(answer);
                self.h = (answer & 0xff) as u8;
            }

            0x26 => {
                // MVI H, byte
                self.h = self.mem[self.pc + 1];
                self.pc += 1;
            }

            0x28 => {} // NOP

            0x29 => {
                // DAD H
                let hl = Self::extend(self.h, self.l);

                let answer: u16 = hl + hl;

                Self::assign_ref((&mut self.h, &mut self.l), Self::seperate(answer));
                
                self.carry_flag(answer);
            }

            0x2A => {
                // LDHD bytes
                let offset = Self::extend(self.mem[self.pc + 1], self.mem[self.pc + 2]) as usize;
                self.h = self.mem[offset];
                self.l = self.mem[offset + 1];

                self.pc += 2;
            }

            0x2B => {
                // DCX H
                let answer = Self::extend(self.h, self.l) - 1;
                
                Self::assign_ref((&mut self.h, &mut self.l), Self::seperate(answer));
            }

            0x2C => {
                // INR L
                let answer = self.l as u16 + 1; 
                self.arith_flags(answer);
                self.l = (answer & 0xff) as u8;
            }

            0x2D => {
                // DCR L
                let answer = self.l as u16 - 1; 
                self.arith_flags(answer);
                self.l = (answer & 0xff) as u8;
            }

            0x2E => {
                // MVI L, byte
                self.l = self.mem[self.pc + 1];
                self.pc += 1;
            }

            0x2F => {
                // CMA
                self.a = !self.a;
            }

            0x30 => {} // NOP

            0x31 => {
                // LXI SP, D16
                self.sp = Self::extend(self.mem[self.pc + 2], self.mem[self.pc + 1]) as usize;
                self.pc += 2;
            }
            
            0x32 => {
                // STA addr
                let offset = Self::extend(self.mem[self.pc + 2], self.mem[self.pc + 1]) as usize;
                self.mem[offset] = self.a;

                self.pc += 2;
            }

            0x33 => {
                // INX SP
                self.sp += 1;
            }

            0x34 => {
                // INR M
                let offset = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.mem[offset] + 1;
                self.arith_flags(self.mem[offset].into());
            }

            0x35 => {
                // DCR M
                let offset = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.mem[offset] - 1;
                self.arith_flags(self.mem[offset].into());
            }

            0x36 => {
                // MVI H, byte
                let offset = Self::extend(self.h, self.l) as usize;
                self.mem[offset] = self.mem[self.pc + 1];
                self.pc += 1;
            }

            0x37 => {
                // STC
                self.cc.cy = true;
            }

            0x38 => {} // NOP

            0x39 => {
                // DAD SP
                let hl = Self::extend(self.h, self.l);

                let answer: u16 = hl + self.sp as u16;

                Self::assign_ref((&mut self.h, &mut self.l), Self::seperate(answer));
                
                self.carry_flag(answer);
            }

            0x3A => {
                // LDA, bytes
                let offset = Self::extend(self.mem[self.pc + 1], self.mem[self.pc + 2]) as usize;
                self.a = self.mem[offset];
            }

            0x3B => {
                // DCX SP
                self.sp -= 1;
            }

            0x3C => {
                // INR A
                let result = self.a + 1;
                self.arith_flags(result.into());
                self.a = result;
            }
            
            0x3D => {
                // DCR A
                let result = self.a - 1;
                self.arith_flags(result.into());
                self.a = result;

            }

            0x3E => {
                // MVI A, byte
                self.a = self.mem[self.pc + 1];
                self.pc += 1;
            }

            0x3F => {
                // CMC
                self.cc.cy = !self.cc.cy;
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

            0x76 => {
                // HLT
                return false;
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

            0x80 => { self.add(self.b); } // ADD B
            0x81 => { self.add(self.c); } // ADD C
            0x82 => { self.add(self.d); } // ADD D
            0x83 => { self.add(self.e); } // ADD E
            0x84 => { self.add(self.h); } // ADD H
            0x85 => { self.add(self.l); } // ADD L
            0x86 => {
                // ADD M
                let offset = Self::extend(self.h, self.l) as usize;
                self.add(self.mem[offset]);
            }
            0x87 => { self.add(self.a); } // ADD A

            0x88 => { self.add_cy(self.b); } // ADC B
            0x89 => { self.add_cy(self.c); } // ADC C
            0x8A => { self.add_cy(self.d); } // ADC D
            0x8B => { self.add_cy(self.e); } // ADC E
            0x8C => { self.add_cy(self.h); } // ADC H
            0x8D => { self.add_cy(self.l); } // ADC L
            0x8E => {
                // ADC M
                let offset = Self::extend(self.h, self.l) as usize;
                self.add_cy(self.mem[offset]);
            }
            0x8F => { self.add_cy(self.a); } // ADC A

            0x90 => { self.sub(self.b); } // SUB B
            0x91 => { self.sub(self.c); } // SUB C
            0x92 => { self.sub(self.d); } // SUB D
            0x93 => { self.sub(self.e); } // SUB E
            0x94 => { self.sub(self.h); } // SUB H
            0x95 => { self.sub(self.l); } // SUB L
            0x96 => {
                // SUB M
                let offset = Self::extend(self.h, self.l) as usize;
                self.sub(self.mem[offset]);
            }
            0x97 => { self.sub(self.a); } // SUB A

            0x98 => { self.sub_cy(self.b); } // SBB B
            0x99 => { self.sub_cy(self.a); } // SBB C
            0x9A => { self.sub_cy(self.d); } // SBB D
            0x9B => { self.sub_cy(self.e); } // SBB E
            0x9C => { self.sub_cy(self.h); } // SBB H
            0x9D => { self.sub_cy(self.l); } // SBB L
            0x9E => {
                // SBB M
                let offset = Self::extend(self.h, self.l) as usize;
                self.sub_cy(self.mem[offset]);
            }
            0x9F => { self.sub_cy(self.a); } // SBB A

            0xA0 => { self.and(self.b); } // ANA B
            0xA1 => { self.and(self.c); } // ANA C
            0xA2 => { self.and(self.d); } // ANA D
            0xA3 => { self.and(self.e); } // ANA E
            0xA4 => { self.and(self.h); } // ANA H
            0xA5 => { self.and(self.l); } // ANA L
            0xA6 => {
                // ANA M
                let offset = Self::extend(self.h, self.l) as usize;
                self.and(self.mem[offset]);
            }
            0xA7 => { self.and(self.a); } // ANA A

            0xA8 => { self.xor(self.b); } // XRA B
            0xA9 => { self.xor(self.a); } // XRA C
            0xAA => { self.xor(self.d); } // XRA D
            0xAB => { self.xor(self.e); } // XRA E
            0xAC => { self.xor(self.h); } // XRA H
            0xAD => { self.xor(self.l); } // XRA L
            0xAE => {
                // XRA M
                let offset = Self::extend(self.h, self.l) as usize;
                self.xor(self.mem[offset]);
            }
            0xAF => { self.xor(self.a); } // XRA A

            0xB0 => { self.or(self.b); } // ORA B
            0xB1 => { self.or(self.c); } // ORA C
            0xB2 => { self.or(self.d); } // ORA D
            0xB3 => { self.or(self.e); } // ORA E
            0xB4 => { self.or(self.h); } // ORA H
            0xB5 => { self.or(self.l); } // ORA L
            0xB6 => {
                // ORA M
                let offset = Self::extend(self.h, self.l) as usize;
                self.or(self.mem[offset]);
            }
            0xB7 => { self.or(self.a); } // ORA A

            0xB8 => { self.cmp(self.b); } // CMP B
            0xB9 => { self.cmp(self.a); } // CMP C
            0xBA => { self.cmp(self.d); } // CMP D
            0xBB => { self.cmp(self.e); } // CMP E
            0xBC => { self.cmp(self.h); } // CMP H
            0xBD => { self.cmp(self.l); } // CMP L
            0xBE => {
                // CMP M
                let offset = Self::extend(self.h, self.l) as usize;
                self.cmp(self.mem[offset]);
            }
            0xBF => { self.cmp(self.a); } // CMP A

            0xC0 => {
                // RNZ
                if !self.cc.z {
                    self.ret();
                }
            }

            0xC1 => {
                // POP B
                Self::assign_ref((&mut self.c, &mut self.b), (self.mem[self.sp], self.mem[self.sp + 1]));
                self.sp += 2;
            }

            0xC2 => {
                // JNZ bytes
                if !self.cc.z {
                    self.jmp();
                } else {
                    self.pc += 2;
                }
            }

            0xC3 => {
                // JMP bytes
                self.jmp();
            }

            0xC4 => {
                // CNZ bytes
                if !self.cc.z {
                    self.call();
                } else {
                    self.pc += 2;
                }
            }

            0xC5 => {
                // PUSH B
                self.mem[self.sp - 1] = self.b;
                self.mem[self.sp - 2] = self.c;
                self.sp -= 2;
            }

            0xC6 => {
                // ADI byte
                self.add(self.mem[self.pc + 1]);
                self.pc += 1;
            }

            0xC7 => {
                // RST 0
                self.call_jmp();
                self.pc = 0;
            }

            0xC8 => {
                // RZ
                if self.cc.z {
                    self.call();
                } else {
                    self.pc += 2;
                }
            }

            0xC9 => {
                // RET
                self.ret();
            }

            0xCA => {
                // JZ bytes
                if self.cc.z {
                    self.jmp();
                } else {
                    self.pc += 2;
                }
            }

            0xCB => {
                // JMP bytes
                self.jmp();
            }

            0xCC => {
                // CZ bytes
                if self.cc.z {
                    self.call();
                } else {
                    self.pc += 2;
                }
            }

            0xCD => {
                // CALL bytes
                self.call();
            }

            0xCE => {
                // ACI byte
                self.add_cy(self.mem[self.pc + 1]);
                self.pc += 1;
            }

            0xCF => {
                // RST 1
                self.call_jmp();
                self.pc = 8;
            }

            0xD0 => {
                // RNC
                if !self.cc.cy {
                    self.ret();
                }
            }

            0xD1 => {
                // POP D
                Self::assign_ref((&mut self.e, &mut self.d), (self.mem[self.sp], self.mem[self.sp + 1]));
                self.sp += 2;
            }

            0xD2 => {
                // JNC bytes
                if !self.cc.cy {
                    self.jmp();
                } else {
                    self.pc += 2;
                }
            }

            0xD4 => {
                // CNC bytes
                if !self.cc.cy {
                    self.call();
                } else {
                    self.pc += 2;
                }
            }

            0xD5 => {
                // PUSH D
                self.mem[self.sp - 1] = self.d;
                self.mem[self.sp - 2] = self.e;
                self.sp -= 2;
            }

            0xD6 => {
                // SUI byte
                self.sub_cy(self.mem[self.pc + 1]);
                self.pc += 1;
            }

            0xD7 => {
                // RST 2
                self.call_jmp();
                self.pc = 16;
            }

            0xD8 => {
                // RC
                if self.cc.cy {
                    self.ret();
                }
            }

            0xD9 => {
                // RET
                self.ret();
            }

            op => {
                let mut friendly_name = String::new();
                disasm::disasm_single(&mut friendly_name, self.mem, self.pc).expect("Faild to write to string");
                friendly_name.pop();
                panic!("Opcode {:x} (aka. `{}`) not implemented!", op, friendly_name);
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

    #[test]
    fn rotate_left1() {
        let mut mem = [0x07, 0x76];
        let mut emu = State::new(&mut mem);
        emu.a = 0b10000000;

        emu.start();
        assert_eq!(0b00000001, emu.a);
        assert_eq!(true, emu.cc.cy);
    }

    #[test]
    fn rotate_left2() {
        let mut mem = [0x07, 0x76];
        let mut emu = State::new(&mut mem);
        emu.a = 0b01000000;

        emu.start();
        assert_eq!(0b10000000, emu.a);
        assert_eq!(false, emu.cc.cy);
    }

    #[test]
    fn rotate_right1() {
        let mut mem = [0x0F, 0x76];
        let mut emu = State::new(&mut mem);
        emu.a = 0b10000000;

        emu.start();
        assert_eq!(0b01000000, emu.a);
        assert_eq!(false, emu.cc.cy);
    }

    #[test]
    fn rotate_right2() {
        let mut mem = [0x0F, 0x76];
        let mut emu = State::new(&mut mem);
        emu.a = 0b00000001;

        emu.start();
        assert_eq!(0b10000000, emu.a);
        assert_eq!(true, emu.cc.cy);
    }

    #[test]
    fn rotate_carry_left1() {
        let mut mem = [0x17, 0x76];
        let mut emu = State::new(&mut mem);
        emu.a = 0b10000000;
        emu.cc.cy = false;

        emu.start();
        assert_eq!(0b00000000, emu.a);
        assert_eq!(true, emu.cc.cy);
    }

    #[test]
    fn rotate_carry_left2() {
        let mut mem = [0x17, 0x76];
        let mut emu = State::new(&mut mem);
        emu.a = 0b00000001;
        emu.cc.cy = true;

        emu.start();
        assert_eq!(0b00000011, emu.a);
        assert_eq!(false, emu.cc.cy);
    }

    #[test]
    fn rotate_carry_right1() {
        let mut mem = [0x1F, 0x76];
        let mut emu = State::new(&mut mem);
        emu.a = 0b10000000;
        emu.cc.cy = false;

        emu.start();
        assert_eq!(0b01000000, emu.a);
        assert_eq!(false, emu.cc.cy);
    }

    #[test]
    fn rotate_carry_right2() {
        let mut mem = [0x1F, 0x76];
        let mut emu = State::new(&mut mem);
        emu.a = 0b00000001;
        emu.cc.cy = true;

        emu.start();
        assert_eq!(0b10000000, emu.a);
        assert_eq!(true, emu.cc.cy);
    }

}

