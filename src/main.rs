#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate pretty_env_logger;

use clap::App;

use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::gameboy::GameBoy;
use crate::mmu::MMU;
use std::process::exit;

mod cartridge;
mod cpu;
mod gameboy;
mod gpu;
mod input;
mod mmu;

fn main() {
    // Command Line Arg Parser
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Logger Init
    pretty_env_logger::init();

    // Init GameBoy
    let mut gameboy = GameBoy {
        cpu: CPU::new(),
        mmu: MMU::new(),
        cartridge: Cartridge::new(),
        is_paused: false
    };

    // Parse args
    let bios_path = matches.value_of("bios").unwrap_or("");
    let rom_path= matches.value_of("rom").unwrap_or("");
    let vram_debug = matches.value_of("debug_vram").unwrap_or("false");
    if vram_debug.eq_ignore_ascii_case("true") {
        gameboy.mmu.gpu.vram_debug = true;
        gameboy.mmu.gpu.vram_debug_canvas.window_mut().show();
    }

    if bios_path.is_empty() && rom_path.is_empty() {
        error!("No ROM or BIOS was specified. You must provide at least one (Usually ROM or both). Use the --help flag for more information.");
        exit(1);
    }

    // Load BIOS if provided
    if !bios_path.is_empty() {
        let result = gameboy.mmu.load_bios(bios_path);
        if result.is_err() {
            warn!("Failed to load BIOS, skipping");
            error!("Error: {:?}", result.err());
            gameboy.skip_bios(true);
        } else if result.is_ok() {
            debug!("BIOS loaded successfully");
            gameboy.mmu.is_bios_mapped = true;
        }
    } else {
        gameboy.skip_bios(true);
    }

    // Load ROM if provided
    if !rom_path.is_empty() {
        gameboy.load_rom(rom_path);
    }

    gameboy.run();
}
