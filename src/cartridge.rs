// Cartridge Types
  const ROM_ONLY: u8            = 0x00;
//const MBC1: u8                = 0x01;
//const MBC1_RAM: u8            = 0x02;
//const MBC1_RAM_BATT: u8       = 0x03;
//const MBC2: u8                = 0x05;
//const MBC2_BATT: u8           = 0x06;
//const RAM: u8                 = 0x08;
//const RAM_BATT: u8            = 0x09;
//const MMM01: u8               = 0x0B;
//const MMM01_SRAM: u8          = 0x0C;
//const MMM01_SRAM_BATT: u8     = 0x0D;
//const MBC3_TIMER_BATT: u8     = 0x0F;
//const MBC3_TIMER_RAM_BATT: u8 = 0x10;
//const MBC3: u8                = 0x11;
//const MBC3_RAM: u8            = 0x12;
//const MBC3_RAM_BATT: u8       = 0x13;
//const MBC5: u8                = 0x19;
//const MBC5_BATT: u8           = 0x1A;
//const MBC5_RAM_BATT: u8       = 0x1B;
//const MBC5_RUMBLE: u8         = 0x1C;
//const MBC5_RUMBLE_SRAM: u8    = 0x1D;
//const MBC5_RUMB_SRAM_BATT: u8 = 0x1E;
//const POCKET_CAMERA: u8       = 0x1F;
//const BANDAI_TAMA5: u8        = 0xFD;
//const HUDSON_HUC_3: u8        = 0xFE;
//const HUDSON_HUC_1: u8        = 0xFF;

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

    pub fn trace_cartridge(&mut self) {
        trace!("Cartridge Data");
        trace!("Title: '{}'", self.title);
        trace!("GB Type: {:#04X}", self.gameboy_type);
        trace!("Is SGB: {:#04X}", self.is_super_gameboy);
        trace!("Cart Type: {:#04X}", self.cartridge_type);
        trace!("ROM Size: {:#04X}", self.rom_size);
        trace!("RAM Size: {:#04X}", self.ram_size);
        trace!("Region: {:#04X}", self.region);
        trace!("Licensee: {:#04X}", self.licensee);
        trace!("Version: {:#04X}", self.version);
        trace!("CheckSum: {:#06X}", self.checksum);
    }
}