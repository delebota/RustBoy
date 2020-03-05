// Cartridge Types
pub const ROM_ONLY: u8            = 0x00;
pub const MBC1: u8                = 0x01;
pub const MBC1_RAM: u8            = 0x02;
pub const MBC1_RAM_BATT: u8       = 0x03;
pub const MBC2: u8                = 0x05;
pub const MBC2_BATT: u8           = 0x06;
//  const RAM: u8                 = 0x08;
//  const RAM_BATT: u8            = 0x09;
//  const MMM01: u8               = 0x0B;
//  const MMM01_SRAM: u8          = 0x0C;
//  const MMM01_SRAM_BATT: u8     = 0x0D;
pub const MBC3_TIMER_BATT: u8     = 0x0F;
pub const MBC3_TIMER_RAM_BATT: u8 = 0x10;
pub const MBC3: u8                = 0x11;
pub const MBC3_RAM: u8            = 0x12;
pub const MBC3_RAM_BATT: u8       = 0x13;
pub const MBC5: u8                = 0x19;
pub const MBC5_BATT: u8           = 0x1A;
pub const MBC5_RAM_BATT: u8       = 0x1B;
pub const MBC5_RUMBLE: u8         = 0x1C;
pub const MBC5_RUMBLE_SRAM: u8    = 0x1D;
pub const MBC5_RUMB_SRAM_BATT: u8 = 0x1E;
//  const POCKET_CAMERA: u8       = 0x1F;
//  const BANDAI_TAMA5: u8        = 0xFD;
//  const HUDSON_HUC_3: u8        = 0xFE;
//  const HUDSON_HUC_1: u8        = 0xFF;

pub struct Cartridge {
    pub title: String,
    pub gameboy_type: u8,       // Type of gameboy; 0x80 = CGB, 0x00 = Other/Not CGB
    pub is_super_gameboy: u8,   // Super Gameboy functionality; 0x00 = Gameboy, 0x03 = SGB
    pub cartridge_type: u8,
    pub rom_size: u8,           // Size of the ROM (How many banks)
    pub ram_size: u8,           // Size of the RAM (How many banks)
    pub region: u8,             // Region, 0 = Japanese, 1 = Non-Japanese
    pub licensee: u8,           // Licensee code, 0x33 = Check 0x144, 0x79 = Accolade, 0xA4 = Konami (SGB Won't work if != 0x33)
    pub version: u8,
    pub checksum: u16
}

impl Cartridge {
    pub fn new() -> Cartridge {
        debug!("Initializing Cartridge");

        let title = String::from("                ");
        let gameboy_type = 0x00;
        let is_super_gameboy = 0x00;
        let cartridge_type = ROM_ONLY;
        let rom_size = 0x00;
        let ram_size = 0x00;
        let region = 0x00;
        let licensee = 0x33;
        let version = 0x00;
        let checksum = 0x0000;

        Cartridge {
            title,
            gameboy_type,
            is_super_gameboy,
            cartridge_type,
            rom_size,
            ram_size,
            region,
            licensee,
            version,
            checksum
        }
    }

    pub fn print_cartridge(&mut self) {
        debug!("Cartridge Data");
        debug!("Title: '{}'", self.title);
        debug!("GB Type: {:#04X}", self.gameboy_type);
        debug!("Is SGB: {:#04X}", self.is_super_gameboy);
        debug!("Cart Type: {:#04X}", self.cartridge_type);
        debug!("ROM Size: {:#04X}", self.rom_size);
        debug!("RAM Size: {:#04X}", self.ram_size);
        debug!("Region: {:#04X}", self.region);
        debug!("Licensee: {:#04X}", self.licensee);
        debug!("Version: {:#04X}", self.version);
        debug!("CheckSum: {:#06X}", self.checksum);
    }
}