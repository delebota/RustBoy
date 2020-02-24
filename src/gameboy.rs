use std::process::exit;

use crate::cartridge::Cartridge;
use crate::cpu::{CPU, VBLANK_INTERRUPT_BIT, LCD_INTERRUPT_BIT, TIMER_INTERRUPT_BIT, SERIAL_INTERRUPT_BIT, JOYPAD_INTERRUPT_BIT};
use crate::mmu::MMU;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct GameBoy {
    pub cpu: CPU,
    pub mmu: MMU,
    pub cartridge: Cartridge,
    pub is_paused: bool
}

impl GameBoy {
    pub fn run(&mut self) {
        debug!("GameBoy running");

        if !self.cpu.skip_bios {
            self.cpu.program_counter = 0x0;
        } else {
            self.emulate_bios_setup();
            self.cpu.program_counter = 0x100;
        }

        loop {
            if !self.is_paused {
                for event in self.mmu.gpu.event_pump.poll_iter() {
                    match event {
                        Event::Quit    { timestamp } => exit(2),
                        Event::KeyDown { keycode: Some(Keycode::Escape), ..} => exit(2),

                        Event::KeyDown { keycode: Some(Keycode::Right), ..} => {  self.mmu.gpu.input.keys[1] &= 0xE},
                        Event::KeyDown { keycode: Some(Keycode::Left), ..} => {   self.mmu.gpu.input.keys[1] &= 0xD},
                        Event::KeyDown { keycode: Some(Keycode::Up), ..} => {     self.mmu.gpu.input.keys[1] &= 0xB},
                        Event::KeyDown { keycode: Some(Keycode::Down), ..} => {   self.mmu.gpu.input.keys[1] &= 0x7},
                        Event::KeyDown { keycode: Some(Keycode::Z), ..} => {      self.mmu.gpu.input.keys[0] &= 0xE},
                        Event::KeyDown { keycode: Some(Keycode::X), ..} => {      self.mmu.gpu.input.keys[0] &= 0xD},
                        Event::KeyDown { keycode: Some(Keycode::Space), ..} => {  self.mmu.gpu.input.keys[0] &= 0xB},
                        Event::KeyDown { keycode: Some(Keycode::KpEnter), ..} => {self.mmu.gpu.input.keys[0] &= 0x7},

                        Event::KeyUp   { keycode: Some(Keycode::Right), ..} => {  self.mmu.gpu.input.keys[1] |= 0x1},
                        Event::KeyUp   { keycode: Some(Keycode::Left), ..} => {   self.mmu.gpu.input.keys[1] |= 0x2},
                        Event::KeyUp   { keycode: Some(Keycode::Up), ..} => {     self.mmu.gpu.input.keys[1] |= 0x4},
                        Event::KeyUp   { keycode: Some(Keycode::Down), ..} => {   self.mmu.gpu.input.keys[1] |= 0x8},
                        Event::KeyUp   { keycode: Some(Keycode::Z), ..} => {      self.mmu.gpu.input.keys[0] |= 0x1},
                        Event::KeyUp   { keycode: Some(Keycode::X), ..} => {      self.mmu.gpu.input.keys[0] |= 0x2},
                        Event::KeyUp   { keycode: Some(Keycode::Space), ..} => {  self.mmu.gpu.input.keys[0] |= 0x4},
                        Event::KeyUp   { keycode: Some(Keycode::KpEnter), ..} => {self.mmu.gpu.input.keys[0] |= 0x8},
                        _ => {}
                    }
                }

                self.cpu.tick(&mut self.mmu);
                self.mmu.gpu.tick(self.cpu.get_clock_m());

                // Interrupts - Check if master enable is on, check if any interrupts are enabled, check if any interrupts have been fired (0xFF0F)
                let interrupt_enable_register = self.mmu.interrupt_enable_register;
                let interrupt_flags_register = self.mmu.read_byte(0xFF0F);
                if self.cpu.interrupt_master_enable && interrupt_enable_register > 0 && interrupt_flags_register > 0 {
                    // Vertical Blank Interrupt
                    if interrupt_enable_register & VBLANK_INTERRUPT_BIT == 1 && interrupt_flags_register & VBLANK_INTERRUPT_BIT == 1 {
                        self.mmu.write_byte(0xFF0F, interrupt_flags_register & (255 - VBLANK_INTERRUPT_BIT));
                        self.cpu.trigger_interrupt(&mut self.mmu, VBLANK_INTERRUPT_BIT);
                    }

                    // LCD Interrupt
                    if interrupt_enable_register & LCD_INTERRUPT_BIT == 2 && interrupt_flags_register & LCD_INTERRUPT_BIT == 2 {
                        self.mmu.write_byte(0xFF0F, interrupt_flags_register & (255 - LCD_INTERRUPT_BIT));
                        self.cpu.trigger_interrupt(&mut self.mmu, LCD_INTERRUPT_BIT);
                    }

                    // Timer Interrupt
                    if interrupt_enable_register & TIMER_INTERRUPT_BIT == 4 && interrupt_flags_register & TIMER_INTERRUPT_BIT == 4 {
                        self.mmu.write_byte(0xFF0F, interrupt_flags_register & (255 - TIMER_INTERRUPT_BIT));
                        self.cpu.trigger_interrupt(&mut self.mmu, TIMER_INTERRUPT_BIT);
                    }

                    // Serial Interrupt
                    if interrupt_enable_register & SERIAL_INTERRUPT_BIT == 8 && interrupt_flags_register & SERIAL_INTERRUPT_BIT == 8 {
                        self.mmu.write_byte(0xFF0F, interrupt_flags_register & (255 - SERIAL_INTERRUPT_BIT));
                        self.cpu.trigger_interrupt(&mut self.mmu, SERIAL_INTERRUPT_BIT);
                    }

                    // Joypad Interrupt
                    if interrupt_enable_register & JOYPAD_INTERRUPT_BIT == 16 && interrupt_flags_register & JOYPAD_INTERRUPT_BIT == 16 {
                        self.mmu.write_byte(0xFF0F, interrupt_flags_register & (255 - JOYPAD_INTERRUPT_BIT));
                        self.cpu.trigger_interrupt(&mut self.mmu, JOYPAD_INTERRUPT_BIT);
                    }
                }
            }

            // TODO - proper processing speed
            // microseconds not millis
            // thread::sleep(time::Duration::from_millis(10));
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        // Load ROM from disk
        let result = self.mmu.load_rom(rom_path);
        if result.is_err() {
            error!("Failed to load ROM");
            error!("Error: {:?}", result.err());
            exit(1);
        }

        // Setup Cartridge
        debug!("Setting up Cartridge Data");
        //TODO
//        let mut title: Vec<u8> = vec![0; 16];
//        for i in 0..16 {
//            // Read Title
//            title[i] = self.mmu.read_byte((0x134 + i) as u16);
//        }
//        self.cartridge.title = String::from_utf8(title).unwrap();
        self.cartridge.gameboy_type = self.mmu.read_byte(0x143);
        self.cartridge.is_super_gameboy = self.mmu.read_byte(0x146);
        self.cartridge.cartridge_type = self.mmu.read_byte(0x147);
        self.cartridge.rom_size = self.mmu.read_byte(0x148);
        self.cartridge.ram_size = self.mmu.read_byte(0x149);
        self.cartridge.region = self.mmu.read_byte(0x14A);
        self.cartridge.licensee = self.mmu.read_byte(0x14B);
        self.cartridge.version = self.mmu.read_byte(0x14C);
        self.cartridge.checksum = self.mmu.read_word(0x14E);
        self.cartridge.print_cartridge();

        self.mmu.set_cartridge_type(self.cartridge.cartridge_type);
    }

    pub fn skip_bios(&mut self, skip: bool) {
        self.cpu.skip_bios = skip;
    }

    fn emulate_bios_setup(&mut self) {
        self.cpu.write_register_af(0x01);
        self.cpu.write_register_f(0xB0);
        self.cpu.write_register_bc(0x0013);
        self.cpu.write_register_de(0x00D8);
        self.cpu.write_register_hl(0x014D);
        self.cpu.stack_pointer = 0xFFFE;
        self.mmu.write_byte(0xFF00, 0x0F);
        self.mmu.write_byte(0xFF05, 0x00);
        self.mmu.write_byte(0xFF06, 0x00);
        self.mmu.write_byte(0xFF07, 0x00);
        self.mmu.write_byte(0xFF10, 0x80);
        self.mmu.write_byte(0xFF11, 0xBF);
        self.mmu.write_byte(0xFF12, 0xF3);
        self.mmu.write_byte(0xFF14, 0xBF);
        self.mmu.write_byte(0xFF16, 0x3F);
        self.mmu.write_byte(0xFF17, 0x00);
        self.mmu.write_byte(0xFF19, 0xBF);
        self.mmu.write_byte(0xFF1A, 0x7F);
        self.mmu.write_byte(0xFF1B, 0xFF);
        self.mmu.write_byte(0xFF1C, 0x9F);
        self.mmu.write_byte(0xFF1E, 0xBF);
        self.mmu.write_byte(0xFF20, 0xFF);
        self.mmu.write_byte(0xFF21, 0x00);
        self.mmu.write_byte(0xFF22, 0x00);
        self.mmu.write_byte(0xFF23, 0xBF);
        self.mmu.write_byte(0xFF24, 0x77);
        self.mmu.write_byte(0xFF25, 0xF3);
        self.mmu.write_byte(0xFF26, 0xF1);
        self.mmu.write_byte(0xFF40, 0x91);
        self.mmu.write_byte(0xFF42, 0x00);
        self.mmu.write_byte(0xFF43, 0x00);
        self.mmu.write_byte(0xFF45, 0x00);
        self.mmu.write_byte(0xFF47, 0xFC);
        self.mmu.write_byte(0xFF48, 0xFF);
        self.mmu.write_byte(0xFF49, 0xFF);
        self.mmu.write_byte(0xFF4A, 0x00);
        self.mmu.write_byte(0xFF4B, 0x00);
        self.mmu.write_byte(0xFFFF, 0x00);
    }
}