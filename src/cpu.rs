use std::process::exit;

use crate::mmu::MMU;

// Flag Bits
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

static OPERATION_BYTES: [u16; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, // 0
    1, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 1
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 2
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 8
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 9
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // A
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // B
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, // C
    1, 1, 3, 0, 3, 1, 2, 1, 1, 1, 3, 0, 3, 0, 2, 1, // D
    2, 1, 1, 0, 0, 1, 2, 1, 2, 1, 3, 0, 0, 0, 2, 1, // E
    2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 3, 1, 0, 0, 2, 1  // F
];

static OPERATION_MACHINE_CYCLES: [u32; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, // 0
    1, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 1
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 2
    2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 4
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 5
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 6
    2, 2, 2, 2, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, // 7
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 8
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 9
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // A
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // B
    2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 1, 3, 6, 2, 4, // C
    2, 3, 3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4, // D
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, // E
    3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4  // F
];

static OPERATION_MACHINE_CYCLES_BRANCHED: [u32; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, // 0
    1, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 1
    3, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 2
    3, 3, 2, 2, 3, 3, 3, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 4
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 5
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 6
    2, 2, 2, 2, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, // 7
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 8
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 9
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // A
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // B
    5, 3, 4, 4, 6, 4, 2, 4, 5, 4, 4, 1, 6, 6, 2, 4, // C
    5, 3, 4, 0, 6, 4, 2, 4, 5, 4, 4, 0, 6, 0, 2, 4, // D
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, // E
    3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4  // F
];

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
        let mut use_machine_cycles_branched: bool = false;
        let mut increment_program_counter: bool = true;

        match opcode {
            0x00 => {
                trace!("{:#04X}: NOP.", opcode);
            },
            0x01 => {
                trace!("{:#04X}: LD BC,d16. BC:{:#06X} <- d16:{:#06X}", opcode, self.read_register_bc(), mmu.read_word(self.program_counter + 1));

                self.write_register_bc(mmu.read_word(self.program_counter + 1));
            },
            0x02 => {
                trace!("{:#04X}: LD (BC),A. BC:{:#06X} <- A: {:#04X}", opcode, self.read_register_bc(), self.read_register_a());

                mmu.write_byte(self.read_register_bc(), self.read_register_a());
            },
            0x03 => {
                trace!("{:#04X}: INC BC. BC:{:#06X} -> {:#06X}", opcode, self.read_register_bc(), self.read_register_bc() + 1);

                self.write_register_bc(self.read_register_bc() + 1);
            },
            0x04 => {
                trace!("{:#04X}: INC B. B:{:#04X} -> {:#04X}", opcode, self.read_register_b(), self.read_register_b() + 1);

                let result = self.increase_register_u8(self.read_register_b());
                self.write_register_b(result);
            },
            0x05 => {
                trace!("{:#04X}: DEC B. B:{:#04X} -> {:#04X}", opcode, self.read_register_b(), self.read_register_b().wrapping_sub(1));

                let result = self.decrease_register_u8(self.read_register_b());
                self.write_register_b(result);
            },
            0x06 => {
                trace!("{:#04X}: LD B,d8. B:{:#04X} <- d8:{:#04X}", opcode, self.read_register_b(), mmu.read_byte(self.program_counter + 1));

                self.write_register_b(mmu.read_byte(self.program_counter + 1));
            },
            0x07 => {
                trace!("{:#04X}: RLC A. A:{:#04X}", opcode, self.read_register_a());

                let result = self.rotate_left(self.read_register_a());
                self.write_register_a(result);
            }
            0x08 => {
                trace!("{:#04X}: LD (a16),SP. (a16):{:#06X} <- SP:{:#06X}", opcode, mmu.read_word(self.program_counter + 1), self.stack_pointer);

                let next_word = mmu.read_word(self.program_counter + 1);
                mmu.write_word(next_word, self.stack_pointer);
            },
            0x09 => {
                trace!("{:#04X}: ADD HL,BC. HL:{:#06X} + BC:{:#06X}", opcode, self.read_register_hl(), self.read_register_bc());

                self.add_u16_to_hl(self.read_register_bc());
            },
            0x0A => {
                trace!("{:#04X}: LD A,(BC). A:{:#04X} <- (BC):{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_bc()));

                self.write_register_a(mmu.read_byte(self.read_register_bc()));
            },
            0x0B => {
                trace!("{:#04X}: DEC BC. BC:{:#04X} -> {:#04X}", opcode, self.read_register_bc(), self.read_register_bc() - 1);

                self.write_register_bc(self.read_register_bc() - 1);
            },
            0x0C => {
                trace!("{:#04X}: INC C. C:{:#04X} -> {:#04X}", opcode, self.read_register_c(), self.read_register_c() + 1);

                let result = self.increase_register_u8(self.read_register_c());
                self.write_register_c(result);
            },
            0x0D => {
                trace!("{:#04X}: DEC C. C:{:#04X} -> {:#04X}", opcode, self.read_register_c(), self.read_register_c() - 1);

                let result = self.decrease_register_u8(self.read_register_c());
                self.write_register_c(result);
            },
            0x0E => {
                trace!("{:#04X}: LD C,d8. C:{:#04X} <- d8:{:#04X}", opcode, self.read_register_c(), mmu.read_byte(self.program_counter + 1));

                self.write_register_c(mmu.read_byte(self.program_counter + 1));
            },
            0x0F => {
                trace!("{:#04X}: RRC A. A:{:#04X}", opcode, self.read_register_a());

                let result = self.rotate_right(self.read_register_a());
                self.write_register_a(result);
            },
            0x11 => {
                trace!("{:#04X}: LD DE,d16. DE:{:#06X} <- d16:{:#06X}", opcode, self.read_register_de(), mmu.read_word(self.program_counter + 1));

                self.write_register_de(mmu.read_word(self.program_counter + 1));
            },
            0x12 => {
                trace!("{:#04X}: LD (DE),A. DE:{:#06X} <- A: {:#04X}", opcode, self.read_register_de(), self.read_register_a());

                mmu.write_byte(self.read_register_de(), self.read_register_a());
            },
            0x13 => {
                trace!("{:#04X}: INC DE. DE:{:#06X} -> {:#06X}", opcode, self.read_register_de(), self.read_register_de() + 1);

                self.write_register_de(self.read_register_de() + 1);
            },
            0x14 => {
                trace!("{:#04X}: INC D. D:{:#04X} -> {:#04X}", opcode, self.read_register_d(), self.read_register_d() + 1);

                let result = self.increase_register_u8(self.read_register_d());
                self.write_register_d(result);
            },
            0x15 => {
                trace!("{:#04X}: DEC D. D:{:#04X} -> {:#04X}", opcode, self.read_register_d(), self.read_register_d() - 1);

                let result = self.decrease_register_u8(self.read_register_d());
                self.write_register_d(result);
            },
            0x16 => {
                trace!("{:#04X}: LD D,d8. D:{:#04X} <- d8:{:#04X}", opcode, self.read_register_d(), mmu.read_byte(self.program_counter + 1));

                self.write_register_d(mmu.read_byte(self.program_counter + 1));
            },
            0x17 => {
                trace!("{:#04X}: RL A. A:{:#04X}", opcode, self.read_register_a());

                let result = self.rotate_left_through_carry(self.read_register_a());
                self.write_register_a(result);
            },
            0x18 => {
                trace!("{:#04X}: JR r8.", opcode);

                let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                if next_byte_signed < 0 {
                    self.program_counter -= ((next_byte_signed + 2) * -1) as u16;
                } else {
                    self.program_counter += (next_byte_signed + 2) as u16;
                }

                increment_program_counter = false;
            },
            0x19 => {
                trace!("{:#04X}: ADD HL,DE. HL:{:#06X} + DE:{:#06X}", opcode, self.read_register_hl(), self.read_register_de());

                self.add_u16_to_hl(self.read_register_de());
            },
            0x1A => {
                trace!("{:#04X}: LD A,(DE). A:{:#04X} <- (DE):{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_de()));

                self.write_register_a(mmu.read_byte(self.read_register_de()));
            },
            0x1B => {
                trace!("{:#04X}: DEC DE. DE:{:#04X} -> {:#04X}", opcode, self.read_register_de(), self.read_register_de() - 1);

                self.write_register_de(self.read_register_de() - 1);
            },
            0x1C => {
                trace!("{:#04X}: INC E. E:{:#04X} -> {:#04X}", opcode, self.read_register_e(), self.read_register_e() + 1);

                let result = self.increase_register_u8(self.read_register_e());
                self.write_register_e(result);
            },
            0x1D => {
                trace!("{:#04X}: DEC E. E:{:#04X} -> {:#04X}", opcode, self.read_register_e(), self.read_register_e() - 1);

                let result = self.decrease_register_u8(self.read_register_e());
                self.write_register_e(result);
            },
            0x1E => {
                trace!("{:#04X}: LD E,d8. E:{:#04X} <- d8:{:#04X}", opcode, self.read_register_e(), mmu.read_byte(self.program_counter + 1));

                self.write_register_e(mmu.read_byte(self.program_counter + 1));
            },
            0x1F => {
                trace!("{:#04X}: RR A. A:{:#04X}", opcode, self.read_register_a());

                let result = self.rotate_right_through_carry(self.read_register_a());
                self.write_register_a(result);
            },
            0x20 => {
                trace!("{:#04X}: JR NZ,r8. Z:{:#04X}", opcode, self.read_flag(ZERO_BIT));

                if self.read_flag(ZERO_BIT) == 0 {
                    let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                    if next_byte_signed < 0 {
                        trace!("Jumping {}", next_byte_signed + 2);
                        self.program_counter -= ((next_byte_signed + 2) * -1) as u16;
                    } else {
                        trace!("Jumping {}", (next_byte_signed + 2));
                        self.program_counter += (next_byte_signed + 2) as u16;
                    }

                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0x21 => {
                trace!("{:#04X}: LD HL,d16. HL:{:#06X} <- d16:{:#06X}", opcode, self.read_register_hl(), mmu.read_word(self.program_counter + 1));

                self.write_register_hl(mmu.read_word(self.program_counter + 1));
            },
            0x22 => {
                trace!("{:#04X}: LD (HL+),A. (HL):{:#06X} <- A:{:#04X}", opcode, mmu.read_byte(self.read_register_hl()), self.read_register_a());

                mmu.write_byte(self.read_register_hl(), self.read_register_a());
                self.write_register_hl(self.read_register_hl() + 1);
            },
            0x23 => {
                trace!("{:#04X}: INC HL. HL:{:#06X} -> {:#06X}", opcode, self.read_register_hl(), self.read_register_hl() + 1);

                self.write_register_hl(self.read_register_hl() + 1);
            },
            0x24 => {
                trace!("{:#04X}: INC H. H:{:#04X} -> {:#04X}", opcode, self.read_register_h(), self.read_register_h() + 1);

                let result = self.increase_register_u8(self.read_register_h());
                self.write_register_h(result);
            },
            0x25 => {
                trace!("{:#04X}: DEC H. H:{:#04X} -> {:#04X}", opcode, self.read_register_h(), self.read_register_h().wrapping_sub(1));

                let result = self.decrease_register_u8(self.read_register_h());
                self.write_register_h(result);
            },
            0x26 => {
                trace!("{:#04X}: LD H,d8. H:{:#04X} <- d8:{:#04X}", opcode, self.read_register_h(), mmu.read_byte(self.program_counter + 1));

                self.write_register_h(mmu.read_byte(self.program_counter + 1));
            },
            0x27 => {
                trace!("{:#04X}: DAA. A:{:#04X}", opcode, self.read_register_a());

                let mut a: u16 = self.read_register_a() as u16;

                if self.read_flag(SUBTRACTION_BIT) == 0 {
                    if self.read_flag(HALF_CARRY_BIT) == 1 || (a & 0xF) > 9 {
                        a += 0x06;
                    }

                    if self.read_flag(CARRY_BIT) == 1 || (a > 0x9F) {
                        a += 0x60;
                    }
                } else {
                    if self.read_flag(HALF_CARRY_BIT) == 1 {
                        a = (a - 6) & 0xFF;
                    }

                    if self.read_flag(CARRY_BIT) == 1 {
                        a -= 0x60;
                    }
                }

                if a & 0x100 == 0x100 {
                    self.set_flag_bit(CARRY_BIT);
                }

                a &= 0xFF;

                if a == 0 {
                    self.set_flag_bit(ZERO_BIT);
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                self.unset_flag_bit(HALF_CARRY_BIT);

                self.write_register_a(a as u8);
            }
            0x28 => {
                trace!("{:#04X}: JR Z,r8. Z:{:#04X}", opcode, self.read_flag(ZERO_BIT));

                if self.read_flag(ZERO_BIT) == 1 {
                    let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                    if next_byte_signed < 0 {
                        trace!("Jumping {}", next_byte_signed + 2);
                        self.program_counter -= ((next_byte_signed + 2) * -1) as u16;
                    } else {
                        trace!("Jumping {}", next_byte_signed + 2);
                        self.program_counter += (next_byte_signed + 2) as u16;
                    }

                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0x29 => {
                trace!("{:#04X}: ADD HL,HL. HL:{:#06X} + HL:{:#06X}", opcode, self.read_register_hl(), self.read_register_hl());

                self.add_u16_to_hl(self.read_register_hl());
            },
            0x2A => {
                trace!("{:#04X}: LD A,(HL+). A:{:#04X} <- (HL):{:#06X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                let byte = mmu.read_byte(self.read_register_hl());
                self.write_register_a(byte);
                self.write_register_hl(self.read_register_hl() + 1);
            },
            0x2B => {
                trace!("{:#04X}: DEC HL. HL:{:#04X} -> {:#04X}", opcode, self.read_register_hl(), self.read_register_hl() - 1);

                self.write_register_hl(self.read_register_hl() - 1);
            },
            0x2C => {
                trace!("{:#04X}: INC L. L:{:#04X} -> {:#04X}", opcode, self.read_register_l(), self.read_register_l() + 1);

                let result = self.increase_register_u8(self.read_register_l());
                self.write_register_l(result);
            },
            0x2D => {
                trace!("{:#04X}: DEC L. L:{:#04X} -> {:#04X}", opcode, self.read_register_l(), self.read_register_l().wrapping_sub(1));

                let result = self.decrease_register_u8(self.read_register_l());
                self.write_register_l(result);
            },
            0x2E => {
                trace!("{:#04X}: LD L,d8. L:{:#04X} <- d8:{:#04X}", opcode, self.read_register_l(), mmu.read_byte(self.program_counter + 1));

                self.write_register_l(mmu.read_byte(self.program_counter + 1));
            },
            0x2F => {
                trace!("{:#04X}: CPL A. A:{:#04X}", opcode, self.read_register_a());

                self.write_register_a(!self.read_register_a());
                self.set_flag_bit(HALF_CARRY_BIT);
                self.set_flag_bit(SUBTRACTION_BIT);
            },
            0x30 => {
                trace!("{:#04X}: JR NC,r8. C:{:#04X}", opcode, self.read_flag(CARRY_BIT));

                if self.read_flag(CARRY_BIT) == 0 {
                    let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                    if next_byte_signed < 0 {
                        trace!("Jumping {}", next_byte_signed + 2);
                        self.program_counter -= ((next_byte_signed + 2) * -1) as u16;
                    } else {
                        trace!("Jumping {}", (next_byte_signed + 2));
                        self.program_counter += (next_byte_signed + 2) as u16;
                    }

                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0x31 => {
                trace!("{:#04X}: LD SP,d16. SP:{:#06X} <- d16:{:#06X}", opcode, self.stack_pointer, mmu.read_word(self.program_counter + 1));
                self.stack_pointer = mmu.read_word(self.program_counter + 1);
            },
            0x32 => {
                trace!("{:#04X}: LD (HL-),A. (HL):{:#06X} <- A:{:#04X}", opcode, mmu.read_byte(self.read_register_hl()), self.read_register_a());

                mmu.write_byte(self.read_register_hl(), self.read_register_a());
                self.write_register_hl(self.read_register_hl() - 1);
            },
            0x33 => {
                trace!("{:#04X}: INC SP. SP:{:#06X} -> {:#06X}", opcode, self.stack_pointer, self.stack_pointer + 1);

                self.stack_pointer += 1;
            },
            0x34 => {
                trace!("{:#04X}: INC (HL). (HL):{:#04X} -> {:#04X}", opcode, self.read_register_hl(), self.read_register_hl() + 1);

                let result = self.increase_register_u16(self.read_register_hl());
                self.write_register_hl(result);
            },
            0x35 => {
                trace!("{:#04X}: DEC (HL). (HL):{:#04X} -> {:#04X}", opcode, self.read_register_hl(), self.read_register_hl().wrapping_sub(1));

                let result = self.decrease_register_u16(self.read_register_hl());
                self.write_register_hl(result);
            },
            0x36 => {
                trace!("{:#04X}: LD (HL),d8. HL:{:#06X} <- d8:{:#04X}", opcode, self.read_register_hl(), mmu.read_byte(self.program_counter + 1));

                let next_byte = mmu.read_byte(self.program_counter + 1);
                mmu.write_byte(self.read_register_hl(), next_byte);
            },
            0x37 => {
                trace!("{:#04X}: SCF.", opcode);

                self.set_flag_bit(CARRY_BIT);
                self.unset_flag_bit(HALF_CARRY_BIT);
                self.unset_flag_bit(SUBTRACTION_BIT);
            },
            0x38 => {
                trace!("{:#04X}: JR C,r8. C:{:#04X}", opcode, self.read_flag(CARRY_BIT));

                if self.read_flag(CARRY_BIT) == 1 {
                    let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                    if next_byte_signed < 0 {
                        trace!("Jumping {}", next_byte_signed + 2);
                        self.program_counter -= ((next_byte_signed + 2) * -1) as u16;
                    } else {
                        trace!("Jumping {}", (next_byte_signed + 2));
                        self.program_counter += (next_byte_signed + 2) as u16;
                    }

                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0x39 => {
                trace!("{:#04X}: ADD HL,SP. HL:{:#06X} + SP:{:#06X}", opcode, self.read_register_hl(), self.stack_pointer);

                self.add_u16_to_hl(self.stack_pointer);
            },
            0x3A => {
                trace!("{:#04X}: LD A,(HL-). A:{:#04X} <- (HL):{:#06X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                let byte = mmu.read_byte(self.read_register_hl());
                self.write_register_a(byte);
                self.write_register_hl(self.read_register_hl() - 1);
            },
            0x3B => {
                trace!("{:#04X}: DEC SP. SP:{:#04X} -> {:#04X}", opcode, self.stack_pointer, self.stack_pointer - 1);

                self.stack_pointer -= 1;
            },
            0x3C => {
                trace!("{:#04X}: INC A. A:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() + 1);

                let result = self.increase_register_u8(self.read_register_a());
                self.write_register_a(result);
            },
            0x3D => {
                trace!("{:#04X}: DEC A. A:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() - 1);

                let result = self.decrease_register_u8(self.read_register_a());
                self.write_register_a(result);
            },
            0x3E => {
                trace!("{:#04X}: LD A,d8. A:{:#04X} <- d8:{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.program_counter + 1));

                self.write_register_a(mmu.read_byte(self.program_counter + 1));
            },
            0x3F => {
                trace!("{:#04X}: CCF.", opcode);

                if self.read_flag(CARRY_BIT) == 1 {
                    self.unset_flag_bit(CARRY_BIT);
                } else {
                    self.set_flag_bit(CARRY_BIT);
                }

                self.unset_flag_bit(HALF_CARRY_BIT);
                self.unset_flag_bit(SUBTRACTION_BIT);
            },
            0x40 => {
                trace!("{:#04X}: LD B,B. B:{:#04X} <- B:{:#04X}", opcode, self.read_register_b(), self.read_register_b());

                self.write_register_b(self.read_register_b());
            },
            0x41 => {
                trace!("{:#04X}: LD B,C. B:{:#04X} <- C:{:#04X}", opcode, self.read_register_b(), self.read_register_c());

                self.write_register_b(self.read_register_c());
            },
            0x42 => {
                trace!("{:#04X}: LD B,D. B:{:#04X} <- D:{:#04X}", opcode, self.read_register_b(), self.read_register_d());

                self.write_register_b(self.read_register_d());
            },
            0x43 => {
                trace!("{:#04X}: LD B,E. B:{:#04X} <- E:{:#04X}", opcode, self.read_register_b(), self.read_register_e());

                self.write_register_b(self.read_register_e());
            },
            0x44 => {
                trace!("{:#04X}: LD B,H. B:{:#04X} <- H:{:#04X}", opcode, self.read_register_b(), self.read_register_h());

                self.write_register_b(self.read_register_h());
            },
            0x45 => {
                trace!("{:#04X}: LD B,L. B:{:#04X} <- L:{:#04X}", opcode, self.read_register_b(), self.read_register_l());

                self.write_register_b(self.read_register_l());
            },
            0x46 => {
                trace!("{:#04X}: LD B,(HL). B:{:#04X} <- (HL):{:#04X}", opcode, self.read_register_b(), mmu.read_byte(self.read_register_hl()));

                self.write_register_a(mmu.read_byte(self.read_register_hl()));
            },
            0x47 => {
                trace!("{:#04X}: LD B,A. B:{:#04X} <- A:{:#04X}", opcode, self.read_register_b(), self.read_register_a());

                self.write_register_b(self.read_register_a());
            },
            0x48 => {
                trace!("{:#04X}: LD C,B. C:{:#04X} <- B:{:#04X}", opcode, self.read_register_c(), self.read_register_b());

                self.write_register_c(self.read_register_b());
            },
            0x49 => {
                trace!("{:#04X}: LD C,C. C:{:#04X} <- C:{:#04X}", opcode, self.read_register_c(), self.read_register_c());

                self.write_register_c(self.read_register_c());
            },
            0x4A => {
                trace!("{:#04X}: LD C,D. C:{:#04X} <- D:{:#04X}", opcode, self.read_register_c(), self.read_register_d());

                self.write_register_c(self.read_register_d());
            },
            0x4B => {
                trace!("{:#04X}: LD C,E. C:{:#04X} <- E:{:#04X}", opcode, self.read_register_c(), self.read_register_e());

                self.write_register_c(self.read_register_e());
            },
            0x4C => {
                trace!("{:#04X}: LD C,H. C:{:#04X} <- H:{:#04X}", opcode, self.read_register_c(), self.read_register_h());

                self.write_register_c(self.read_register_h());
            },
            0x4D => {
                trace!("{:#04X}: LD C,L. C:{:#04X} <- L:{:#04X}", opcode, self.read_register_c(), self.read_register_l());

                self.write_register_c(self.read_register_l());
            },
            0x4E => {
                trace!("{:#04X}: LD C,(HL). C:{:#04X} <- (HL):{:#04X}", opcode, self.read_register_c(), mmu.read_byte(self.read_register_hl()));

                self.write_register_c(mmu.read_byte(self.read_register_hl()));
            },
            0x4F => {
                trace!("{:#04X}: LD C,A. C:{:#04X} <- A:{:#04X}", opcode, self.read_register_c(), self.read_register_a());

                self.write_register_c(self.read_register_a());
            },
            0x50 => {
                trace!("{:#04X}: LD D,B. D:{:#04X} <- B:{:#04X}", opcode, self.read_register_d(), self.read_register_b());

                self.write_register_d(self.read_register_b());
            },
            0x51 => {
                trace!("{:#04X}: LD D,C. D:{:#04X} <- C:{:#04X}", opcode, self.read_register_d(), self.read_register_c());

                self.write_register_d(self.read_register_c());
            },
            0x52 => {
                trace!("{:#04X}: LD D,D. D:{:#04X} <- D:{:#04X}", opcode, self.read_register_d(), self.read_register_d());

                self.write_register_d(self.read_register_d());
            },
            0x53 => {
                trace!("{:#04X}: LD D,E. D:{:#04X} <- E:{:#04X}", opcode, self.read_register_d(), self.read_register_e());

                self.write_register_d(self.read_register_e());
            },
            0x54 => {
                trace!("{:#04X}: LD D,H. D:{:#04X} <- H:{:#04X}", opcode, self.read_register_d(), self.read_register_h());

                self.write_register_d(self.read_register_h());
            },
            0x55 => {
                trace!("{:#04X}: LD D,L. D:{:#04X} <- L:{:#04X}", opcode, self.read_register_d(), self.read_register_l());

                self.write_register_d(self.read_register_l());
            },
            0x56 => {
                trace!("{:#04X}: LD D,(HL). D:{:#04X} <- (HL):{:#04X}", opcode, self.read_register_d(), mmu.read_byte(self.read_register_hl()));

                self.write_register_d(mmu.read_byte(self.read_register_hl()));
            },
            0x57 => {
                trace!("{:#04X}: LD D,A. D:{:#04X} <- A:{:#04X}", opcode, self.read_register_d(), self.read_register_a());

                self.write_register_d(self.read_register_a());
            },
            0x58 => {
                trace!("{:#04X}: LD E,B. E:{:#04X} <- B:{:#04X}", opcode, self.read_register_e(), self.read_register_b());

                self.write_register_e(self.read_register_b());
            },
            0x59 => {
                trace!("{:#04X}: LD E,C. E:{:#04X} <- C:{:#04X}", opcode, self.read_register_e(), self.read_register_c());

                self.write_register_e(self.read_register_c());
            },
            0x5A => {
                trace!("{:#04X}: LD E,D. E:{:#04X} <- D:{:#04X}", opcode, self.read_register_e(), self.read_register_d());

                self.write_register_e(self.read_register_d());
            },
            0x5B => {
                trace!("{:#04X}: LD E,E. E:{:#04X} <- E:{:#04X}", opcode, self.read_register_e(), self.read_register_e());

                self.write_register_e(self.read_register_e());
            },
            0x5C => {
                trace!("{:#04X}: LD E,H. E:{:#04X} <- H:{:#04X}", opcode, self.read_register_e(), self.read_register_h());

                self.write_register_e(self.read_register_h());
            },
            0x5D => {
                trace!("{:#04X}: LD E,L. E:{:#04X} <- L:{:#04X}", opcode, self.read_register_e(), self.read_register_l());

                self.write_register_e(self.read_register_l());
            },
            0x5E => {
                trace!("{:#04X}: LD E,(HL). E:{:#04X} <- (HL):{:#04X}", opcode, self.read_register_e(), mmu.read_byte(self.read_register_hl()));

                self.write_register_e(mmu.read_byte(self.read_register_hl()));
            },
            0x5F => {
                trace!("{:#04X}: LD E,A. E:{:#04X} <- A:{:#04X}", opcode, self.read_register_e(), self.read_register_a());

                self.write_register_e(self.read_register_a());
            },
            0x60 => {
                trace!("{:#04X}: LD H,B. H:{:#04X} <- B:{:#04X}", opcode, self.read_register_h(), self.read_register_b());

                self.write_register_h(self.read_register_b());
            },
            0x61 => {
                trace!("{:#04X}: LD H,C. H:{:#04X} <- C:{:#04X}", opcode, self.read_register_h(), self.read_register_c());

                self.write_register_h(self.read_register_c());
            },
            0x62 => {
                trace!("{:#04X}: LD H,D. H:{:#04X} <- D:{:#04X}", opcode, self.read_register_h(), self.read_register_d());

                self.write_register_h(self.read_register_d());
            },
            0x63 => {
                trace!("{:#04X}: LD H,E. H:{:#04X} <- E:{:#04X}", opcode, self.read_register_h(), self.read_register_e());

                self.write_register_h(self.read_register_e());
            },
            0x64 => {
                trace!("{:#04X}: LD H,H. H:{:#04X} <- H:{:#04X}", opcode, self.read_register_h(), self.read_register_h());

                self.write_register_h(self.read_register_h());
            },
            0x65 => {
                trace!("{:#04X}: LD H,L. H:{:#04X} <- L:{:#04X}", opcode, self.read_register_h(), self.read_register_l());

                self.write_register_h(self.read_register_l());
            },
            0x66 => {
                trace!("{:#04X}: LD H,(HL). H:{:#04X} <- (HL):{:#04X}", opcode, self.read_register_h(), mmu.read_byte(self.read_register_hl()));

                self.write_register_h(mmu.read_byte(self.read_register_hl()));
            },
            0x67 => {
                trace!("{:#04X}: LD H,A. H:{:#04X} <- A:{:#04X}", opcode, self.read_register_h(), self.read_register_a());

                self.write_register_h(self.read_register_a());
            },
            0x68 => {
                trace!("{:#04X}: LD L,B. L:{:#04X} <- B:{:#04X}", opcode, self.read_register_l(), self.read_register_b());

                self.write_register_l(self.read_register_b());
            },
            0x69 => {
                trace!("{:#04X}: LD L,C. L:{:#04X} <- C:{:#04X}", opcode, self.read_register_l(), self.read_register_c());

                self.write_register_l(self.read_register_c());
            },
            0x6A => {
                trace!("{:#04X}: LD L,D. L:{:#04X} <- D:{:#04X}", opcode, self.read_register_l(), self.read_register_d());

                self.write_register_l(self.read_register_d());
            },
            0x6B => {
                trace!("{:#04X}: LD L,E. L:{:#04X} <- E:{:#04X}", opcode, self.read_register_l(), self.read_register_e());

                self.write_register_l(self.read_register_e());
            },
            0x6C => {
                trace!("{:#04X}: LD L,H. L:{:#04X} <- H:{:#04X}", opcode, self.read_register_l(), self.read_register_h());

                self.write_register_l(self.read_register_h());
            },
            0x6D => {
                trace!("{:#04X}: LD L,L. L:{:#04X} <- L:{:#04X}", opcode, self.read_register_l(), self.read_register_l());

                self.write_register_l(self.read_register_l());
            },
            0x6E => {
                trace!("{:#04X}: LD L,(HL). L:{:#04X} <- (HL):{:#04X}", opcode, self.read_register_l(), mmu.read_byte(self.read_register_hl()));

                self.write_register_l(mmu.read_byte(self.read_register_hl()));
            },
            0x6F => {
                trace!("{:#04X}: LD L,A. L:{:#04X} <- A:{:#04X}", opcode, self.read_register_l(), self.read_register_a());

                self.write_register_l(self.read_register_a());
            },
            0x70 => {
                trace!("{:#04X}: LD (HL),B. HL:{:#06X} <- B:{:#04X}", opcode, self.read_register_hl(), self.read_register_b());

                mmu.write_byte(self.read_register_hl(), self.read_register_b());
            },
            0x71 => {
                trace!("{:#04X}: LD (HL),C. HL:{:#06X} <- C:{:#04X}", opcode, self.read_register_hl(), self.read_register_c());

                mmu.write_byte(self.read_register_hl(), self.read_register_c());
            },
            0x72 => {
                trace!("{:#04X}: LD (HL),D. HL:{:#06X} <- D:{:#04X}", opcode, self.read_register_hl(), self.read_register_d());

                mmu.write_byte(self.read_register_hl(), self.read_register_d());
            },
            0x73 => {
                trace!("{:#04X}: LD (HL),E. HL:{:#06X} <- E:{:#04X}", opcode, self.read_register_hl(), self.read_register_e());

                mmu.write_byte(self.read_register_hl(), self.read_register_e());
            },
            0x74 => {
                trace!("{:#04X}: LD (HL),H. HL:{:#06X} <- H:{:#04X}", opcode, self.read_register_hl(), self.read_register_h());

                mmu.write_byte(self.read_register_hl(), self.read_register_h());
            },
            0x75 => {
                trace!("{:#04X}: LD (HL),L. HL:{:#06X} <- L:{:#04X}", opcode, self.read_register_hl(), self.read_register_l());

                mmu.write_byte(self.read_register_hl(), self.read_register_b());
            },
            0x77 => {
                trace!("{:#04X}: LD (HL),A. HL:{:#06X} <- A: {:#04X}", opcode, self.read_register_hl(), self.read_register_a());

                mmu.write_byte(self.read_register_hl(), self.read_register_a());
            },
            0x78 => {
                trace!("{:#04X}: LD A,B. A:{:#04X} <- B:{:#04X}", opcode, self.read_register_a(), self.read_register_b());

                self.write_register_a(self.read_register_b());
            },
            0x79 => {
                trace!("{:#04X}: LD A,C. A:{:#04X} <- C:{:#04X}", opcode, self.read_register_a(), self.read_register_c());

                self.write_register_a(self.read_register_c());
            },
            0x7A => {
                trace!("{:#04X}: LD A,D. A:{:#04X} <- D:{:#04X}", opcode, self.read_register_a(), self.read_register_d());

                self.write_register_a(self.read_register_d());
            },
            0x7B => {
                trace!("{:#04X}: LD A,E. A:{:#04X} <- E:{:#04X}", opcode, self.read_register_a(), self.read_register_e());

                self.write_register_a(self.read_register_e());
            },
            0x7C => {
                trace!("{:#04X}: LD A,H. A:{:#04X} <- H:{:#04X}", opcode, self.read_register_a(), self.read_register_h());

                self.write_register_a(self.read_register_h());
            },
            0x7D => {
                trace!("{:#04X}: LD A,L. A:{:#04X} <- L:{:#04X}", opcode, self.read_register_a(), self.read_register_l());

                self.write_register_a(self.read_register_l());
            },
            0x7E => {
                trace!("{:#04X}: LD A,(HL). A:{:#04X} <- (HL):{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                self.write_register_a(mmu.read_byte(self.read_register_hl()));
            },
            0x7F => {
                trace!("{:#04X}: LD A,A. A:{:#04X} <- A:{:#04X}", opcode, self.read_register_a(), self.read_register_a());

                self.write_register_a(self.read_register_a());
            },
            0x80 => {
                trace!("{:#04X}: ADD A,B. A = {:#04X} + B:{:#04X}", opcode, self.read_register_a(), self.read_register_b());

                self.add_u8_to_a(self.read_register_b());
            },
            0x81 => {
                trace!("{:#04X}: ADD A,C. A = {:#04X} + C:{:#04X}", opcode, self.read_register_a(), self.read_register_c());

                self.add_u8_to_a(self.read_register_c());
            },
            0x82 => {
                trace!("{:#04X}: ADD A,D. A = {:#04X} + D:{:#04X}", opcode, self.read_register_a(), self.read_register_d());

                self.add_u8_to_a(self.read_register_d());
            },
            0x83 => {
                trace!("{:#04X}: ADD A,E. A = {:#04X} + E:{:#04X}", opcode, self.read_register_a(), self.read_register_e());

                self.add_u8_to_a(self.read_register_e());
            },
            0x84 => {
                trace!("{:#04X}: ADD A,H. A = {:#04X} + H:{:#04X}", opcode, self.read_register_a(), self.read_register_h());

                self.add_u8_to_a(self.read_register_h());
            },
            0x85 => {
                trace!("{:#04X}: ADD A,L. A = {:#04X} + L:{:#04X}", opcode, self.read_register_a(), self.read_register_l());

                self.add_u8_to_a(self.read_register_l());
            },
            0x86 => {
                trace!("{:#04X}: ADD A,(HL). A = {:#04X} + {:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                self.add_u8_to_a(mmu.read_byte(self.read_register_hl()));
            },
            0x87 => {
                trace!("{:#04X}: ADD A,A. A = {:#04X} + A:{:#04X}", opcode, self.read_register_a(), self.read_register_a());

                self.add_u8_to_a(self.read_register_a());
            },
            0x88 => {
                trace!("{:#04X}: ADC A,B. A = {:#04X} + B:{:#04X}", opcode, self.read_register_a(), self.read_register_b());

                self.add_u8_and_carry_to_a(self.read_register_b());
            },
            0x89 => {
                trace!("{:#04X}: ADC A,C. A = {:#04X} + C:{:#04X}", opcode, self.read_register_a(), self.read_register_c());

                self.add_u8_and_carry_to_a(self.read_register_c());
            },
            0x8A => {
                trace!("{:#04X}: ADC A,D. A = {:#04X} + D:{:#04X}", opcode, self.read_register_a(), self.read_register_d());

                self.add_u8_and_carry_to_a(self.read_register_d());
            },
            0x8B => {
                trace!("{:#04X}: ADC A,E. A = {:#04X} + E:{:#04X}", opcode, self.read_register_a(), self.read_register_e());

                self.add_u8_and_carry_to_a(self.read_register_e());
            },
            0x8C => {
                trace!("{:#04X}: ADC A,H. A = {:#04X} + H:{:#04X}", opcode, self.read_register_a(), self.read_register_h());

                self.add_u8_and_carry_to_a(self.read_register_h());
            },
            0x8D => {
                trace!("{:#04X}: ADC A,L. A = {:#04X} + L:{:#04X}", opcode, self.read_register_a(), self.read_register_l());

                self.add_u8_and_carry_to_a(self.read_register_l());
            },
            0x8E => {
                trace!("{:#04X}: ADC A,(HL). A = {:#04X} + (HL):{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                self.add_u8_and_carry_to_a(mmu.read_byte(self.read_register_hl()));
            },
            0x8F => {
                trace!("{:#04X}: ADC A,A. A = {:#04X} + A:{:#04X}", opcode, self.read_register_a(), self.read_register_a());

                self.add_u8_and_carry_to_a(self.read_register_a());
            },
            0x90 => {
                trace!("{:#04X}: SUB B. A:{:#04X} - B:{:#04X}", opcode, self.read_register_a(), self.read_register_b());

                self.subtract_u8_from_a(self.read_register_b());
            },
            0x91 => {
                trace!("{:#04X}: SUB C. A:{:#04X} - C:{:#04X}", opcode, self.read_register_a(), self.read_register_c());

                self.subtract_u8_from_a(self.read_register_c());
            },
            0x92 => {
                trace!("{:#04X}: SUB D. A:{:#04X} - D:{:#04X}", opcode, self.read_register_a(), self.read_register_d());

                self.subtract_u8_from_a(self.read_register_d());
            },
            0x93 => {
                trace!("{:#04X}: SUB E. A:{:#04X} - E:{:#04X}", opcode, self.read_register_a(), self.read_register_e());

                self.subtract_u8_from_a(self.read_register_e());
            },
            0x94 => {
                trace!("{:#04X}: SUB H. A:{:#04X} - H:{:#04X}", opcode, self.read_register_a(), self.read_register_h());

                self.subtract_u8_from_a(self.read_register_h());
            },
            0x95 => {
                trace!("{:#04X}: SUB L. A:{:#04X} - L:{:#04X}", opcode, self.read_register_a(), self.read_register_l());

                self.subtract_u8_from_a(self.read_register_l());
            },
            0x96 => {
                trace!("{:#04X}: SUB (HL). A:{:#04X} - (HL):{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                self.subtract_u8_from_a(mmu.read_byte(self.read_register_hl()));
            },
            0x97 => {
                trace!("{:#04X}: SUB A. A:{:#04X} - A:{:#04X}", opcode, self.read_register_a(), self.read_register_a());

                self.subtract_u8_from_a(self.read_register_a());
            },
            0x98 => {
                trace!("{:#04X}: SBC B. A:{:#04X} - B:{:#04X}", opcode, self.read_register_a(), self.read_register_b());

                self.subtract_u8_and_carry_from_a(self.read_register_b());
            },
            0x99 => {
                trace!("{:#04X}: SBC C. A:{:#04X} - C:{:#04X}", opcode, self.read_register_a(), self.read_register_c());

                self.subtract_u8_and_carry_from_a(self.read_register_c());
            },
            0x9A => {
                trace!("{:#04X}: SBC D. A:{:#04X} - D:{:#04X}", opcode, self.read_register_a(), self.read_register_d());

                self.subtract_u8_and_carry_from_a(self.read_register_d());
            },
            0x9B => {
                trace!("{:#04X}: SBC E. A:{:#04X} - E:{:#04X}", opcode, self.read_register_a(), self.read_register_e());

                self.subtract_u8_and_carry_from_a(self.read_register_e());
            },
            0x9C => {
                trace!("{:#04X}: SBC H. A:{:#04X} - H:{:#04X}", opcode, self.read_register_a(), self.read_register_h());

                self.subtract_u8_and_carry_from_a(self.read_register_h());
            },
            0x9D => {
                trace!("{:#04X}: SBC L. A:{:#04X} - L:{:#04X}", opcode, self.read_register_a(), self.read_register_l());

                self.subtract_u8_and_carry_from_a(self.read_register_l());
            },
            0x9E => {
                trace!("{:#04X}: SBC (HL). A:{:#04X} - (HL):{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                self.subtract_u8_and_carry_from_a(mmu.read_byte(self.read_register_hl()));
            },
            0x9F => {
                trace!("{:#04X}: SBC A. A:{:#04X} - A:{:#04X}", opcode, self.read_register_a(), self.read_register_a());

                self.subtract_u8_and_carry_from_a(self.read_register_a());
            },
            0xA0 => {
                trace!("{:#04X}: AND B. B:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & self.read_register_b());

                self.and_with_register_a(self.read_register_b());
            },
            0xA1 => {
                trace!("{:#04X}: AND C. C:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & self.read_register_c());

                self.and_with_register_a(self.read_register_c());
            },
            0xA2 => {
                trace!("{:#04X}: AND D. D:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & self.read_register_d());

                self.and_with_register_a(self.read_register_d());
            },
            0xA3 => {
                trace!("{:#04X}: AND E. E:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & self.read_register_e());

                self.and_with_register_a(self.read_register_e());
            },
            0xA4 => {
                trace!("{:#04X}: AND H. H:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & self.read_register_h());

                self.and_with_register_a(self.read_register_h());
            },
            0xA5 => {
                trace!("{:#04X}: AND L. L:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & self.read_register_l());

                self.and_with_register_a(self.read_register_l());
            },
            0xA6 => {
                trace!("{:#04X}: AND (HL). B:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & mmu.read_byte(self.read_register_hl()));

                self.and_with_register_a(mmu.read_byte(self.read_register_hl()));
            },
            0xA7 => {
                trace!("{:#04X}: AND A. A:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & self.read_register_a());

                self.and_with_register_a(self.read_register_a());
            },
            0xA8 => {
                trace!("{:#04X}: XOR B. B:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ self.read_register_b());

                self.xor_with_register_a(self.read_register_b());
            },
            0xA9 => {
                trace!("{:#04X}: XOR C. C:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ self.read_register_c());

                self.xor_with_register_a(self.read_register_c());
            },
            0xAA => {
                trace!("{:#04X}: XOR D. D:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ self.read_register_d());

                self.xor_with_register_a(self.read_register_d());
            },
            0xAB => {
                trace!("{:#04X}: XOR E. E:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ self.read_register_e());

                self.xor_with_register_a(self.read_register_e());
            },
            0xAC => {
                trace!("{:#04X}: XOR H. H:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ self.read_register_h());

                self.xor_with_register_a(self.read_register_h());
            },
            0xAD => {
                trace!("{:#04X}: XOR L. L:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ self.read_register_l());

                self.xor_with_register_a(self.read_register_l());
            },
            0xAE => {
                trace!("{:#04X}: XOR (HL). (HL):{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ mmu.read_byte(self.read_register_hl()));

                self.xor_with_register_a(mmu.read_byte(self.read_register_hl()));
            },
            0xAF => {
                trace!("{:#04X}: XOR A. A:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ self.read_register_a());

                self.xor_with_register_a(self.read_register_a());
            },
            0xB0 => {
                trace!("{:#04X}: OR B. B:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | self.read_register_b());

                self.or_with_register_a(self.read_register_b());
            },
            0xB1 => {
                trace!("{:#04X}: OR C. C:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | self.read_register_c());

                self.or_with_register_a(self.read_register_c());
            },
            0xB2 => {
                trace!("{:#04X}: OR D. D:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | self.read_register_d());

                self.or_with_register_a(self.read_register_d());
            },
            0xB3 => {
                trace!("{:#04X}: OR E. E:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | self.read_register_e());

                self.or_with_register_a(self.read_register_e());
            },
            0xB4 => {
                trace!("{:#04X}: OR H. H:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | self.read_register_h());

                self.or_with_register_a(self.read_register_h());
            },
            0xB5 => {
                trace!("{:#04X}: OR L. L:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | self.read_register_l());

                self.or_with_register_a(self.read_register_l());
            },
            0xB6 => {
                trace!("{:#04X}: OR (HL). (HL):{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | mmu.read_byte(self.read_register_hl()));

                self.or_with_register_a(mmu.read_byte(self.read_register_hl()));
            },
            0xB7 => {
                trace!("{:#04X}: OR A. A:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | self.read_register_a());

                self.or_with_register_a(self.read_register_a());
            },
            0xB8 => {
                trace!("{:#04X}: CP B. A:{:#04X} B:{:#04X}", opcode, self.read_register_a(), self.read_register_b());

                self.compare_with_register_a(self.read_register_b());
            },
            0xB9 => {
                trace!("{:#04X}: CP C. A:{:#04X} C:{:#04X}", opcode, self.read_register_a(), self.read_register_c());

                self.compare_with_register_a(self.read_register_c());
            },
            0xBA => {
                trace!("{:#04X}: CP D. A:{:#04X} D:{:#04X}", opcode, self.read_register_a(), self.read_register_d());

                self.compare_with_register_a(self.read_register_d());
            },
            0xBB => {
                trace!("{:#04X}: CP E. A:{:#04X} E:{:#04X}", opcode, self.read_register_a(), self.read_register_e());

                self.compare_with_register_a(self.read_register_e());
            },
            0xBC => {
                trace!("{:#04X}: CP H. A:{:#04X} H:{:#04X}", opcode, self.read_register_a(), self.read_register_h());

                self.compare_with_register_a(self.read_register_h());
            },
            0xBD => {
                trace!("{:#04X}: CP L. A:{:#04X} L:{:#04X}", opcode, self.read_register_a(), self.read_register_l());

                self.compare_with_register_a(self.read_register_l());
            },
            0xBE => {
                trace!("{:#04X}: CP (HL). A:{:#04X} (HL):{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.read_register_hl()));

                self.compare_with_register_a(mmu.read_byte(self.read_register_hl()));
            },
            0xBF => {
                trace!("{:#04X}: CP A. A:{:#04X} A:{:#04X}", opcode, self.read_register_a(), self.read_register_a());

                self.compare_with_register_a(self.read_register_a());
            },
            0xC0 => {
                trace!("{:#04X}: RET NZ. Returning to {:#06X}", opcode, mmu.read_word(self.stack_pointer + 2));

                if self.read_flag(ZERO_BIT) == 0 {
                    // Increment SP to find the return address
                    self.stack_pointer += 2;

                    // Jump there
                    self.program_counter = mmu.read_word(self.stack_pointer);
                    increment_program_counter = false;
                }
            },
            0xC1 => {
                trace!("{:#04X}: POP BC. BC:{:#06X} <- {:#06X} SP:{:#06X}", opcode, self.read_register_bc(), mmu.read_word(self.stack_pointer), self.stack_pointer);

                self.write_register_bc(mmu.read_word(self.stack_pointer));
                self.stack_pointer += 2;
            },
            0xC2 => {
                trace!("{:#04X}: JP NZ,a16. Jump to {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                if self.read_flag(ZERO_BIT) == 0 {
                    trace!("Jumping to {}", mmu.read_word(self.program_counter + 1));
                    self.program_counter = mmu.read_word(self.program_counter + 1);
                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0xC3 => {
                trace!("{:#04X}: JP a16. Jump to {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                self.program_counter = mmu.read_word(self.program_counter + 1);
                increment_program_counter = false;
            },
            0xC4 => {
                trace!("{:#04X}: CALL NZ,a16. Calling {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                if self.read_flag(ZERO_BIT) == 0 {
                    trace!("Calling {}", mmu.read_word(self.program_counter + 1));

                    // Write address of next instruction to the stack
                    mmu.write_word(self.stack_pointer, self.program_counter + 3);

                    // We wrote two bytes, so decrement accordingly (Stack grows downwards)
                    self.stack_pointer -= 2;

                    // Set program_counter to address of function
                    self.program_counter = mmu.read_word(self.program_counter + 1);
                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0xC5 => {
                trace!("{:#04X}: PUSH BC. SP:{:#06X} <- BC:{:#06X}", opcode, self.stack_pointer - 2, self.read_register_bc());

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.read_register_bc());
            },
            0xC6 => {
                trace!("{:#04X}: ADD A,d8. A = {:#04X} + d8:{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.program_counter + 1));

                self.add_u8_to_a(mmu.read_byte(self.program_counter + 1));
            },
            0xC7 => {
                trace!("{:#04X}: RST 0x0000", opcode);

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.program_counter);

                self.program_counter = 0x0000;
                increment_program_counter = false;
            },
            0xC8 => {
                trace!("{:#04X}: RET Z. Returning to {:#06X}", opcode, mmu.read_word(self.stack_pointer + 2));

                if self.read_flag(ZERO_BIT) == 1 {
                    // Increment SP to find the return address
                    self.stack_pointer += 2;

                    // Jump there
                    self.program_counter = mmu.read_word(self.stack_pointer);
                    increment_program_counter = false;
                }
            },
            0xC9 => {
                trace!("{:#04X}: RET. Returning to {:#06X}", opcode, mmu.read_word(self.stack_pointer + 2));

                // Increment SP to find the return address
                self.stack_pointer += 2;

                // Jump there
                self.program_counter = mmu.read_word(self.stack_pointer);
                increment_program_counter = false;
            },
            0xCA => {
                trace!("{:#04X}: JP Z,a16. Jump to {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                if self.read_flag(ZERO_BIT) == 1 {
                    trace!("Jumping to {}", mmu.read_word(self.program_counter + 1));
                    self.program_counter = mmu.read_word(self.program_counter + 1);
                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0xCB => {
                error!("0xCB instruction in the wrong OpCode table");
                exit(1);
            },
            0xCC => {
                trace!("{:#04X}: CALL Z,a16. Calling {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                if self.read_flag(ZERO_BIT) == 1 {
                    trace!("Calling {}", mmu.read_word(self.program_counter + 1));

                    // Write address of next instruction to the stack
                    mmu.write_word(self.stack_pointer, self.program_counter + 3);

                    // We wrote two bytes, so decrement accordingly (Stack grows downwards)
                    self.stack_pointer -= 2;

                    // Set program_counter to address of function
                    self.program_counter = mmu.read_word(self.program_counter + 1);
                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0xCD => {
                trace!("{:#04X}: CALL a16. Calling {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                // Write address of next instruction to the stack
                mmu.write_word(self.stack_pointer, self.program_counter + 3);

                // We wrote two bytes, so decrement accordingly (Stack grows downwards)
                self.stack_pointer -= 2;

                // Set program_counter to address of function
                self.program_counter = mmu.read_word(self.program_counter + 1);
                increment_program_counter = false;
            },
            0xCE => {
                trace!("{:#04X}: ADC A,d8. A = {:#04X} + d8:{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.program_counter + 1));

                self.add_u8_and_carry_to_a(mmu.read_byte(self.program_counter + 1));
            },
            0xCF => {
                trace!("{:#04X}: RST 0x0008", opcode);

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.program_counter);

                self.program_counter = 0x0008;
                increment_program_counter = false;
            },
            0xD0 => {
                trace!("{:#04X}: RET NC. Returning to {:#06X}", opcode, mmu.read_word(self.stack_pointer + 2));

                if self.read_flag(CARRY_BIT) == 0 {
                    // Increment SP to find the return address
                    self.stack_pointer += 2;

                    // Jump there
                    self.program_counter = mmu.read_word(self.stack_pointer);
                    increment_program_counter = false;
                }
            },
            0xD1 => {
                trace!("{:#04X}: POP DE. DE:{:#06X} <- {:#06X} SP:{:#06X}", opcode, self.read_register_de(), mmu.read_word(self.stack_pointer), self.stack_pointer);

                self.write_register_de(mmu.read_word(self.stack_pointer));
                self.stack_pointer += 2;
            },
            0xD2 => {
                trace!("{:#04X}: JP NC,a16. Jump to {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                if self.read_flag(CARRY_BIT) == 0 {
                    trace!("Jumping to {}", mmu.read_word(self.program_counter + 1));
                    self.program_counter = mmu.read_word(self.program_counter + 1);
                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0xD4 => {
                trace!("{:#04X}: CALL NC,a16. Calling {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                if self.read_flag(CARRY_BIT) == 0 {
                    trace!("Calling {}", mmu.read_word(self.program_counter + 1));

                    // Write address of next instruction to the stack
                    mmu.write_word(self.stack_pointer, self.program_counter + 3);

                    // We wrote two bytes, so decrement accordingly (Stack grows downwards)
                    self.stack_pointer -= 2;

                    // Set program_counter to address of function
                    self.program_counter = mmu.read_word(self.program_counter + 1);
                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0xD5 => {
                trace!("{:#04X}: PUSH DE. SP:{:#06X} <- DE:{:#06X}", opcode, self.stack_pointer - 2, self.read_register_de());

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.read_register_de());
            },
            0xD6 => {
                trace!("{:#04X}: SUB d8. A:{:#04X} - d8:{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.program_counter + 1));

                self.subtract_u8_from_a(mmu.read_byte(self.program_counter + 1));
            },
            0xD7 => {
                trace!("{:#04X}: RST 0x0010", opcode);

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.program_counter);

                self.program_counter = 0x0010;
                increment_program_counter = false;
            },
            0xD8 => {
                trace!("{:#04X}: RET C. Returning to {:#06X}", opcode, mmu.read_word(self.stack_pointer + 2));

                if self.read_flag(CARRY_BIT) == 1 {
                    // Increment SP to find the return address
                    self.stack_pointer += 2;

                    // Jump there
                    self.program_counter = mmu.read_word(self.stack_pointer);
                    increment_program_counter = false;
                }
            },
            0xD9 => {
                trace!("{:#04X}: RETI. Returning to {:#06X}", opcode, mmu.read_word(self.stack_pointer + 2));

                // Increment SP to find the return address
                self.stack_pointer += 2;

                // Jump there
                self.program_counter = mmu.read_word(self.stack_pointer);
                increment_program_counter = false;

                self.interrupt_master_enable = true;
            },
            0xDA => {
                trace!("{:#04X}: JP C,a16. Jump to {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                if self.read_flag(CARRY_BIT) == 1 {
                    trace!("Jumping to {}", mmu.read_word(self.program_counter + 1));
                    self.program_counter = mmu.read_word(self.program_counter + 1);
                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0xDC => {
                trace!("{:#04X}: CALL C,a16. Calling {:#06X}", opcode, mmu.read_word(self.program_counter + 1));

                if self.read_flag(CARRY_BIT) == 1 {
                    trace!("Calling {}", mmu.read_word(self.program_counter + 1));

                    // Write address of next instruction to the stack
                    mmu.write_word(self.stack_pointer, self.program_counter + 3);

                    // We wrote two bytes, so decrement accordingly (Stack grows downwards)
                    self.stack_pointer -= 2;

                    // Set program_counter to address of function
                    self.program_counter = mmu.read_word(self.program_counter + 1);
                    increment_program_counter = false;
                    use_machine_cycles_branched = true;
                }
            },
            0xDE => {
                trace!("{:#04X}: SBC d8. A:{:#04X} - d8:{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.program_counter + 1));

                self.subtract_u8_and_carry_from_a(mmu.read_byte(self.program_counter + 1));
            },
            0xDF => {
                trace!("{:#04X}: RST 0x0018", opcode);

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.program_counter);

                self.program_counter = 0x0018;
                increment_program_counter = false;
            },
            0xE0 => {
                trace!("{:#04X}: LDH ($FF00+a8),A. $FF00+a8:{:#06X} <- A:{:#04X}", opcode, (0xFF00 + mmu.read_byte(self.program_counter + 1) as u16), self.read_register_a());

                let next_byte = mmu.read_byte(self.program_counter + 1) as u16;
                mmu.write_byte(0xFF00 + next_byte, self.read_register_a());
            },
            0xE1 => {
                trace!("{:#04X}: POP HL. HL:{:#06X} <- {:#06X} SP:{:#06X}", opcode, self.read_register_hl(), mmu.read_word(self.stack_pointer), self.stack_pointer);

                self.write_register_hl(mmu.read_word(self.stack_pointer));
                self.stack_pointer += 2;
            },
            0xE2 => {
                trace!("{:#04X}: LD ($FF00+C),A. $FF00+C:{:#06X} <- A:{:#04X}", opcode, (0xFF00 + self.read_register_c() as u16), self.read_register_a());

                mmu.write_byte(0xFF00 + self.read_register_c() as u16, self.read_register_a());
            },
            0xE5 => {
                trace!("{:#04X}: PUSH HL. SP:{:#06X} <- HL:{:#06X}", opcode, self.stack_pointer - 2, self.read_register_hl());

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.read_register_hl());
            },
            0xE7 => {
                trace!("{:#04X}: RST 0x0020", opcode);

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.program_counter);

                self.program_counter = 0x0020;
                increment_program_counter = false;
            },
            0xE6 => {
                trace!("{:#04X}: AND d8. A:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() & mmu.read_byte(self.program_counter + 1));

                self.and_with_register_a(mmu.read_byte(self.program_counter + 1));
            },
            0xE8 => {
                trace!("{:#04X}: ADD SP,d8. SP:{:#06X} + d8:{:#04X}", opcode, self.stack_pointer, mmu.read_byte(self.program_counter + 1));

                let result;
                let next_byte_signed: i8 = mmu.read_byte(self.program_counter + 1) as i8;
                if next_byte_signed < 0 {
                    result = self.stack_pointer - (next_byte_signed * -1) as u16;
                    self.stack_pointer = result;
                } else {
                    result = self.stack_pointer + next_byte_signed as u16;
                    self.stack_pointer = result;
                }

                self.unset_flag_bit(ZERO_BIT);
                self.unset_flag_bit(SUBTRACTION_BIT);

                // TODO - does this work?
                if ((self.stack_pointer ^ next_byte_signed as u16 ^ (result & 0xFFFF)) & 0x100) == 0x100 {
                    self.set_flag_bit(CARRY_BIT);
                } else {
                    self.unset_flag_bit(CARRY_BIT);
                }

                // TODO - does this work?
                if ((self.stack_pointer ^ next_byte_signed as u16 ^ (result & 0xFFFF)) & 0x10) == 0x10 {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }
            },
            0xE9 => {
                trace!("{:#04X}: JP HL. Jump to {:#06X}", opcode, self.read_register_hl());

                self.program_counter = self.read_register_hl();
                increment_program_counter = false;
            },
            0xEA => {
                trace!("{:#04X}: LD (a16),A. a16:{:#06X} <- A:{:#04X}", opcode, mmu.read_word(self.program_counter + 1), self.read_register_a());

                let next_word = mmu.read_word(self.program_counter + 1);
                mmu.write_byte(next_word, self.read_register_a());
            },
            0xEE => {
                trace!("{:#04X}: XOR d8. d8:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() ^ mmu.read_byte(self.program_counter + 1));

                self.xor_with_register_a(mmu.read_byte(self.program_counter + 1));
            },
            0xEF => {
                trace!("{:#04X}: RST 0x0028", opcode);

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.program_counter);

                self.program_counter = 0x0028;
                increment_program_counter = false;
            },
            0xF0 => {
                trace!("{:#04X}: LDH A,($FF00+a8). A:{:#04X} <- {:#04X}, Value of {:#06X}", opcode, self.read_register_a(), mmu.read_byte(self.program_counter + 1), (0xFF00 + mmu.read_byte(self.program_counter + 1) as u16));

                let next_byte = mmu.read_byte(self.program_counter + 1);
                self.write_register_a(mmu.read_byte(0xFF00 + next_byte as u16));
            },
            0xF1 => {
                trace!("{:#04X}: POP AF. AF:{:#06X} <- {:#06X} SP:{:#06X}", opcode, self.read_register_af(), mmu.read_word(self.stack_pointer), self.stack_pointer);

                self.write_register_af(mmu.read_word(self.stack_pointer));
                self.stack_pointer += 2;
            },
            0xF2 => {
                trace!("{:#04X}: LD A,($FF00+C). A:{:#04X} <- $FF00+C:{:#06X}", opcode, self.read_register_a(), (0xFF00 + self.read_register_c() as u16));

                let byte = mmu.read_byte(0xFF00 + self.read_register_c() as u16);
                self.write_register_a(byte);
            },
            0xF3 => {
                trace!("{:#04X}: Disable Interrupts.", opcode);

                self.interrupt_master_enable = false;
            },
            0xF5 => {
                trace!("{:#04X}: PUSH AF. SP:{:#06X} <- AF:{:#06X}", opcode, self.stack_pointer - 2, self.read_register_af());

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.read_register_af());
            },
            0xF6 => {
                trace!("{:#04X}: OR d8. d8:{:#04X} -> {:#04X}", opcode, self.read_register_a(), self.read_register_a() | mmu.read_byte(self.program_counter + 1));

                self.or_with_register_a(mmu.read_byte(self.program_counter + 1));
            },
            0xF7 => {
                trace!("{:#04X}: RST 0x0030", opcode);

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.program_counter);

                self.program_counter = 0x0030;
                increment_program_counter = false;
            },
            0xF8 =>{
                trace!("{:#04X}: LD HL,SP+r8. HL:{:#06X} <- SP+r8:{:#06X}", opcode, self.read_register_hl(), self.stack_pointer);

                self.unset_flag_bit(ZERO_BIT);
                self.unset_flag_bit(SUBTRACTION_BIT);

                let r8 = mmu.read_byte(self.program_counter) as u16;
                let result: u16 = self.stack_pointer + r8;

                if ((self.stack_pointer ^ r8 ^ result) & 0x100) == 0x100 {
                    self.set_flag_bit(CARRY_BIT);
                } else {
                    self.unset_flag_bit(CARRY_BIT);
                }

                if ((self.stack_pointer ^ r8 ^ result) & 0x10) == 0x10 {
                    self.set_flag_bit(HALF_CARRY_BIT);
                } else {
                    self.unset_flag_bit(HALF_CARRY_BIT);
                }

                self.write_register_hl(result);
            },
            0xF9 => {
                trace!("{:#04X}: LD SP,HL. SP:{:#06X} <- HL:{:#06X}", opcode, self.stack_pointer, self.read_register_hl());

                self.stack_pointer = self.read_register_hl();
            },
            0xFA => {
                trace!("{:#04X}: LD A,a16. A:{:#04X} <- a16:{:#06X}", opcode, self.read_register_a(), mmu.read_word(self.program_counter + 1));

                let next_word = mmu.read_word(self.program_counter + 1);
                self.write_register_a(mmu.read_byte(next_word));
            },
            0xFB => {
                trace!("{:#04X}: Enable Interrupts.", opcode);

                self.interrupt_master_enable = true;
            },
            0xFE => {
                trace!("{:#04X}: CP d8. A:{:#04X} d8:{:#04X}", opcode, self.read_register_a(), mmu.read_byte(self.program_counter + 1));

                self.compare_with_register_a(mmu.read_byte(self.program_counter + 1));
            },
            0xFF => {
                trace!("{:#04X}: RST 0x0038", opcode);

                self.stack_pointer -= 2;
                mmu.write_word(self.stack_pointer, self.program_counter);

                self.program_counter = 0x0038;
                increment_program_counter = false;
            },
            0xD3 | 0xDB | 0xDD | 0xE3 |
            0xE4 | 0xEB | 0xEC | 0xED |
            0xF4 | 0xFC | 0xFD        => {
                error!("Tried to call unused OpCode {}", opcode);
                exit(1);
            }
            _ => {
                error!("Unknown OpCode {:#04X}", opcode);
                exit(1);
            }
        }

        self.update_clock_and_program_counter(opcode, use_machine_cycles_branched, increment_program_counter);
    }

    fn process_cb_opcode(&mut self, mmu: &mut MMU) {
        let opcode = mmu.read_byte(self.program_counter + 1);

        match opcode {
            0x11 => {
                trace!("0xCB {:#04X}: RL C. C: {:#04X}", opcode, self.read_register_c());

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
            },
            0x7C => {
                trace!("0xCB {:#04X}: BIT 7,H. H:{:#04X}", opcode, self.read_register_h());

                // Test bit
                if self.most_significant_bit(self.read_register_h()) == 0 {
                    self.set_flag_bit(ZERO_BIT)
                } else {
                    self.unset_flag_bit(ZERO_BIT);
                }

                // Set flags
                self.set_flag_bit(HALF_CARRY_BIT);
                self.unset_flag_bit(SUBTRACTION_BIT);
            },
            _ => {
                error!("Unknown 0xCB OpCode {:#04X}", opcode);
                exit(1);
            }
        }

        self.update_clock_and_program_counter_for_cb_operations(opcode);
    }

    fn update_clock_and_program_counter(&mut self, opcode: u8, use_machine_cycles_branched: bool, increment_program_counter: bool) {
        if !use_machine_cycles_branched {
            self.clock.m = OPERATION_MACHINE_CYCLES[opcode as usize];
        } else {
            self.clock.m = OPERATION_MACHINE_CYCLES_BRANCHED[opcode as usize];
        }

        self.clock.t = self.clock.m * 4;

        if increment_program_counter {
            self.program_counter += OPERATION_BYTES[opcode as usize];
        }
    }

    fn update_clock_and_program_counter_for_cb_operations(&mut self, opcode: u8) {
//      let op_nibble_1 = (opcode & 0x00F0) >> 4;
        let op_nibble_2 = opcode & 0x000F;

        if op_nibble_2 == 0x6 || op_nibble_2 == 0xE {
            self.clock.m = 4;
        } else {
            self.clock.m = 2;
        }

        self.clock.t = self.clock.m * 4;

        self.program_counter += 2;
    }

    fn increase_register_u8(&mut self, value: u8) -> u8 {
        if value & 0xF == 0xF {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        let result = value + 1;

        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        self.unset_flag_bit(SUBTRACTION_BIT);

        return result;
    }

    fn increase_register_u16(&mut self, value: u16) -> u16 {
        if value & 0xF == 0xF {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        let result = value + 1;

        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        self.unset_flag_bit(SUBTRACTION_BIT);

        return result;
    }

    fn decrease_register_u8(&mut self, value: u8) -> u8 {
        self.set_flag_bit(SUBTRACTION_BIT);

        // TODO - Pretty sure this doesn't work right
        if (((value & 0xF) + (-1i8 & 0xF) as u8) & 0x10) == 0x10 {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        let result = value.wrapping_sub(1);

        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        return result;
    }

    fn decrease_register_u16(&mut self, value: u16) -> u16 {
        self.set_flag_bit(SUBTRACTION_BIT);

        // TODO - Pretty sure this doesn't work right
        if (((value & 0xF) + (-1i8 & 0xF) as u16) & 0x10) == 0x10 {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        let result = value.wrapping_sub(1);

        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        return result;
    }

    fn rotate_left(&mut self, value: u8) -> u8 {
        // Unset flag bits
        self.unset_flag_bit(SUBTRACTION_BIT);
        self.unset_flag_bit(HALF_CARRY_BIT);

        let mut result;
        if self.most_significant_bit(value) > 0 {
            self.set_flag_bit(CARRY_BIT);
            result = (value << 1);
            result |= 0x1;
        } else {
            self.unset_flag_bit(CARRY_BIT);
            result = (value << 1);
        }

        // Set zero bit
        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        return result;
    }

    fn rotate_left_through_carry(&mut self, value: u8) -> u8 {
        // Store carry flag
        let carry = self.read_flag(CARRY_BIT);

        // Unset flag bits
        self.unset_flag_bit(SUBTRACTION_BIT);
        self.unset_flag_bit(HALF_CARRY_BIT);

        if self.most_significant_bit(value) > 0 {
            self.set_flag_bit(CARRY_BIT);
        } else {
            self.unset_flag_bit(CARRY_BIT);
        }

        let result = (value << 1) | carry;

        // Set zero bit
        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        return result;
    }

    fn rotate_right(&mut self, value: u8) -> u8 {
        // Unset flag bits
        self.unset_flag_bit(SUBTRACTION_BIT);
        self.unset_flag_bit(HALF_CARRY_BIT);

        let mut result;
        if self.least_significant_bit(value) > 0 {
            self.set_flag_bit(CARRY_BIT);
            result = (value >> 1);
            result |= 0x80;
        } else {
            self.unset_flag_bit(CARRY_BIT);
            result = (value >> 1);
        }

        // Set zero bit
        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        return result;
    }

    fn rotate_right_through_carry(&mut self, value: u8) -> u8 {
        // Store carry flag
        let carry;
        if self.read_flag(CARRY_BIT) == 1 {
            carry = 0x80;
        } else {
            carry = 0x00;
        }

        // Unset flag bits
        self.unset_flag_bit(SUBTRACTION_BIT);
        self.unset_flag_bit(HALF_CARRY_BIT);

        if self.least_significant_bit(value) > 0 {
            self.set_flag_bit(CARRY_BIT);
        } else {
            self.unset_flag_bit(CARRY_BIT);
        }

        let result = (value >> 1) | carry;

        // Set zero bit
        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        return result;
    }

    fn add_u8_to_a(&mut self, value: u8) {
        let a = self.read_register_a();

        self.unset_flag_bit(SUBTRACTION_BIT);

        if a as u16 + value as u16 > 0xFF {
            self.set_flag_bit(CARRY_BIT);
        } else {
            self.unset_flag_bit(CARRY_BIT);
        }

        // TODO - Does this work?
        if ((a & 0xF) + value & 0xF) > 0xF {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        let result = a.wrapping_add(value);

        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        self.write_register_a(result);
    }

    fn add_u8_and_carry_to_a(&mut self, value: u8) {
        let a = self.read_register_a();
        let carry = self.read_flag(CARRY_BIT);
        let value_carry = value + carry;
        let (result, overflow) = a.overflowing_add(value_carry);

        self.unset_flag_bit(SUBTRACTION_BIT);

        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        if overflow {
            self.set_flag_bit(CARRY_BIT);
        } else {
            self.unset_flag_bit(CARRY_BIT);
        }

        //TODO - Does this work right?
        if result > 0xF {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        self.write_register_a(result);
    }

    fn add_u16_to_hl(&mut self, value: u16) {
        let hl = self.read_register_hl();

        self.unset_flag_bit(SUBTRACTION_BIT);

        if (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        let (result, overflow) = hl.overflowing_add(value);
        if overflow {
            self.set_flag_bit(CARRY_BIT);
        } else {
            self.unset_flag_bit(CARRY_BIT);
        }

        self.write_register_hl(result);
    }

    fn subtract_u8_from_a(&mut self, value: u8) {
        let a = self.read_register_a();

        self.set_flag_bit(SUBTRACTION_BIT);

        if value > a {
            self.set_flag_bit(CARRY_BIT);
        } else {
            self.unset_flag_bit(CARRY_BIT);
        }

        // TODO - Does this work right???
        if (a - value) & 0xF > a & 0xF {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        let result = a - value;

        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        self.write_register_a(result);
    }

    fn subtract_u8_and_carry_from_a(&mut self, value: u8) {
        let a = self.read_register_a();
        let carry = self.read_flag(CARRY_BIT);
        let value_carry: u8 = value + carry;
        let (result, overflow) = a.overflowing_sub(value_carry);

        if overflow {
            self.set_flag_bit(CARRY_BIT);
        } else {
            self.unset_flag_bit(CARRY_BIT);
        }

        //TODO - Does this work right?
        if (result & 0xF) > (a & 0xF) {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
        }

        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        self.set_flag_bit(SUBTRACTION_BIT);

        self.write_register_a(result);
    }

    fn and_with_register_a(&mut self, value: u8) {
        let result = self.read_register_a() & value;

        // Set flags
        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        self.unset_flag_bit(SUBTRACTION_BIT);
        self.unset_flag_bit(CARRY_BIT);

        self.set_flag_bit(HALF_CARRY_BIT);

        self.write_register_a(result);
    }

    fn or_with_register_a(&mut self, value: u8) {
        let result = self.read_register_a() | value;

        // Set flags
        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }
        self.unset_flag_bit(SUBTRACTION_BIT);
        self.unset_flag_bit(HALF_CARRY_BIT);
        self.unset_flag_bit(CARRY_BIT);

        self.write_register_a(result);
    }

    fn xor_with_register_a(&mut self, value: u8) {
        let result = self.read_register_a() ^ value;

        // Set flags
        if result == 0 {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }
        self.unset_flag_bit(SUBTRACTION_BIT);
        self.unset_flag_bit(HALF_CARRY_BIT);
        self.unset_flag_bit(CARRY_BIT);

        self.write_register_a(result);
    }

    fn compare_with_register_a(&mut self, value: u8) {
        let a = self.read_register_a();

        self.set_flag_bit(SUBTRACTION_BIT);

        if a == value {
            self.set_flag_bit(ZERO_BIT);
        } else {
            self.unset_flag_bit(ZERO_BIT);
        }

        if a < value {
            self.set_flag_bit(CARRY_BIT);
        } else {
            self.unset_flag_bit(CARRY_BIT);
        }

        // TODO - Does this work?
        if ((a.wrapping_sub(value)) & 0xF) > (a & 0xF) {
            self.set_flag_bit(HALF_CARRY_BIT);
        } else {
            self.unset_flag_bit(HALF_CARRY_BIT);
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

    fn least_significant_bit(&mut self, x: u8) -> u8 {
        return 0x01 & x;
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