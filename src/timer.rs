pub struct Timer {
    pub div: u16,
    pub tima: u8,
    tima_speed: u16,
    pub tma: u8,
    pub tac: u8,
    counter: u16,
    div_counter: u8,
    tima_overflow_last_step: bool
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tima: 0,
            tima_speed: 256,
            tma: 0,
            tac: 0,
            counter: 0,
            div_counter: 0,
            tima_overflow_last_step: false
        }
    }

    pub fn update(&mut self) {
        match self.tac & 0x3 {
            0 => {self.tima_speed = 256},
            1 => {self.tima_speed = 4},
            2 => {self.tima_speed = 16},
            3 => {self.tima_speed = 64},
            _ => {}
        }
    }

    pub fn step(&mut self, clock_t: u8) {
        self.div_counter += clock_t;
        if self.div_counter >= 16 {
            self.div_counter = 0;
            self.div = self.div.wrapping_add(1);
        }

        if self.tac & 0x4 != 0 {
            self.counter += clock_t as u16;
            if self.counter >= self.tima_speed {
                self.counter = 0;
                if self.tima_overflow_last_step {
                    self.tima = self.tma;
                } else if (self.tima as u16 + 1) & 0xFF == 0 {
                    //TODO - trace
                    debug!("Requesting Timer Interrupt");
                    //TODO - fire IFR for timer
                    self.tima_overflow_last_step = true;
                } else {
                    self.tima += 1;
                }
            }
        }
    }
}