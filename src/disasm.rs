use std::error;
use std::fmt::Write;

pub fn disasm(file_contents: &[u8]) -> Result<String, Box<dyn error::Error>> {
    let mut asm = String::new();
    let mut pos: usize = 0;
    
    while pos < file_contents.len() {
        pos += disasm_single(&mut asm, file_contents, pos)?;
    }

    Ok(asm)
}

pub fn disasm_single(asm: &mut String, code: &[u8], pos: usize) -> Result<usize, Box<dyn error::Error>> {
    write!(asm, "{:0>4X}   ", pos)?;

    let mut op_len = 1;
    match code[pos] {
        0x00 => { writeln!(asm, "NOP")?; }
        0x01 => { writeln!(asm, "LXI     B,#${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0x02 => { writeln!(asm, "STAX    B")?; }
        0x03 => { writeln!(asm, "INX     B")?; }
        0x04 => { writeln!(asm, "INR     B")?; }
        0x05 => { writeln!(asm, "DCR     B")?; }
        0x06 => { writeln!(asm, "MVI     B,#${:02x}", code[pos + 1])?; op_len = 2; }
        0x07 => { writeln!(asm, "RLC")?; }

        0x08 => { writeln!(asm, "NOP")?; }
        0x09 => { writeln!(asm, "DAD     B")?; }
        0x0A => { writeln!(asm, "LDAX    B")?; }
        0x0B => { writeln!(asm, "DCX     B")?; }
        0x0C => { writeln!(asm, "INR     C")?; }
        0x0D => { writeln!(asm, "DCR     C")?; } 
        0x0E => { writeln!(asm, "MVI     C,#${:02x}", code[pos + 1])?; op_len = 2; }
        0x0F => { writeln!(asm, "RRC")?; }


        0x10 => { writeln!(asm, "NOP")?; }
        0x11 => { writeln!(asm, "LXI     D,#${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0x12 => { writeln!(asm, "STAX    D")?; }
        0x13 => { writeln!(asm, "INX     D")?; }
        0x14 => { writeln!(asm, "INR     D")?; }
        0x15 => { writeln!(asm, "DCR     D")?; }
        0x16 => { writeln!(asm, "MVI     D,#${:02x}", code[pos + 1])?; op_len = 2; }
        0x17 => { writeln!(asm, "RAL")?; }

        0x18 => { writeln!(asm, "NOP")?; }
        0x19 => { writeln!(asm, "DAD     D")?; }
        0x1A => { writeln!(asm, "LDAX    D")?; }
        0x1B => { writeln!(asm, "DCX     D")?; }
        0x1C => { writeln!(asm, "INR     E")?; }
        0x1D => { writeln!(asm, "DCR     E")?; } 
        0x1E => { writeln!(asm, "MVI     E,#${:02x}", code[pos + 1])?; op_len = 2; }
        0x1F => { writeln!(asm, "RAR")?; }

        0x20 => { writeln!(asm, "NOP")?; }
        0x21 => { writeln!(asm, "LXI     H,#${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0x22 => { writeln!(asm, "SHLD    ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0x23 => { writeln!(asm, "INX     H")?; }
        0x24 => { writeln!(asm, "INR     H")?; }
        0x25 => { writeln!(asm, "DCR     H")?; }
        0x26 => { writeln!(asm, "MVI     H,#${:02x}", code[pos + 1])?; op_len = 2; }
        0x27 => { writeln!(asm, "DAA")?; }

        0x28 => { writeln!(asm, "NOP")?; }
        0x29 => { writeln!(asm, "DAD     H")?; }
        0x2A => { writeln!(asm, "LDHD    ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0x2B => { writeln!(asm, "DCX     H")?; }
        0x2C => { writeln!(asm, "INR     L")?; }
        0x2D => { writeln!(asm, "DCR     L")?; } 
        0x2E => { writeln!(asm, "MVI     L,#${:02x}", code[pos + 1])?; op_len = 2; }
        0x2F => { writeln!(asm, "CMA")?; }

        0x30 => { writeln!(asm, "NOP")?; }
        0x31 => { writeln!(asm, "LXI     SP,#${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0x32 => { writeln!(asm, "STA     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0x33 => { writeln!(asm, "INX     SP")?; }
        0x34 => { writeln!(asm, "INR     M")?; }
        0x35 => { writeln!(asm, "DCR     M")?; }
        0x36 => { writeln!(asm, "MVI     M,#${:02x}", code[pos + 1])?; op_len = 2; }
        0x37 => { writeln!(asm, "STC")?; }

        0x38 => { writeln!(asm, "NOP")?; }
        0x39 => { writeln!(asm, "DAD     SP")?; }
        0x3A => { writeln!(asm, "LDA     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0x3B => { writeln!(asm, "DCX     SP")?; }
        0x3C => { writeln!(asm, "INR     A")?; }
        0x3D => { writeln!(asm, "DCR     A")?; } 
        0x3E => { writeln!(asm, "MVI     A,#${:02x}", code[pos + 1])?; op_len = 2; }
        0x3F => { writeln!(asm, "CMC")?; }

        0x40 => { writeln!(asm, "MOV     B,B")?; }
        0x41 => { writeln!(asm, "MOV     B,C")?; }
        0x42 => { writeln!(asm, "MOV     B,D")?; }
        0x43 => { writeln!(asm, "MOV     B,E")?; }
        0x44 => { writeln!(asm, "MOV     B,H")?; }
        0x45 => { writeln!(asm, "MOV     B,L")?; }
        0x46 => { writeln!(asm, "MOV     B,M")?; }
        0x47 => { writeln!(asm, "MOV     B,A")?; }

        0x48 => { writeln!(asm, "MOV     C,B")?; }
        0x49 => { writeln!(asm, "MOV     C,C")?; }
        0x4A => { writeln!(asm, "MOV     C,D")?; }
        0x4B => { writeln!(asm, "MOV     C,E")?; }
        0x4C => { writeln!(asm, "MOV     C,H")?; }
        0x4D => { writeln!(asm, "MOV     C,L")?; }
        0x4E => { writeln!(asm, "MOV     C,M")?; }
        0x4F => { writeln!(asm, "MOV     C,A")?; }

        0x50 => { writeln!(asm, "MOV     D,B")?; }
        0x51 => { writeln!(asm, "MOV     D,C")?; }
        0x52 => { writeln!(asm, "MOV     D,D")?; }
        0x53 => { writeln!(asm, "MOV     D,E")?; }
        0x54 => { writeln!(asm, "MOV     D,H")?; }
        0x55 => { writeln!(asm, "MOV     D,L")?; }
        0x56 => { writeln!(asm, "MOV     D,M")?; }
        0x57 => { writeln!(asm, "MOV     D,A")?; }

        0x58 => { writeln!(asm, "MOV     E,B")?; }
        0x59 => { writeln!(asm, "MOV     E,C")?; }
        0x5A => { writeln!(asm, "MOV     E,D")?; }
        0x5B => { writeln!(asm, "MOV     E,E")?; }
        0x5C => { writeln!(asm, "MOV     E,H")?; }
        0x5D => { writeln!(asm, "MOV     E,L")?; }
        0x5E => { writeln!(asm, "MOV     E,M")?; }
        0x5F => { writeln!(asm, "MOV     E,A")?; }
        
        0x60 => { writeln!(asm, "MOV     H,B")?; }
        0x61 => { writeln!(asm, "MOV     H,C")?; }
        0x62 => { writeln!(asm, "MOV     H,D")?; }
        0x63 => { writeln!(asm, "MOV     H,E")?; }
        0x64 => { writeln!(asm, "MOV     H,H")?; }
        0x65 => { writeln!(asm, "MOV     H,L")?; }
        0x66 => { writeln!(asm, "MOV     H,M")?; }
        0x67 => { writeln!(asm, "MOV     H,A")?; }

        0x68 => { writeln!(asm, "MOV     L,B")?; }
        0x69 => { writeln!(asm, "MOV     L,C")?; }
        0x6A => { writeln!(asm, "MOV     L,D")?; }
        0x6B => { writeln!(asm, "MOV     L,E")?; }
        0x6C => { writeln!(asm, "MOV     L,H")?; }
        0x6D => { writeln!(asm, "MOV     L,L")?; }
        0x6E => { writeln!(asm, "MOV     L,M")?; }
        0x6F => { writeln!(asm, "MOV     L,A")?; }

        0x70 => { writeln!(asm, "MOV     M,B")?; }
        0x71 => { writeln!(asm, "MOV     M,C")?; }
        0x72 => { writeln!(asm, "MOV     M,D")?; }
        0x73 => { writeln!(asm, "MOV     M,E")?; }
        0x74 => { writeln!(asm, "MOV     M,H")?; }
        0x75 => { writeln!(asm, "MOV     M,L")?; }

        0x76 => { writeln!(asm, "HLT")?; }

        0x77 => { writeln!(asm, "MOV     M,A")?; }

        0x78 => { writeln!(asm, "MOV     A,B")?; }
        0x79 => { writeln!(asm, "MOV     A,C")?; }
        0x7A => { writeln!(asm, "MOV     A,D")?; }
        0x7B => { writeln!(asm, "MOV     A,E")?; }
        0x7C => { writeln!(asm, "MOV     A,H")?; }
        0x7D => { writeln!(asm, "MOV     A,L")?; }
        0x7E => { writeln!(asm, "MOV     A,M")?; }
        0x7F => { writeln!(asm, "MOV     A,A")?; }
        
        0x80 => { writeln!(asm, "ADD     B")?; }
        0x81 => { writeln!(asm, "ADD     C")?; }
        0x82 => { writeln!(asm, "ADD     D")?; }
        0x83 => { writeln!(asm, "ADD     E")?; }
        0x84 => { writeln!(asm, "ADD     H")?; }
        0x85 => { writeln!(asm, "ADD     L")?; }
        0x86 => { writeln!(asm, "ADD     M")?; }
        0x87 => { writeln!(asm, "ADD     A")?; }

        0x88 => { writeln!(asm, "ADC     B")?; }
        0x89 => { writeln!(asm, "ADC     C")?; }
        0x8A => { writeln!(asm, "ADC     D")?; }
        0x8B => { writeln!(asm, "ADC     E")?; }
        0x8C => { writeln!(asm, "ADC     H")?; }
        0x8D => { writeln!(asm, "ADC     L")?; }
        0x8E => { writeln!(asm, "ADC     M")?; }
        0x8F => { writeln!(asm, "ADC     A")?; }
        
        0x90 => { writeln!(asm, "SUB     B")?; }
        0x91 => { writeln!(asm, "SUB     C")?; }
        0x92 => { writeln!(asm, "SUB     D")?; }
        0x93 => { writeln!(asm, "SUB     E")?; }
        0x94 => { writeln!(asm, "SUB     H")?; }
        0x95 => { writeln!(asm, "SUB     L")?; }
        0x96 => { writeln!(asm, "SUB     M")?; }
        0x97 => { writeln!(asm, "SUB     A")?; }

        0x98 => { writeln!(asm, "SBB     B")?; }
        0x99 => { writeln!(asm, "SBB     C")?; }
        0x9A => { writeln!(asm, "SBB     D")?; }
        0x9B => { writeln!(asm, "SBB     E")?; }
        0x9C => { writeln!(asm, "SBB     H")?; }
        0x9D => { writeln!(asm, "SBB     L")?; }
        0x9E => { writeln!(asm, "SBB     M")?; }
        0x9F => { writeln!(asm, "SBB     A")?; }
            
        0xA0 => { writeln!(asm, "ANA     B")?; }
        0xA1 => { writeln!(asm, "ANA     C")?; }
        0xA2 => { writeln!(asm, "ANA     D")?; }
        0xA3 => { writeln!(asm, "ANA     E")?; }
        0xA4 => { writeln!(asm, "ANA     H")?; }
        0xA5 => { writeln!(asm, "ANA     L")?; }
        0xA6 => { writeln!(asm, "ANA     M")?; }
        0xA7 => { writeln!(asm, "ANA     A")?; }

        0xA8 => { writeln!(asm, "XRA     B")?; }
        0xA9 => { writeln!(asm, "XRA     C")?; }
        0xAA => { writeln!(asm, "XRA     D")?; }
        0xAB => { writeln!(asm, "XRA     E")?; }
        0xAC => { writeln!(asm, "XRA     H")?; }
        0xAD => { writeln!(asm, "XRA     L")?; }
        0xAE => { writeln!(asm, "XRA     M")?; }
        0xAF => { writeln!(asm, "XRA     A")?; }
        
        0xB0 => { writeln!(asm, "ORA     B")?; }
        0xB1 => { writeln!(asm, "ORA     C")?; }
        0xB2 => { writeln!(asm, "ORA     D")?; }
        0xB3 => { writeln!(asm, "ORA     E")?; }
        0xB4 => { writeln!(asm, "ORA     H")?; }
        0xB5 => { writeln!(asm, "ORA     L")?; }
        0xB6 => { writeln!(asm, "ORA     M")?; }
        0xB7 => { writeln!(asm, "ORA     A")?; }

        0xB8 => { writeln!(asm, "CMP     B")?; }
        0xB9 => { writeln!(asm, "CMP     C")?; }
        0xBA => { writeln!(asm, "CMP     D")?; }
        0xBB => { writeln!(asm, "CMP     E")?; }
        0xBC => { writeln!(asm, "CMP     H")?; }
        0xBD => { writeln!(asm, "CMP     L")?; }
        0xBE => { writeln!(asm, "CMP     M")?; }
        0xBF => { writeln!(asm, "CMP     A")?; }

        0xC0 => { writeln!(asm, "RNZ")?; }
        0xC1 => { writeln!(asm, "POP     B")?; }
        
        0xC2 => { writeln!(asm, "JNZ     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xC3 => { writeln!(asm, "JMP     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }      
        0xC4 => { writeln!(asm, "CNZ     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }        

        0xC5 => { writeln!(asm, "PUSH    B")?; }
        0xC6 => { writeln!(asm, "ADI     #${:02x}", code[pos + 1])?; op_len = 2; }
        0xC7 => { writeln!(asm, "RST     0")?; }
        0xC8 => { writeln!(asm, "RZ")?; }
        0xC9 => { writeln!(asm, "RET")?; }
        
        0xCA => { writeln!(asm, "JZ      ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xCB => { writeln!(asm, "JMP     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xCC => { writeln!(asm, "CZ      ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xCD => { writeln!(asm, "CALL    ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xCE => { writeln!(asm, "ACI     #${:02x}", code[pos + 1])?; op_len = 2; }
        0xCF => { writeln!(asm, "RST     1")?; }
        0xD0 => { writeln!(asm, "RNC")?; }

        0xD1 => { writeln!(asm, "POP     D")?; }
        0xD2 => { writeln!(asm, "JNC     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xD3 => { writeln!(asm, "OUT     #${:02x}", code[pos + 1])?; op_len = 2; }
        0xD4 => { writeln!(asm, "CNC     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xD5 => { writeln!(asm, "PUSH    D")?; }
        0xD6 => { writeln!(asm, "SUI     #${:02x}", code[pos + 1])?; op_len = 2; }
        0xD7 => { writeln!(asm, "RST     2")?; }
        0xD8 => { writeln!(asm, "RC")?; } 
        0xD9 => { writeln!(asm, "RET")?; }

        0xDA => { writeln!(asm, "JC      ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xDB => { writeln!(asm, "IN      #${:02x}", code[pos + 1])?; op_len = 2; }
        0xDC => { writeln!(asm, "CC     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; } 
        0xDD => { writeln!(asm, "CALL    ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xDE => { writeln!(asm, "SBI     #${:02x}", code[pos + 1])?; op_len = 2; }
        0xDF => { writeln!(asm, "RST     3")?; }
        0xE0 => { writeln!(asm, "RPO")?; }

        0xE1 => { writeln!(asm, "POP     H")?; }
        0xE2 => { writeln!(asm, "JPO     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        
        0xE3 => { writeln!(asm, "XTHL")?; }
        0xE4 => { writeln!(asm, "CTO     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xE5 => { writeln!(asm, "PUSH    H")?; }
        0xE6 => { writeln!(asm, "ANI     #${:02x}", code[pos + 1])?; op_len = 2; } 
        0xE7 => { writeln!(asm, "RST     4")?; }
        0xE8 => { writeln!(asm, "RPE")?; }
        0xE9 => { writeln!(asm, "PCHL")?; }
        0xEA => { writeln!(asm, "JPE     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xEB => { writeln!(asm, "XCHG")?; } 
        0xEC => { writeln!(asm, "CPE     ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xED => { writeln!(asm, "CALL    ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xEE => { writeln!(asm, "XRI     #${:02x}", code[pos + 1])?; op_len = 2; }
        0xEF => { writeln!(asm, "RST     5")?; }
        0xF0 => { writeln!(asm, "RP")?; }

        0xF1 => { writeln!(asm, "POP     PSW")?; } 
        0xF2 => { writeln!(asm, "JP      ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xF3 => { writeln!(asm, "DI")?; }
        0xF4 => { writeln!(asm, "CP      ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xF5 => { writeln!(asm, "PUSH    PSW")?; }
        0xF6 => { writeln!(asm, "ORI     #${:02x}", code[pos + 1])?; op_len = 2; }
        0xF7 => { writeln!(asm, "RST     6")?; }
        0xF8 => { writeln!(asm, "RM")?; }
        0xF9 => { writeln!(asm, "SPHL")?; }
        0xFA => { writeln!(asm, "JM      ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xFB => { writeln!(asm, "EI")?; } 
        0xFC => { writeln!(asm, "CM      ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xFD => { writeln!(asm, "CALL    ${:02x}{:02x}", code[pos + 2], code[pos + 1])?; op_len = 3; }
        0xFE => { writeln!(asm, "CPI     #${:02x}", code[pos + 1])?; op_len = 2; }
        0xFF => { writeln!(asm, "RST     7")?; }
    }

    Ok(op_len)
}

