// Emulate memory and memory mapped IO for Asteroids game
use find_folder;
use std::fs::File;
use std::io::prelude::*;

pub struct MappedIO {
    pub clck3khz: u8,   // from 0x2001
    pub halt: u8,
    pub swhyper: u8,
    pub swfire: u8,
    swdiagst: u8,
    swslam: u8,
    swtest: u8,
    
    swlcoin: u8,    // from 0x2400
    swccoin: u8,
    swrcoin: u8,
    pub sw1start: u8,
    sw2start: u8,
    pub swthrust: u8,
    pub swrotrght: u8,
    pub swrotleft: u8,

    swcoinage: u8,    // 0x2800
    swcnrmult: u8,
    swcncmult: u8,
    swlanguage: u8,

    pub godvg: u8,    // 0x3000
    lmpscns: u8,  // 0x3200
    watchdog: u8, // 0x3400
    sndexp: u8,   // 0x3600
    sndthump: u8, // 0x3a00
    sndsaucr: u8, // from 0x3c00
    sndsfire: u8,
    sndselsau: u8,
    sndthrust: u8,
    sndfire: u8,
    sndbonus: u8,
    sndreset: u8, // 0x3e00
}

impl MappedIO {
    fn new() -> MappedIO {
        MappedIO {
            clck3khz: 0,
            halt: 0,
            swhyper: 0,
            swfire: 0,
            swdiagst: 0,
            swslam: 0,
            swtest: 0x0,
    
            swlcoin: 0,
            swccoin: 0,
            swrcoin: 0,
            sw1start: 0,
            sw2start: 0,
            swthrust: 0,
            swrotrght: 0,
            swrotleft: 0,

            swcoinage: 0,    // free play
            swcnrmult: 0,
            swcncmult: 0,
            swlanguage: 0,   // english

            godvg: 0,
            lmpscns: 0,
            watchdog: 0,
            sndexp: 0,
            sndthump: 0,
            sndsaucr: 0,
            sndsfire: 0,
            sndselsau: 0,
            sndthrust: 0,
            sndfire: 0,
            sndbonus: 0,
            sndreset: 0,
        }
    }
}

pub struct Memory {
    game_ram: [u8; 1024],    // 0000-03FF / 8000-83FF
    dvg_ram: [u8; 4096],     // 4000-4FFF / C000-DFFF
    dvg_rom: [u8; 2048],     // 5000-57FF / D000-D7FF
    game_rom: [u8; 6144],    // 6800-7FFF / E800-FFFF
    pub mapped_io: MappedIO, 
}

impl Memory {
    pub fn new () -> Memory {
        let mut memory = Memory{
            game_ram: [0; 1024],
            dvg_ram: [0; 4096],
            dvg_rom: [0; 2048],
            game_rom: [0; 6144],
            mapped_io: MappedIO::new(),
        };

        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let rom_file = assets.join("asteroids.rom");
        let mut file = File::open(rom_file).expect("Error opening ROM file");
        file.read_exact(&mut memory.dvg_rom).unwrap();
        file.read_exact(&mut memory.game_rom).unwrap();
        memory
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        let a = addr as usize & 0x7FFF;
        if a < 0x400 {
            self.game_ram[a]
        }
        else if a >= 0x4000 && a < 0x5000 {
            self.dvg_ram[a - 0x4000]
        }
        else if a >= 0x5000 && a < 0x5800 {
            self.dvg_rom[a - 0x5000]
        }
        else if a >= 0x6800 {
            self.game_rom[a - 0x6800]
        }
        else if a == 0x2001 {
            self.mapped_io.clck3khz
        }
        else if a == 0x2002 {
            self.mapped_io.halt
        }
        else if a == 0x2007 {
            self.mapped_io.swtest
        }
        else if a == 0x2403 {
            self.mapped_io.sw1start
        }
        else if a == 0x2004 {
            self.mapped_io.swfire
        }
        else if a == 0x2405 {
            self.mapped_io.swthrust
        }
        else if a == 0x2406 {
            self.mapped_io.swrotrght
        }
        else if a == 0x2407 {
            self.mapped_io.swrotleft
        }
        else if a == 0x2003 {
            self.mapped_io.swhyper
        }
        else {
            0
        }
    }

    pub fn set_byte(&mut self, addr: u16, byte: u8) {
        let a = addr as usize & 0x7FFF;
        if a < 0x400 {
            self.game_ram[a] = byte;
        }
        else if a >= 0x4000 && a < 0x5000 {
            self.dvg_ram[a - 0x4000] = byte;
        }
        else if a == 0x2001 {
            self.mapped_io.clck3khz = byte;
        }
        else if a == 0x3000 {
            self.mapped_io.godvg = byte;
        }
    }
}
