use std::process::exit;

use sdl2::{EventPump, Sdl};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

// GPU States
pub const STATE_HBLANK: u8    = 0;
pub const STATE_VBLANK: u8    = 1;
pub const STATE_OAM_READ: u8  = 2;
pub const STATE_VRAM_READ: u8 = 3;

pub struct GPU {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    pub vram_debug_canvas: Canvas<Window>,
    event_pump: EventPump,
    vram: [u8; 8192],
    tileset: Box<[Vec<Vec<u8>>]>,
    state: u8,
    state_clock: u32,
    lcd_control: u8,
    lcd_status: u8, // TODO - implement and use this
    scroll_y: u8,
    scroll_x: u8,
    palette: [u8; 4],
    sprite_palette_0: [u8; 4], //TODO - 0 is always transparent, may need seperate palette_ref
    sprite_palette_1: [u8; 4], //TODO - 0 is always transparent, may need seperate palette_ref
    window_y: u8,
    window_x: u8,
    render_line: u8,
    palette_reference: [Color; 4],
    lock_vram: bool,
    pub vram_debug: bool
}

impl GPU {
    pub fn new() -> GPU {
        debug!("Initializing GPU");

        let sdl_context = sdl2::init().unwrap();
        let canvas = sdl_context.video().unwrap()
            .window("RustBoy", 160, 144).position(800, 100).build().unwrap()
            .into_canvas().accelerated().build().unwrap();
        let vram_debug_canvas = sdl_context.video().unwrap()
            .window("VRAM", 256, 256).position(800, 300).hidden().build().unwrap()
            .into_canvas().accelerated().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        let vram = [0; 8192];
        let tileset = vec![vec![vec![0u8; 8]; 8]; 384].into_boxed_slice();
        let state = STATE_OAM_READ;
        let state_clock = 0;
        let lcd_control = 0;
        let lcd_status = 0;
        let scroll_y = 0;
        let scroll_x = 0;
        let palette = [0; 4];
        let sprite_palette_0 = [0; 4];
        let sprite_palette_1 = [0; 4];
        let window_y = 0;
        let window_x = 0;
        let render_line = 0;
        let palette_reference = [Color::RGB(155,188,15), Color::RGB(139,172,15), Color::RGB(48,98,48), Color::RGB(15,56,15)];
        let lock_vram = false;
        let vram_debug = false;

        GPU {
            sdl_context,
            canvas,
            vram_debug_canvas,
            event_pump,
            vram,
            tileset,
            state,
            state_clock,
            lcd_control,
            lcd_status,
            scroll_y,
            scroll_x,
            palette,
            sprite_palette_0,
            sprite_palette_1,
            window_y,
            window_x,
            render_line,
            palette_reference,
            lock_vram,
            vram_debug
        }
    }

    //TODO - implement remaining gpu registers
    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            0xFF40 => {
                return self.lcd_control;
            },
            0xFF41 => {
                return self.lcd_status;
            },
            0xFF42 => {
                return self.scroll_y;
            },
            0xFF43 => {
                return self.scroll_x;
            },
            0xFF44 => {
                return self.render_line;
            },
            0xFF47 => {
                warn!("Tried to read palette from GPU. You cannot read this value. Returning 0");
                return 0;
            },
            0xFF48 | 0xFF49 => {
                warn!("Tried to read sprite palette from GPU. You cannot read this value. Returning 0");
                return 0;
            },
            0xFF4A => {
                return self.window_y;
            },
            0xFF4B => {
                return self.window_x;
            },
            _ => {
                warn!("Tried to read GPU register {:#06X}. Unknown or not yet implemented. Returning 0", address);
                return 0;
            }
        }
    }

    //TODO - implement remaining gpu registers
    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => {
                self.lcd_control = value;
                return;
            },
            0xFF41 => {
                self.lcd_status = value;
            },
            0xFF42 => {
                self.scroll_y = value;
                return;
            },
            0xFF43 => {
                self.scroll_x = value;
                return;
            },
            0xFF44 => {
                warn!("Tried to write render_line in GPU. You cannot write this value. Returning without writing");
                return;
            },
            0xFF47 => {
                self.palette[3] = (value & 0xC0) >> 6;
                self.palette[2] = (value & 0x30) >> 4;
                self.palette[1] = (value & 0x0C) >> 2;
                self.palette[0] =  value & 0x03;
                return;
            },
            0xFF48 => {
                self.sprite_palette_0[3] = (value & 0xC0) >> 6;
                self.sprite_palette_0[2] = (value & 0x30) >> 4;
                self.sprite_palette_0[1] = (value & 0x0C) >> 2;
                self.sprite_palette_0[0] =  value & 0x03;
                return;
            },
            0xFF49 => {
                self.sprite_palette_1[3] = (value & 0xC0) >> 6;
                self.sprite_palette_1[2] = (value & 0x30) >> 4;
                self.sprite_palette_1[1] = (value & 0x0C) >> 2;
                self.sprite_palette_1[0] =  value & 0x03;
                return;
            },
            0xFF4A => {
                self.window_y = value;
                return;
            },
            0xFF4B => {
                self.window_x = value;
                return;
            },
            _ => {
                warn!("Tried to write to GPU register {:#06X}. Unknown or not yet implemented. Returning without writing", address);
                return;
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        return self.vram[(address - 0x8000) as usize];
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        // GameBoy code has requested to write to vram
        trace!("GPU Write. Address: {:#06X}. Translated: {:#06X}. Value: {:#04X}", address, address - 0x8000, value);

        if !self.lock_vram {
            let index = address - 0x8000;
            self.vram[index as usize] = value;

            if index < 0x1800 {
                let normalized_index = index & 0xFFFE;

                let byte1 = self.vram[normalized_index as usize];
                let byte2 = self.vram[(normalized_index + 1) as usize];

                let tile_index = index / 16;
                let row_index = (index % 16) / 2;

                for pixel_index in 0..8 {
                    let mask = 1 << (7 - pixel_index);
                    let lsb = byte1 & mask;
                    let msb = byte2 & mask;

                    let pixel_value = match (lsb != 0, msb != 0) {
                        (true, true) => self.palette[3],
                        (false, true) => self.palette[2],
                        (true, false) => self.palette[1],
                        (false, false) => self.palette[0],
                    };

                    self.tileset[tile_index as usize][row_index as usize][pixel_index as usize] = pixel_value;

                    if self.vram_debug {
                        // If VRAM Debugging - draw update
                        self.vram_debug_canvas.set_draw_color(self.palette_reference[pixel_value as usize]);
                        let result = self.vram_debug_canvas.draw_point(Point::new((((tile_index % 32) * 8) + pixel_index as u16) as i32, (((tile_index / 32) * 8) + row_index) as i32));
                        if result.is_err() {
                            error!("Error: {:?}", result.err());
                            exit(1);
                        }
                    }
                }
            } else {
                if self.vram_debug {
                    // If VRAM Debugging - draw update
                    self.vram_debug_canvas.set_draw_color(Color::RGB(255 - value, 255 - value, 255 - value));
                    let result = self.vram_debug_canvas.draw_line(Point::new(((index % 32) * 8) as i32, (index / 32) as i32), Point::new((((index % 32) * 8) + 8) as i32, (index / 32) as i32));
                    if result.is_err() {
                        error!("Error: {:?}", result.err());
                        exit(1);
                    }
                }
            }
        }

        if self.vram_debug {
            // If VRAM Debugging - update the canvas
            self.vram_debug_canvas.present();
        }
    }

    fn render_scanline(&mut self) {
        let mut tilemap_base: u16 = 0x1800;
        if self.get_background_tilemap() == 1 {
            tilemap_base = 0x1C00;
        }

        // TODO - better var names
        let step1 = (self.render_line + self.scroll_y) & 255;
        let step2: u16 = (step1 >> 3) as u16;
        let step3: u16 = step2 << 5;

        //TODO - somewhere in here we need to check bg_tileset flag and offset accordingly
        let offset_base: u16 = tilemap_base + step3 as u16;

        let y: u8 = (self.render_line + self.scroll_y) & 7;

        for x in 0..160 {
            let t_index: u8 = self.vram[(offset_base + (x / 8)) as usize];
            let color = self.palette_reference[self.palette[self.tileset[t_index as usize][y as usize][(x % 8)  as usize] as usize] as usize];

            self.canvas.set_draw_color(color);
            let result = self.canvas.draw_point(Point::new(x as i32, self.render_line as i32));
            if result.is_err() {
                error!("Error: {:?}", result.err());
                exit(1);
            }
        }
    }

    pub fn tick(&mut self, clock_m: u32) {

        self.state_clock += clock_m;

        match self.state {
            STATE_OAM_READ => {
                //trace!("GPU STATE: OAM READ");

                if self.state_clock >= 20 {
                    self.state_clock = 0;
                    self.state = STATE_VRAM_READ;
                }
            },
            STATE_VRAM_READ => {
                //trace!("GPU STATE: VRAM READ");

                if self.state_clock >= 43 {
                    self.state_clock = 0;
                    self.state = STATE_HBLANK;

                    // Render a scanline
                    self.render_scanline();
                }
            },
            STATE_HBLANK => {
                //trace!("GPU STATE: HBLANK");

                if self.state_clock >= 51 {
                    self.state_clock = 0;
                    self.render_line += 1;

                    if self.render_line == 143 {
                        self.state = STATE_VBLANK;

                        if self.get_display_status() == 1 {
                            self.canvas.present();
                        }
//                        MMU->intflags |= 1; // TODO - Do we need this?
                    } else {
                        self.state = STATE_OAM_READ;
                    }
                }
            },
            STATE_VBLANK => {
                //trace!("GPU STATE: VBLANK");

                if self.state_clock >= 114 {
                    self.state_clock = 0;
                    self.render_line += 1;

                    if self.render_line > 153 {
                        self.state = STATE_OAM_READ;
                        self.render_line = 0;
                    }
                }
            },
            _ => {
                warn!("Unknown GPU State: {}. Resetting to STATE_OAM_READ (2)", self.state);
                self.state = STATE_OAM_READ;
            }
        }
    }

    fn get_display_status(&self) -> u8 {
        return (self.lcd_control & 0x80) >> 7;
    }

    fn get_window_tilemap(&self) -> u8 {
        return (self.lcd_control & 0x40) >> 6;
    }

    fn get_window_status(&self) -> u8 {
        return (self.lcd_control & 0x20) >> 5;
    }

    fn get_background_tileset(&self) -> u8 {
        return (self.lcd_control & 0x10) >> 4;
    }

    fn get_background_tilemap(&self) -> u8 {
        return (self.lcd_control & 0x08) >> 3;
    }

    fn get_sprite_size(&self) -> u8 {
        return (self.lcd_control & 0x04) >> 2;
    }

    fn get_sprite_status(&self) -> u8 {
        return (self.lcd_control & 0x02) >> 1;
    }

    // TODO - this should be checked whenever we draw BG elements
    fn get_background_status(&self) -> u8 {
        return self.lcd_control & 0x01;
    }
}