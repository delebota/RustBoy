//TODO
//#define SET_INT_VBLANK(x)   intflags = ((intflags & 0xFE) | x) //Set bit 0
//#define SET_INT_LCDSTAT(x)  intflags = ((intflags & 0xFD) | (x << 1)) //Set bit 1
//#define SET_INT_TIMER(x)    intflags = ((intflags & 0xFB) | (x << 2))
//#define SET_INT_SERIAL(x)   intflags = ((intflags & 0xF7) | (x << 3))
//#define SET_INT_JOYPAD(x)   intflags = ((intflags & 0xEF) | (x << 4))

use std::fs::File;
use std::io;
use std::io::Read;
use std::process::exit;

use crate::gpu::GPU;

pub struct MMU {
    pub gpu: GPU,
    bios: [u8; 256],
    rom_banks: Box<[Vec<u8>]>,     // 16k ROM Banks,   0x0000 - 0x7FFF , ROM Bank 0 + switchable ROM bank
//  vram: [u8; 8192],              // 8k Video RAM,    0x8000 - 0x9FFF , Video RAM, stored in GPU
    eram: [u8; 8192],              // 8k External RAM, 0xA000 - 0xBFFF , switchable RAM bank
    wram: [u8; 8192],              // 8k Working RAM,  0xC000 - 0xDFFF , internal RAM
                                   // 8k Working RAM,  0xE000 - 0xFDFF , copy of internal RAM
    oam:  [u8;  160],              // Object Attr Mem, 0xFE00 - 0xFE9F
                                   // Empty            0xFEA0 - 0xFEFF
    io_ports: [u8; 64],            // I/O Ports        0xFF00 - 0xFF3F , I/O Ports
                                   // Empty            0xFF40 - 0xFF7F , GPU Registers
    zram: [u8;  127],              // Zero Page RAM,   0xFF80 - 0xFFFE
    interrupt_enable_register: u8, // Int Enable Reg,  0xFFFF

    active_rom_bank: u8,
    pub is_bios_mapped: bool
}

impl MMU {
    pub fn new() -> MMU {
        debug!("Initializing MMU");

        let gpu = GPU::new();
        let bios = [0; 256];
        let rom_banks = vec![vec![0u8; 16384]; 128].into_boxed_slice();
        let eram = [0; 8192];
        let wram = [0; 8192];
        let oam = [0; 160];
        let io_ports = [0; 64];
        let zram = [0; 127];
        let interrupt_enable_register = 0;
        let active_rom_bank = 1;
        let is_bios_mapped = false;

        MMU {
            gpu,
            bios,
            rom_banks,
            eram,
            wram,
            oam,
            io_ports,
            zram,
            interrupt_enable_register,
            active_rom_bank,
            is_bios_mapped
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
        let file_size = file.read_to_end(&mut buffer)?;

        // TODO - This is setup for tetris, loop should be based on total filesize/banks
        for rom_bank in 0..0x2 {
            for i in 0..0x4000 {
                self.rom_banks[rom_bank][i] = buffer[i + (rom_bank * 0x4000)];
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

            match addr_nibble_1 {
                0x0 | 0x1 | 0x2 | 0x3 => { // ROM Bank 0
                    return self.rom_banks[0][address as usize];
                },
                0x4 | 0x5 | 0x6 | 0x7 => { // Switchable ROM Bank
                    return self.rom_banks[self.active_rom_bank as usize][(address - 0x4000) as usize];
                },
                0x8 | 0x9 => { // Video RAM
                    return self.gpu.read_byte(address);
                },
                0xA | 0xB => { // External RAM (switchable RAM bank)
                    return self.eram[(address - 0xA000) as usize];
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
                                return self.oam[(address & 0xFF) as usize];
                            } else {
                                warn!("Tried to read a byte from unused address {:#X}, returned 0", address);
                                return 0;
                            }
                        },
                        0xF => { // I/O Ports, Zero Page RAM, Int Enable Register
                            if address == 0xFFFF { // Int Enable Register
                                return self.interrupt_enable_register;
                            }

                            match addr_nibble_3 {
                                0x0 | 0x1 | 0x2 | 0x3 => { // I/O Ports
                                    return self.io_ports[(address - 0xFF00) as usize];
                                },
                                0x4 | 0x5 | 0x6 | 0x7 => { // GPU Registers
                                    return self.gpu.read_register(address);
                                },
                                0x8 | 0x9 | 0xA | 0xB |
                                0xC | 0xD | 0xE | 0xF => { // Zero Page RAM
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

            match addr_nibble_1 {
                0x0 | 0x1 | 0x2 | 0x3 => { // ROM Bank 0
                    warn!("Tried to write to ROM Bank 0");
                    return;
                },
                0x4 | 0x5 | 0x6 | 0x7 => { // Switchable ROM Bank
                    warn!("Tried to write to Switchable ROM Bank");
                    return;
                },
                0x8 | 0x9 => { // Video RAM
                    self.gpu.write_byte(address, value);
                    return;
                },
                0xA | 0xB => { // External RAM (switchable RAM bank)
                    self.eram[(address - 0xA000) as usize] = value;
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
                                self.oam[(address & 0xFF) as usize] = value;
                                return;
                            } else {
                                warn!("Tried to write to an unused address {:#X}", address);
                                return;
                            }
                        },
                        0xF => { // I/O Ports, Zero Page RAM, Int Enable Register
                            if address == 0xFFFF { // Int Enable Register
                                self.interrupt_enable_register = value;
                                return;
                            }

                            match addr_nibble_3 {
                                0x0 | 0x1 | 0x2 | 0x3 => { // I/O Ports
                                    self.io_ports[(address - 0xFF00) as usize] = value;
                                    return;
                                },
                                0x4 | 0x5 | 0x6 | 0x7 => { // GPU Registers
                                    if address == 0xFF50 && value == 0x01 {
                                        trace!("BIOS has finished running");
                                        self.is_bios_mapped = false;
                                    } else {
                                        self.gpu.write_register(address, value);
                                    }
                                    return;
                                },
                                0x8 | 0x9 | 0xA | 0xB |
                                0xC | 0xD | 0xE | 0xF => { // Zero Page RAM
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
}