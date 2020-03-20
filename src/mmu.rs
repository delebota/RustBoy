use std::fs::File;
use std::io;
use std::io::Read;
use std::process::exit;

use crate::cartridge::{ROM_ONLY, MBC1, MBC1_RAM, MBC1_RAM_BATT};
use crate::gpu::GPU;
use crate::timer::Timer;

pub struct MMU {
    pub gpu: GPU,
    bios: [u8; 256],
    rom_banks: Box<[Vec<u8>]>,          // 16k ROM Banks,   0x0000 - 0x7FFF , ROM Bank 0 + switchable ROM bank
//  vram: [u8; 8192],                   // 8k Video RAM,    0x8000 - 0x9FFF , Video RAM, stored in GPU
    eram: [u8; 8192],                   // 8k External RAM, 0xA000 - 0xBFFF , switchable RAM bank
    wram: [u8; 8192],                   // 8k Working RAM,  0xC000 - 0xDFFF , internal RAM
                                        // 8k Working RAM,  0xE000 - 0xFDFF , copy of internal RAM
//  oam:  [u8;  160],                   // Object Attr Mem, 0xFE00 - 0xFE9F , Sprites, stored in GPU
                                        // Empty            0xFEA0 - 0xFEFF
    io_ports: [u8; 64],                 // I/O Ports        0xFF00 - 0xFF3F , I/O Ports
                                        // Empty            0xFF40 - 0xFF7F , GPU Registers
    zram: [u8;  127],                   // Zero Page RAM,   0xFF80 - 0xFFFE
    pub interrupt_enable_register: u8,  // Int Enable Reg,  0xFFFF          , Interrupt Enable/Disable Register

    pub timer: Timer,

    active_rom_bank: u8,
    active_ram_bank: u8,
    active_external_ram: bool,
    rom_size: u8,
    memory_mode: u8,
    cartridge_type: u8,
    pub is_bios_mapped: bool
}

impl MMU {
    pub fn new() -> Self {
        debug!("Initializing MMU");

        MMU {
            gpu: GPU::new(),
            bios: [0; 256],
            rom_banks: vec![vec![0u8; 16384]; 128].into_boxed_slice(),
            eram: [0; 8192],
            wram: [0; 8192],
            io_ports: [0; 64],
            zram: [0; 127],
            interrupt_enable_register: 0,
            timer: Timer::new(),
            active_rom_bank: 1,
            active_ram_bank: 0,
            active_external_ram: false,
            rom_size: 0,
            memory_mode: 0,
            cartridge_type: ROM_ONLY,
            is_bios_mapped: false
        }
    }

    pub fn load_bios(&mut self, path: &str) -> io::Result<()> {
        debug!("Loading BIOS from {}", path);

        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        let file_size = file.read_to_end(&mut buffer)?;

        for i in 0..file_size {
            self.bios[i] = buffer[i];
        }

        Ok(())
    }

    pub fn load_rom(&mut self, path: &str) -> io::Result<()>  {
        debug!("Loading ROM: {}", path);

        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        self.set_rom_size(buffer[0x148]);

        for rom_banks in 0..self.rom_size {
            for i in 0..0x4000 {
                self.rom_banks[rom_banks as usize][i as usize] = buffer[i + (rom_banks as u32 * 0x4000) as usize];
            }
        }

        Ok(())
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        if self.is_bios_mapped && address <= 0xFF {
            return self.bios[address as usize];
        } else {
            // break up into nibbles
            let addr_nibble_1 = (address & 0xF000) >> 12;
            let addr_nibble_2 = (address & 0x0F00) >> 8;
            let addr_nibble_3 = (address & 0x00F0) >> 4;
            let addr_nibble_4 =  address & 0x000F;

            match addr_nibble_1 {
                0x0 | 0x1 | 0x2 | 0x3 => { // ROM Bank 0
                    return self.rom_banks[0][address as usize];
                },
                0x4 | 0x5 | 0x6 | 0x7 => { // Switchable ROM Bank
                    return self.rom_banks[self.active_rom_bank as usize][(address - 0x4000) as usize];
                },
                0x8 | 0x9 => { // Video RAM
                    return self.gpu.read_vram(address);
                },
                0xA | 0xB => { // External RAM (switchable RAM bank)
                    if self.active_external_ram {
                        return self.eram[(address - 0xA000) as usize];
                    } else {
                        return 0;
                    }
                },
                0xC | 0xD => { // Working RAM (internal RAM)
                    return self.wram[(address - 0xC000) as usize];
                },
                0xE => { // Working RAM copy (internal RAM copy)
                    return self.wram[(address - 0xE000) as usize];
                },
                0xF => { // Working RAM copy, Object Attr Memory, I/O Ports, Zero Page RAM, Int Enable Register
                    match addr_nibble_2 {
                        0x0 | 0x1 | 0x2 | 0x3 | // Working RAM Copy
                        0x4 | 0x5 | 0x6 | 0x7 |
                        0x8 | 0x9 | 0xA | 0xB |
                        0xC | 0xD => {
                            return self.wram[(address & 0x1FFF) as usize];
                        },
                        0xE => { // Object Attr Memory
                            if address < 0xFEA0 {
                                return self.gpu.read_oam((address & 0xFF) as u8);
                            } else {
                                warn!("Tried to read a byte from unused address {:#X}, returned 0", address);
                                return 0;
                            }
                        },
                        0xF => { // I/O Ports, Zero Page RAM, Int Enable Register
                            match (addr_nibble_3, addr_nibble_4) {
                                // TODO - Clean this up...
                                (0x0, 0x0) => { // Joypad
                                    return self.gpu.input.read() | 0xC0; // or'd with C0 because Bit 6/7 are unmapped, unmapped bits always return as 1
                                },
                                (0x0, 0x1) => { // Serial Bus
                                    //TODO - Serial Bus
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x0, 0x2) => { // Serial Control
                                    //TODO - Serial Control
                                    return self.io_ports[(address - 0xFF00) as usize] | 0x7E; // or'd with 7E because Bits 1-6 are unmapped, unmapped bits always return as 1
                                },
                                (0x0, 0x4) => { // Timer - DIV
                                    return (self.timer.div >> 8) as u8;
                                },
                                (0x0, 0x5) => { // Timer - TIMA
                                    return self.timer.tima;
                                },
                                (0x0, 0x6) => { // Timer - TMA
                                    return self.timer.tma;
                                },
                                (0x0, 0x7) => { // Timer - TAC
                                    return self.timer.tac | 0xF8; // or'd with F8 because Bits 3-7 are unmapped, unmapped bits always return as 1
                                },
                                (0x0, 0xF) => {
                                    // TODO - Make Interrupt Flag Request a variable
                                    return self.io_ports[(address - 0xFF00) as usize] | 0xE0; // or'd with E0 because Bits 5-7 are unmapped, unmapped bits always return as 1
                                },
                                (0x1, 0x0) => { // APU - NR10
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize] | 0x80; // or'd with 80 because Bit 7 is unmapped, unmapped bits always return as 1
                                },
                                (0x1, 0x1) => { // APU - NR11
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0x2) => { // APU - NR12
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0x3) => { // APU - NR13
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0x4) => { // APU - NR14
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0x6) => { // APU - NR21
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0x7) => { // APU - NR22
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0x8) => { // APU - NR23
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0x9) => { // APU - NR24
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0xA) => { // APU - NR30
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize] | 0x7F; // or'd with 7F because Bits 0-6 are unmapped, unmapped bits always return as 1
                                },
                                (0x1, 0xB) => { // APU - NR31
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0xC) => { // APU - NR32
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize] | 0x9F; // or'd with 9F because Bits 0-4 and 7 are unmapped, unmapped bits always return as 1
                                },
                                (0x1, 0xD) => { // APU - NR33
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x1, 0xE) => { // APU - NR34
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x2, 0x0) => { // APU - NR41
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize] | 0xC0; // or'd with C0 because Bits 6/7 are unmapped, unmapped bits always return as 1
                                },
                                (0x2, 0x1) => { // APU - NR42
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x2, 0x2) => { // APU - NR43
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x2, 0x3) => { // APU - NR44
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize] | 0x3F; // or'd with 3F because Bits 0-5 are unmapped, unmapped bits always return as 1
                                },
                                (0x2, 0x4) => { // APU - NR50
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x2, 0x5) => { // APU - NR51
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x2, 0x6) => { // APU - NR52
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize] | 0x70; // or'd with  because Bits 4-6 are unmapped, unmapped bits always return as 1
                                },
                                (0x3, _) => { // APU - Wave Pattern RAM
                                    //TODO - APU
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                (0x4, 0x1) => { // STAT Register
                                    //TODO - STAT Register
                                    return self.gpu.read_register(address) | 0x80; // or'd with 80 because Bit 7 is unmapped, unmapped bits always return as 1
                                },
                                (0xF, 0xF) => { // Interrupt Enable Register
                                    return self.interrupt_enable_register;
                                },
                                (0x0, _) | (0x1, _) | (0x2, _)  => { // Unused I/O Ports, return 0xFF
                                    return 0xFF;
                                },
                                (0x4, _) | (0x5, _) | (0x6, _) | (0x7, _) => { // GPU Registers
                                    if address < 0xFF4C {
                                        return self.gpu.read_register(address);
                                    } else {
                                        return 0xFF;
                                    }
                                },
                                (0x8, _) | (0x9, _) | (0xA, _) | (0xB, _) |
                                (0xC, _) | (0xD, _) | (0xE, _) | (0xF, _) => { // Zero Page RAM
                                    return self.zram[(address & 0x7F) as usize];
                                },
                                _ => warn!("Tried to read a byte from unmapped address {:#X}", address)
                            }
                        },
                        _ => warn!("Tried to read a byte from unmapped address {:#X}", address)
                    }
                },
                _ => warn!("Tried to read a byte from unmapped address {:#X}", address)
            }
        }

        error!("Tried to read a byte from unmapped address {:#X}", address);
        exit(1);
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        return self.read_byte(address) as u16 | ((self.read_byte(address + 1) as u16) << 8);
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        if self.is_bios_mapped && address < 0xFF {
            warn!("Tried to overwrite BIOS ROM");
            return;
        } else {
            // break up into nibbles
            let addr_nibble_1 = (address & 0xF000) >> 12;
            let addr_nibble_2 = (address & 0x0F00) >> 8;
            let addr_nibble_3 = (address & 0x00F0) >> 4;
            let addr_nibble_4 =  address & 0x000F;

            match addr_nibble_1 {
                0x0 | 0x1 => {
                    trace!("Updating external RAM setting.");
                    self.update_external_ram(value);
                    return;
                },
                0x2 | 0x3 => {
                    trace!("Updating active ROM bank.");
                    self.update_active_rom_bank(value);
                    return;
                },
                0x4 | 0x5 => {
                    trace!("Updating ROM/RAM bank.");
                    self.update_active_rom_ram_bank(value);
                    return;
                },
                0x6 | 0x7 => {
                    trace!("Updating memory model.");
                    self.update_memory_mode(value);
                    return;
                },
                0x8 | 0x9 => { // Video RAM
                    self.gpu.write_vram(address, value);
                    return;
                },
                0xA | 0xB => { // External RAM (switchable RAM bank)
                    if self.active_external_ram {
                        self.eram[(address - 0xA000) as usize] = value;
                    }
                    return;
                },
                0xC | 0xD => { // Working RAM (internal RAM)
                    self.wram[(address - 0xC000) as usize] = value;
                    return;
                },
                0xE => { // Working RAM copy (internal RAM copy)
                    self.wram[(address - 0xE000) as usize] = value;
                    return;
                },
                0xF => { // Working RAM copy, Object Attr Memory, I/O Ports, Zero Page RAM, Int Enable Register
                    match addr_nibble_2 {
                        0x0 | 0x1 | 0x2 | 0x3 | // Working RAM Copy
                        0x4 | 0x5 | 0x6 | 0x7 |
                        0x8 | 0x9 | 0xA | 0xB |
                        0xC | 0xD => {
                            self.wram[(address & 0x1FFF) as usize] = value;
                            return;
                        },
                        0xE => { // Object Attr Memory
                            if address < 0xFEA0 {
                                self.gpu.write_oam((address & 0xFF) as u8, value);
                                self.gpu.build_object_data(address - 0xFE00, value);
                                return;
                            } else {
                                trace!("Tried to write to an unused address {:#X}", address);
                                return;
                            }
                        },
                        0xF => { // I/O Ports, Zero Page RAM, Int Enable Register
                            match (addr_nibble_3, addr_nibble_4) {
                                (0x0, 0x0) => { // Joypad
                                    self.gpu.input.write(value);
                                    return;
                                },
                                (0x0, 0x4) => { // Timer - DIV
                                    self.timer.div = 0;
                                    return;
                                },
                                (0x0, 0x5) => { // Timer - TIMA
                                    self.timer.tima = value;
                                    return;
                                },
                                (0x0, 0x6) => { // Timer - TMA
                                    self.timer.tma = value;
                                    return;
                                },
                                (0x0, 0x7) => { // Timer - TAC
                                    self.timer.tac = value & 0x7;
                                    self.timer.update();
                                    return;
                                },
                                (0xF, 0xF) => { // Interrupt Enable Register
                                    self.interrupt_enable_register = value;
                                    return;
                                },
                                (0x0, _) | (0x1, _) | (0x2, _) | (0x3, _) => { // I/O Ports
                                    self.io_ports[(address - 0xFF00) as usize] = value;
                                    return;
                                },
                                (0x4, _) | (0x5, _) | (0x6, _) | (0x7, _) => { // GPU Registers
                                    if address == 0xFF50 && value == 0x01 {
                                        debug!("BIOS has finished running");
                                        self.is_bios_mapped = false;
                                    } else {
                                        self.gpu.write_register(address, value);
                                    }
                                    return;
                                },
                                (0x8, _) | (0x9, _) | (0xA, _) | (0xB, _) |
                                (0xC, _) | (0xD, _) | (0xE, _)| (0xF, _) => { // Zero Page RAM
                                    self.zram[(address & 0x7F) as usize] = value;
                                    return;
                                },
                                _ => warn!("Tried to write to an unmapped address {:#X}", address)
                            }
                        },
                        _ => warn!("Tried to write to an unmapped address {:#X}", address)
                    }
                },
                _ => warn!("Tried to write to an unmapped address {:#X}", address)
            }
        }
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address + 1, (value >> 8) as u8);
    }

    pub fn set_cartridge_type(&mut self, value: u8) {
        self.cartridge_type = value;
    }

    pub fn set_rom_size(&mut self, value: u8) {
        match value {
            0 => self.rom_size = 2,
            1 => self.rom_size = 4,
            2 => self.rom_size = 8,
            3 => self.rom_size = 16,
            4 => self.rom_size = 32,
            5 => self.rom_size = 64,
            6 => self.rom_size = 128,
            _ => {error!("Unknown ROM size {}", value); exit(1);}
        }
    }

    fn update_external_ram(&mut self, value: u8) {
        // TODO - Finish MBC2,3,5 support
        match self.cartridge_type {
            ROM_ONLY | MBC1 => {
                trace!("Tried to update external RAM on a cartridge with no RAM.");
                return;
            },
            MBC1_RAM | MBC1_RAM_BATT => {
                //TODO - trace
                debug!("Updating external RAM - {:#04X}. 0x0A = enable, all else disable", value);

                if value == 0x0A {
                    self.active_external_ram = true;
                } else {
                    self.active_external_ram = false;
                }
            },
            _ => {
                //TODO - trace
                debug!("Tried to update external RAM on an unknown/unsupported cartridge type - Cart Type:{:#04X} - Value:{}", self.cartridge_type, value);
                exit(1);
            }
        }
    }

    fn update_active_rom_bank(&mut self, value: u8) {
        // TODO - Finish MBC2,3,5 support
        match self.cartridge_type {
            ROM_ONLY => {
                trace!("Tried to change active ROM bank on a ROM_ONLY cartridge.");
                return;
            },
            MBC1 | MBC1_RAM | MBC1_RAM_BATT => {
                //TODO - trace
                debug!("Updating ROM Bank {}", value);

                if value == 0 {
                    self.active_rom_bank = 1;
                } else if value < self.rom_size && value < 32 {
                    self.active_rom_bank = value;
                } else {
                    error!("Tried to assign invalid ROM bank {}, should be 0-31.", value);
                    exit(1);
                }
            },
            _ => {
                //TODO - trace
                debug!("Tried to change active ROM bank on an unknown/unsupported cartridge type - Cart Type:{:#04X} - Value:{}", self.cartridge_type, value);
                exit(1);
            }
        }
    }

    fn update_active_rom_ram_bank(&mut self, value: u8) {
        // TODO - Finish MBC2,3,5 support
        match self.cartridge_type {
            ROM_ONLY => {
                trace!("Tried to change active ROM/RAM bank on a ROM_ONLY cartridge.");
                return;
            },
            MBC1 | MBC1_RAM | MBC1_RAM_BATT => {
                if self.memory_mode == 0 {
                    // ROM Mode
                    trace!("Updating active ROM bank. {}", (self.active_rom_bank & 0x1F) + ((value & 3) << 5));
                    self.active_rom_bank = (self.active_rom_bank & 0x1F) + ((value & 3) << 5);
                } else if self.memory_mode == 1 {
                    // RAM Mode
                    trace!("Updating active RAM bank. {}", value);
                    self.active_ram_bank = value & 3;
                }
            },
            _ => {
                //TODO - trace
                debug!("Tried to change active ROM/RAM bank on an unknown/unsupported cartridge type - Cart Type:{:#04X} - Value:{}", self.cartridge_type, value);
                exit(1);
            }
        }
    }

    fn update_memory_mode(&mut self, value: u8) {
        // TODO - Finish MBC2,3,5 support
        match self.cartridge_type {
            ROM_ONLY | MBC1 => {
                trace!("Tried to change memory mode on a cartridge that only has ROM.");
                return;
            },
            MBC1_RAM | MBC1_RAM_BATT => {
                if value & 0x1 == 1 {
                    //TODO - trace
                    debug!("Memory Mode set to 1");
                    self.memory_mode = 1;
                } else {
                    //TODO - trace
                    debug!("Memory Mode set to 0");
                    self.memory_mode = 0;
                }
            },
            _ => {
                //TODO - trace
                debug!("Tried to change memory mode on an unknown/unsupported cartridge type - Cart Type:{:#04X} - Value:{}", self.cartridge_type, value);
                exit(1);
            }
        }
    }
}