use std::process::exit;

use crate::mmu::MMU;

const ZERO_BIT: u8        = 0x80;
const SUBTRACTION_BIT: u8 = 0x40;
const HALF_CARRY_BIT: u8  = 0x20;
const CARRY_BIT: u8       = 0x10;

//TODO
// Interrupts
//const VBLANK_INTERRUPT_BIT: u8 =  0x01;
//const LCD_INTERRUPT_BIT: u8    = (0x01 << 1);
//const TIMER_INTERRUPT_BIT: u8  = (0x01 << 2);
//const SERIAL_INTERRUPT_BIT: u8 = (0x01 << 3);
//const JOYPAD_INTERRUPT_BIT: u8 = (0x01 << 4);

#[derive(Copy, Clone)]
struct HiLoRegister {
    pub lo: u8,
    pub hi: u8
}

union Register16 {
    hilo: HiLoRegister,
    word: u16
}

pub struct Clock {
    m: u32,
    t: u32
}

pub struct CPU {
    af: Register16,
    bc: Register16,
    de: Register16,
    hl: Register16,
    pub stack_pointer: u16,
    pub program_counter: u16,
    pub skip_bios: bool,
    pub clock: Clock,
    interrupt_master_enable: bool
}

impl CPU {
    pub fn new() -> CPU {
        debug!("Initializing CPU");

        let af = Register16{word: 0};
        let bc = Register16{word: 0};
        let de = Register16{word: 0};
        let hl = Register16{word: 0};
        let stack_pointer = 0x0;
        let program_counter = 0x0;
        let skip_bios = false;
        let clock = Clock{ m: 0, t: 0 };
        let interrupt_master_enable = true;

        CPU {
            af,
            bc,
            de,
            hl,
            stack_pointer,
            program_counter,
            skip_bios,
            clock,
            interrupt_master_enable
        }
    }

    pub fn get_clock_m(&self) -> u32 {
        return self.clock.m;
    }

    pub fn get_clock_t(&self) -> u32 {
        return self.clock.t;
    }

    pub fn tick(&mut self, mmu: &mut MMU) {
        // Fetch opcode
        let opcode = mmu.read_byte(self.program_counter);

        // Call relevant function which emulates the opcode
        match opcode {
            0xCB => {
                self.process_cb_opcode(mmu);
            },
            _ => {
                self.process_opcode(mmu, opcode);
            }
        }
    }

    fn process_opcode(&mut self, mmu: &mut MMU, opcode: u8) {
        match opcode {
            0x00 => {
                trace!("0x00: NOP.");

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x04 => {
                trace!("0x04: INC B. B:{:#04X} -> {:#04X}", self.read_register_b(), self.read_register_b() + 1);

                if self.read_register_b() & 0xF == 0xF {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_b(self.read_register_b() + 1);

                if self.read_register_b() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.unset_flag_bit(SUBTRACTION_BIT);

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x05 => {
                trace!("0x05: DEC B. B:{:#04X} -> {:#04X}", self.read_register_b(), self.read_register_b().wrapping_sub(1));

                self.set_flag_bit(SUBTRACTION_BIT);

                // TODO - Pretty sure this doesn't work right
                if (((self.read_register_b() & 0xF) + (-1i8 & 0xF) as u8) & 0x10) == 0x10 {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_b(self.read_register_b().wrapping_sub(1));

                if self.read_register_b() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x06 => {
                trace!("0x06: LD B,d8. B:{:#04X} -> {:#04X}", self.read_register_b(), mmu.read_byte(self.program_counter + 1));

                self.write_register_b(mmu.read_byte(self.program_counter + 1));

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            0x0C => {
                trace!("0x0C: INC C. C:{:#04X} -> {:#04X}", self.read_register_c(), self.read_register_c() + 1);

                if self.read_register_c() & 0xF == 0xF {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_c(self.read_register_c() + 1);

                if self.read_register_c() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.unset_flag_bit(SUBTRACTION_BIT);

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x0D => {
                trace!("0x0D: DEC C. C:{:#04X} -> {:#04X}", self.read_register_c(), self.read_register_c() - 1);

                self.set_flag_bit(SUBTRACTION_BIT);

                // TODO - Pretty sure this doesn't work right
                if (((self.read_register_c() & 0xF) + (-1i8 & 0xF) as u8) & 0x10) == 0x10 {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_c(self.read_register_c() - 1);

                if self.read_register_c() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x0E => {
                trace!("0x0E: LD C,d8. C:{:#04X} -> {:#04X}", self.read_register_c(), mmu.read_byte(self.program_counter + 1));

                self.write_register_c(mmu.read_byte(self.program_counter + 1));

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            0x11 => {
                trace!("0x11: LD DE,d16. DE:{:#06X} -> {:#06X}", self.read_register_de(), mmu.read_word(self.program_counter + 1));

                self.write_register_de(mmu.read_word(self.program_counter + 1));

                self.clock.m = 3;
                self.clock.t = 12;
                self.program_counter += 3;
            },
            0x13 => {
                trace!("0x13: INC DE. DE:{:#06X} -> {:#06X}", self.read_register_de(), self.read_register_de() + 1);

                self.write_register_de(self.read_register_de() + 1);

                self.clock.m = 1;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0x15 => {
                trace!("0x15: DEC D. D:{:#04X} -> {:#04X}", self.read_register_d(), self.read_register_d() - 1);

                self.set_flag_bit(SUBTRACTION_BIT);

                // TODO - Pretty sure this doesn't work right
                if (((self.read_register_d() & 0xF) + (-1i8 & 0xF) as u8) & 0x10) == 0x10 {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_d(self.read_register_d() - 1);

                if self.read_register_d() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x16 => {
                trace!("0x16: LD D,d8. D:{:#04X} -> {:#04X}", self.read_register_d(), mmu.read_byte(self.program_counter + 1));

                self.write_register_d(mmu.read_byte(self.program_counter + 1));

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            0x17 => {
                trace!("0x17: RLA. A:{:#04X}", self.read_register_a());

                // Store carry flag
                let carry = self.read_flag(CARRY_BIT);
                let a = self.read_register_a();

                // Unset flag bits
                self.unset_flag_bit(SUBTRACTION_BIT);
                self.unset_flag_bit(HALF_CARRY_BIT);

                if self.most_significant_bit(a) > 0 {
                    self.set_flag_bit(CARRY_BIT);
                } else {
                    self.unset_flag_bit(CARRY_BIT);
                }

                self.write_register_a((a << 1) | carry);

                // Set zero bit
                if self.read_register_a() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x18 => {
                trace!("0x18: JR r8.");

                let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                if next_byte_signed < 0 {
                    self.program_counter -= ((next_byte_signed + 2) * -1) as u16;
                } else {
                    self.program_counter += (next_byte_signed + 2) as u16;
                }

                self.clock.m = 2;
                self.clock.t = 12;
            },
            0x1A => {
                trace!("0x1A: LD A,(DE). A:{:#04X} -> {:#04X}", self.read_register_a(), mmu.read_byte(self.read_register_de()));

                self.write_register_a(mmu.read_byte(self.read_register_de()));

                self.clock.m = 1;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0x1D => {
                trace!("0x1D: DEC E. E:{:#04X} -> {:#04X}", self.read_register_e(), self.read_register_e() - 1);

                self.set_flag_bit(SUBTRACTION_BIT);

                // TODO - Pretty sure this doesn't work right
                if (((self.read_register_e() & 0xF) + (-1i8 & 0xF) as u8) & 0x10) == 0x10 {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_e(self.read_register_e() - 1);

                if self.read_register_e() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x1E => {
                trace!("0x1E: LD E,d8. E:{:#04X} -> {:#04X}", self.read_register_e(), mmu.read_byte(self.program_counter + 1));

                self.write_register_e(mmu.read_byte(self.program_counter + 1));

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            0x20 => {
                trace!("0x20: JR NZ,r8. Z:{:#04X}", self.read_flag(ZERO_BIT));

                if self.read_flag(ZERO_BIT) == 0 {
                    let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                    if next_byte_signed < 0 {
                        trace!("Jumping {}", next_byte_signed + 2);
                        self.program_counter -= ((next_byte_signed + 2) * -1) as u16;
                    } else {
                        trace!("Jumping {}", (next_byte_signed + 2));
                        self.program_counter += (next_byte_signed + 2) as u16;
                    }
                    self.clock.t = 12;
                } else {
                    self.program_counter += 2;
                    self.clock.t = 8;
                }

                self.clock.m = 2;
            },
            0x21 => {
                trace!("0x21: LD HL,d16. HL:{:#06X} -> {:#06X}", self.read_register_hl(), mmu.read_word(self.program_counter + 1));
                self.write_register_hl(mmu.read_word(self.program_counter + 1));

                self.clock.m = 3;
                self.clock.t = 12;
                self.program_counter += 3;
            },
            0x22 => {
                trace!("0x22: LD (HL+),A. HL:{:#06X} <- A:{:#04X}", self.read_register_hl(), self.read_register_a());

                mmu.write_byte(self.read_register_hl(), self.read_register_a());
                self.write_register_hl(self.read_register_hl() + 1);

                self.clock.m = 1;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0x23 => {
                trace!("0x23: INC HL. HL:{:#06X} -> {:#06X}", self.read_register_hl(), self.read_register_hl() + 1);

                self.write_register_hl(self.read_register_hl() + 1);

                self.clock.m = 1;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0x24 => {
                trace!("0x24: INC H. H:{:#04X} -> {:#04X}", self.read_register_h(), self.read_register_h() + 1);

                if self.read_register_h() & 0xF == 0xF {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_h(self.read_register_h() + 1);

                if self.read_register_h() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.unset_flag_bit(SUBTRACTION_BIT);

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x28 => {
                trace!("0x28: JR Z,r8. Z:{:#04X}", self.read_flag(ZERO_BIT));

                if self.read_flag(ZERO_BIT) == 1 {
                    let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                    if next_byte_signed < 0 {
                        trace!("Jumping {}", next_byte_signed + 2);
                        self.program_counter -= ((next_byte_signed + 2) * -1) as u16;
                    } else {
                        trace!("Jumping {}", next_byte_signed + 2);
                        self.program_counter += (next_byte_signed + 2) as u16;
                    }
                    self.clock.t = 12;
                } else {
                    self.program_counter += 2;
                    self.clock.t = 8;
                }

                self.clock.m = 2;
            },
            0x2E => {
                trace!("0x2E: LD L,d8. L:{:#04X} -> {:#04X}", self.read_register_l(), mmu.read_byte(self.program_counter + 1));

                self.write_register_l(mmu.read_byte(self.program_counter + 1));

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            0x31 => {
                trace!("0x31: LD SP,d16. SP:{:#06X} -> {:#06X}", self.stack_pointer, mmu.read_word(self.program_counter + 1));
                self.stack_pointer = mmu.read_word(self.program_counter + 1);

                self.clock.m = 3;
                self.clock.t = 12;
                self.program_counter += 3;
            },
            0x32 => {
                trace!("0x32: LD (HL-),A. HL:{:#06X} <- A:{:#04X}", self.read_register_hl(), self.read_register_a());

                mmu.write_byte(self.read_register_hl(), self.read_register_a());
                self.write_register_hl(self.read_register_hl() - 1);

                self.clock.m = 1;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0x3D => {
                trace!("0x3D: DEC A. A:{:#04X} -> {:#04X}", self.read_register_a(), self.read_register_a() - 1);

                self.set_flag_bit(SUBTRACTION_BIT);

                // TODO - Pretty sure this doesn't work right
                if (((self.read_register_a() & 0xF) + (-1i8 & 0xF) as u8) & 0x10) == 0x10 {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_a(self.read_register_a() - 1);

                if self.read_register_a() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x3E => {
                trace!("0x3E: LD A,d8. A:{:#04X} -> {:#04X}", self.read_register_a(), mmu.read_byte(self.program_counter + 1));

                self.write_register_a(mmu.read_byte(self.program_counter + 1));

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            0x4F => {
                trace!("0x4F: LD C,A. C:{:#04X} -> {:#04X}", self.read_register_c(), self.read_register_a());

                self.write_register_c(self.read_register_a());

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x57 => {
                trace!("0x57: LD D,A. D:{:#04X} <- A:{:#04X}", self.read_register_d(), self.read_register_a());

                self.write_register_d(self.read_register_a());

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x67 => {
                trace!("0x67: LD H,A. H:{:#04X} <- A:{:#04X}", self.read_register_h(), self.read_register_a());

                self.write_register_h(self.read_register_a());

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x77 => {
                trace!("0x77: LD (HL),A. HL:{:#06X} <- A: {:#04X}", self.read_register_hl(), self.read_register_a());

                mmu.write_byte(self.read_register_hl(), self.read_register_a());

                self.clock.m = 1;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0x78 => {
                trace!("0x78: LD A,B. A:{:#04X} <- B:{:#04X}", self.read_register_a(), self.read_register_b());

                self.write_register_a(self.read_register_b());

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x7B => {
                trace!("0x7B: LD A,E. A:{:#04X} <- E:{:#04X}", self.read_register_a(), self.read_register_e());

                self.write_register_a(self.read_register_e());

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x7C => {
                trace!("0x7C: LD A,H. A:{:#04X} <- H:{:#04X}", self.read_register_a(), self.read_register_h());

                self.write_register_a(self.read_register_h());

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x7D => {
                trace!("0x7D: LD A,L. A:{:#04X} <- L:{:#04X}", self.read_register_a(), self.read_register_l());

                self.write_register_a(self.read_register_l());

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0x86 => {
                trace!("0x86: ADD A,(HL). A = {:#04X} + {:#04X}", self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                let a = self.read_register_a();
                let hl_byte = mmu.read_byte(self.read_register_hl());

                self.unset_flag_bit(SUBTRACTION_BIT);

                if a as u16 + hl_byte as u16 > 0xFF {
                    self.set_flag_bit(CARRY_BIT);
                } else {
                    self.unset_flag_bit(CARRY_BIT);
                }

                if ((a & 0xF) + (hl_byte) & 0xF) > 0xF {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_a(a.wrapping_add(hl_byte));

                if self.read_register_a() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.clock.m = 1;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0x90 => {
                trace!("0x90: SUB B. A:{:#04X} - B:{:#04X}", self.read_register_a(), self.read_register_b());

                self.set_flag_bit(SUBTRACTION_BIT);

                if self.read_register_b() > self.read_register_a() {
                    self.set_flag_bit(CARRY_BIT);
                } else {
                    self.unset_flag_bit(CARRY_BIT);
                }

                // TODO - Does this work right???
                if (self.read_register_a() - self.read_register_b()) & 0xF > self.read_register_a() & 0xF {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                if self.read_register_a() - self.read_register_b() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.write_register_a(self.read_register_a() - self.read_register_b());

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0xAF => {
                trace!("0xAF: XOR A. A:{:#04X} -> {:#04X}", self.read_register_a(), self.read_register_a() ^ self.read_register_a());

                // Instruction
                self.write_register_a(self.read_register_a() ^ self.read_register_a());

                // Set flags
                if self.read_register_a() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }
                self.unset_flag_bit(SUBTRACTION_BIT);
                self.unset_flag_bit(HALF_CARRY_BIT);
                self.unset_flag_bit(CARRY_BIT);

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0xBE => {
                trace!("0xBE: CP (HL). A:{:#04X} (HL):{:#04X}", self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                let a = self.read_register_a();
                let hl_byte = mmu.read_byte(self.read_register_hl());

                self.set_flag_bit(SUBTRACTION_BIT);

                if a == hl_byte {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                if a < hl_byte {
                    self.set_flag_bit(CARRY_BIT);
                } else {
                    self.unset_flag_bit(CARRY_BIT);
                }

                if ((a.wrapping_sub(hl_byte)) & 0xF) > (a & 0xF) {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.clock.m = 1;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0xC1 => {
                trace!("0xC1: POP BC. BC:{:#06X} <- {:#06X} SP:{:#06X}", self.read_register_bc(), mmu.read_word(self.stack_pointer), self.stack_pointer);

                self.write_register_bc(mmu.read_word(self.stack_pointer));
                self.stack_pointer += 2;

                self.clock.m = 1;
                self.clock.t = 12;
                self.program_counter += 1;
            },
            0xC3 => {
                trace!("0xC3: JP a16. Jump to {:#06X}", mmu.read_word(self.program_counter + 1));

                self.program_counter = mmu.read_word(self.program_counter + 1);

                self.clock.m = 3;
                self.clock.t = 16;
            },
            0xC5 => {
                trace!("0xC5: PUSH BC. SP:{:#06X} <- BC:{:#06X}", self.stack_pointer - 2, self.read_register_bc());

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.read_register_bc());

                self.clock.m = 1;
                self.clock.t = 16;
                self.program_counter += 1;
            },
            0xCB => {
                error!("0xCB instruction in the wrong OpCode table");
                exit(1);
            },
            0xCD => {
                trace!("0xCD: CALL a16. Calling {:#06X}", mmu.read_word(self.program_counter + 1));

                // Write address of next instruction to the stack
                mmu.write_word(self.stack_pointer, self.program_counter + 3);

                // We wrote two bytes, so decrement accordingly (Stack grows downwards)
                self.stack_pointer -= 2;

                // Set program_counter to address of function
                self.program_counter = mmu.read_word(self.program_counter + 1);

                self.clock.m = 3;
                self.clock.t = 24;
            },
            0xC9 => {
                trace!("0xC9: RET. Returning to {:#06X}", mmu.read_word(self.stack_pointer + 2));

                // Increment SP to find the return address
                self.stack_pointer += 2;

                // Jump there
                self.program_counter = mmu.read_word(self.stack_pointer);

                self.clock.m = 1;
                self.clock.t = 16;
            },
            0xE0 => {
                trace!("0xE0: LDH ($FF00+a8),A. $FF00+a8:{:#06X} <- A:{:#04X}", (0xFF00 + mmu.read_byte(self.program_counter + 1) as u16), self.read_register_a());

                let next_byte = mmu.read_byte(self.program_counter + 1) as u16;
                mmu.write_byte(0xFF00 + next_byte, self.read_register_a());

                self.clock.m = 2;
                self.clock.t = 12;
                self.program_counter += 2;
            },
            0xE2 => {
                trace!("0xE2: LD ($FF00+C),A. $FF00+C:{:#06X} <- A:{:#04X}", (0xFF00 + self.read_register_c() as u16), self.read_register_a());

                mmu.write_byte(0xFF00 + self.read_register_c() as u16, self.read_register_a());

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 1;
            },
            0xEA => {
                trace!("0xEA: LD (a16),A. a16:{:#06X} <- A:{:#04X}", mmu.read_word(self.program_counter + 1), self.read_register_a());

                let next_word = mmu.read_word(self.program_counter + 1);
                mmu.write_byte(next_word, self.read_register_a());

                self.clock.m = 3;
                self.clock.t = 16;
                self.program_counter += 3;
            },
            0xF0 => {
                trace!("0xF0: LDH A,($FF00+a8). A:{:#04X} <- {:#04X}, Value of {:#06X}", self.read_register_a(), mmu.read_byte(self.program_counter + 1), (0xFF00 + mmu.read_byte(self.program_counter + 1) as u16));

                let next_byte = mmu.read_byte(self.program_counter + 1);
                self.write_register_a(mmu.read_byte(0xFF00 + next_byte as u16));

                self.clock.m = 2;
                self.clock.t = 12;
                self.program_counter += 2;
            },
            0xF3 => {
                trace!("0xF3: Disable Interrupts.");

                self.interrupt_master_enable = false;

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0xFB => {
                trace!("0xFB: Enable Interrupts.");

                self.interrupt_master_enable = true;

                self.clock.m = 1;
                self.clock.t = 4;
                self.program_counter += 1;
            },
            0xFE => {
                trace!("0xFE: CP d8. A:{:#04X} d8:{:#04X}", self.read_register_a(), mmu.read_byte(self.program_counter + 1));

                let a = self.read_register_a();
                let next_byte = mmu.read_byte(self.program_counter + 1);

                self.set_flag_bit(SUBTRACTION_BIT);

                if a == next_byte {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                if a < next_byte {
                    self.set_flag_bit(CARRY_BIT);
                } else {
                    self.unset_flag_bit(CARRY_BIT);
                }

                if ((a.wrapping_sub(next_byte)) & 0xF) > (a & 0xF) {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            _ => {
                error!("Unknown OpCode {:#04X}", opcode);
                exit(1);
            }
        }
    }

    fn process_cb_opcode(&mut self, mmu: &mut MMU) {
        let opcode = mmu.read_byte(self.program_counter + 1);

        match opcode {
            0x11 => {
                trace!("0xCB11: RL C. C: {:#04X}", self.read_register_c());

                // Store carry flag
                let carry = self.read_flag(CARRY_BIT);
                let c = self.read_register_c();

                // Unset flag bits
                self.unset_flag_bit(SUBTRACTION_BIT);
                self.unset_flag_bit(HALF_CARRY_BIT);

                if self.most_significant_bit(c) > 0 {
                    self.set_flag_bit(CARRY_BIT);
                } else {
                    self.unset_flag_bit(CARRY_BIT);
                }

                self.write_register_c((c << 1) | carry);

                // Set zero bit
                if self.read_register_c() == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            0x7C => {
                trace!("0xCB7C: BIT 7,H. H:{:#04X}", self.read_register_h());

                // Test bit
                if self.most_significant_bit(self.read_register_h()) == 0 {
                    self.set_flag_bit(ZERO_BIT)
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                // Set flags
                self.set_flag_bit(HALF_CARRY_BIT);
                self.unset_flag_bit(SUBTRACTION_BIT);

                self.clock.m = 2;
                self.clock.t = 8;
                self.program_counter += 2;
            },
            _ => {
                error!("Unknown 0xCB OpCode {:#04X}", opcode);
                exit(1);
            }
        }
    }

    fn read_register_a(&self) -> u8 {
        unsafe {
            return self.af.hilo.hi;
        }
    }

    fn read_register_b(&self) -> u8 {
        unsafe {
            return self.bc.hilo.hi;
        }
    }

    fn read_register_c(&self) -> u8 {
        unsafe {
            return self.bc.hilo.lo;
        }
    }

    fn read_register_d(&self) -> u8 {
        unsafe {
            return self.de.hilo.hi;
        }
    }

    fn read_register_e(&self) -> u8 {
        unsafe {
            return self.de.hilo.lo;
        }
    }

    fn read_register_f(&self) -> u8 {
        unsafe {
            return self.af.hilo.lo;
        }
    }

    fn read_register_h(&self) -> u8 {
        unsafe {
            return self.hl.hilo.hi;
        }
    }

    fn read_register_l(&self) -> u8 {
        unsafe {
            return self.hl.hilo.lo;
        }
    }

    fn read_register_af(&self) -> u16 {
        unsafe {
            return self.af.word;
        }
    }

    fn read_register_bc(&self) -> u16 {
        unsafe {
            return self.bc.word;
        }
    }

    fn read_register_de(&self) -> u16 {
        unsafe {
            return self.de.word;
        }
    }

    fn read_register_hl(&self) -> u16 {
        unsafe {
            return self.hl.word;
        }
    }

    fn write_register_a(&mut self, a: u8) {
        self.af.hilo.hi = a;
    }

    fn write_register_b(&mut self, b: u8) {
        self.bc.hilo.hi = b;
    }

    fn write_register_c(&mut self, c: u8) {
        self.bc.hilo.lo = c;
    }

    fn write_register_d(&mut self, d: u8) {
        self.de.hilo.hi = d;
    }

    fn write_register_e(&mut self, e: u8) {
        self.de.hilo.lo = e;
    }

    pub fn write_register_f(&mut self, f: u8) {
        self.af.hilo.lo = f;
    }

    fn write_register_h(&mut self, h: u8) {
        self.hl.hilo.hi = h;
    }

    fn write_register_l(&mut self, l: u8) {
        self.hl.hilo.lo = l;
    }

    pub fn write_register_af(&mut self, af: u16) {
        self.af.word = af;
    }

    pub fn write_register_bc(&mut self, bc: u16) {
        self.bc.word = bc;
    }

    pub fn write_register_de(&mut self, de: u16) {
        self.de.word = de;
    }

    pub fn write_register_hl(&mut self, hl: u16) {
        self.hl.word = hl;
    }

    fn set_flag_bit(&mut self, bit: u8) {
        self.write_register_f(self.read_register_f() | bit);
    }

    fn unset_flag_bit(&mut self, bit: u8) {
        self.write_register_f(self.read_register_f() & !bit);
    }

    fn most_significant_bit(&mut self, x: u8) -> u8 {
        return 0x80 & x;
    }

    fn read_flag(&self, flag: u8) -> u8 {
        match flag {
            ZERO_BIT => return (self.read_register_f() & ZERO_BIT) >> 7,
            SUBTRACTION_BIT => return (self.read_register_f() & SUBTRACTION_BIT) >> 6,
            HALF_CARRY_BIT => return (self.read_register_f() & HALF_CARRY_BIT) >> 5,
            CARRY_BIT => return (self.read_register_f() & CARRY_BIT) >> 4,
            _ => {
                error!("Tried to read unknown flag {:#04X}", flag);
            }
        }

        exit(1);
    }

    //TODO - other functions
//    #define SET_BIT(x,y)            (x |= (0x01 << y))
//
//    #define UNSET_BIT(x,y)          (x &= ~(0x01 << y))

//    #define LSB(x)                  (x & 0x01)
//
//    #define BYTESWAP(x)             ((x << 8) | (x & 0x00FF))
//
//    uint16_t GetPC();
}