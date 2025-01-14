// Emulate memory and memory mapped IO for Asteroids game

use find_folder;
use std::fs::File;
use std::io::prelude::*;

pub struct MappedIO {
    pub clck3khz: u8, // from 0x2001
    pub halt: u8,
    pub swhyper: u8,
    pub swfire: u8,
    //swdiagst: u8,
    //swslam: u8,
    //swtest: u8,

    //swlcoin: u8,    // from 0x2400
    //swccoin: u8,
    //swrcoin: u8,
    pub sw1start: u8,
    //sw2start: u8,
    pub swthrust: u8,
    pub swrotrght: u8,
    pub swrotleft: u8,

    //swcoinage: u8,    // 0x2800
    //swcnrmult: u8,
    //swcncmult: u8,
    //swlanguage: u8,
    pub godvg: u8, // 0x3000
    //lmpscns: u8,  // 0x3200
    //watchdog: u8, // 0x3400
    pub sndexp: u8,   // 0x3600
    pub sndthump: u8, // 0x3a00
    pub sndsaucr: u8, // from 0x3c00
    pub sndsfire: u8,
    pub sndselsau: u8,
    pub sndthrust: u8,
    pub sndfire: u8,
    pub sndbonus: u8,
    pub sndreset: u8, // 0x3e00
}

impl MappedIO {
    fn new() -> MappedIO {
        MappedIO {
            clck3khz: 0,
            halt: 0,
            swhyper: 0,
            swfire: 0,

            sw1start: 0,
            swthrust: 0,
            swrotrght: 0,
            swrotleft: 0,

            godvg: 0,
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
    game_ram: [u8; 1024], // 0000-03FF / 8000-83FF
    dvg_ram: [u8; 4096],  // 4000-4FFF / C000-CFFF
    dvg_rom: [u8; 2048],  // 5000-57FF / D000-D7FF
    game_rom: [u8; 6144], // 6800-7FFF / E800-FFFF
    pub mapped_io: MappedIO,
}

impl Memory {
    pub fn new() -> Memory {
        let mut memory = Memory {
            game_ram: [0; 1024],
            dvg_ram: [0; 4096],
            dvg_rom: [0; 2048],
            game_rom: [0; 6144],
            mapped_io: MappedIO::new(),
        };

        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets")
            .unwrap();
        let rom_file = assets.join("asteroids.rom");
        let mut file = File::open(rom_file).expect("Error opening ROM file");
        file.read_exact(&mut memory.dvg_rom).unwrap();
        file.read_exact(&mut memory.game_rom).unwrap();
        memory
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize & 0x7FFF;
        match addr {
            a if a < 0x400 => self.game_ram[a],
            a if a >= 0x4000 && a < 0x5000 => self.dvg_ram[a - 0x4000],
            a if a >= 0x5000 && a < 0x5800 => self.dvg_rom[a - 0x5000],
            a if a >= 0x6800 => self.game_rom[a - 0x6800],
            0x2001 => self.mapped_io.clck3khz,
            0x2002 => self.mapped_io.halt,
            0x2403 => self.mapped_io.sw1start,
            0x2004 => self.mapped_io.swfire,
            0x2405 => self.mapped_io.swthrust,
            0x2406 => self.mapped_io.swrotrght,
            0x2407 => self.mapped_io.swrotleft,
            0x2003 => self.mapped_io.swhyper,
            _ => 0,
        }
    }

    pub fn set_byte(&mut self, addr: u16, byte: u8) {
        let addr = addr as usize & 0x7FFF;
        match addr {
            a if a < 0x400 => {
                self.game_ram[a] = byte;
            }
            a if a >= 0x4000 && a < 0x5000 => {
                self.dvg_ram[a - 0x4000] = byte;
            }
            0x2001 => {
                self.mapped_io.clck3khz = byte;
            }
            0x3000 => {
                self.mapped_io.godvg = byte;
            }
            0x3600 => {
                self.mapped_io.sndexp = byte;
            }
            0x3A00 => {
                self.mapped_io.sndthump = byte;
            }
            0x3C00 => {
                self.mapped_io.sndsaucr = byte;
            }
            0x3C01 => {
                self.mapped_io.sndsfire = byte;
            }
            0x3C02 => {
                self.mapped_io.sndselsau = byte;
            }
            0x3C03 => {
                self.mapped_io.sndthrust = byte;
            }
            0x3C04 => {
                self.mapped_io.sndfire = byte;
            }
            0x3C05 => {
                self.mapped_io.sndbonus = byte;
            }
            _ => {}
        }
    }
}
