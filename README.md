## RustBoy
GameBoy (Original) Emulator written in Rust and SDL2.

Eventually want to support GameBoy Pocket/Color/Super.

## How to Run

    RustBoy.exe -b roms/boot.bin -r roms/game.gb
    cargo run -- -b roms/boot.bin -r roms/game.gb

```
-b, --bios <FILE>
Sets the BIN file to load that contains the BIOS. If no file is specified RustBoy will boot straight into the specified ROM.

-d, --debug <BOOL>
Enable/Disable Debug windows. (May slow performance)

-p, --pause <BOOL>
Starts the emulator in paused mode if set to true.

-r, --rom <FILE>
Sets the ROM file to load. If no ROM is specified RustBoy will hang after BIOS execution,or immediately if no BIOS is loaded.
```