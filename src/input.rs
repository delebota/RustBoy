pub struct Input {
    pub column: u8,
    pub keys: [u8; 2],
}

impl Input {
    pub fn new() -> Input {
        debug!("Initializing Input");

        let column = 0;
        let keys = [0x0F, 2];

        Input {
            column,
            keys
        }
    }
}