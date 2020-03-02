pub struct Input {
    pub column: u8,
    pub keys: [u8; 2],
}

impl Input {
    pub fn new() -> Input {
        debug!("Initializing Input");

        let column = 0;
        let keys = [0x0F; 2];

        Input {
            column,
            keys
        }
    }

    pub fn read(&self) -> u8 {
        match self.column {
            0x10 => {return self.keys[0]},
            0x20 => {return self.keys[1]},
            _    => {return 0}
        }
    }

    pub fn write(&mut self, value: u8) {
        self.column = value & 0x30;
    }
}