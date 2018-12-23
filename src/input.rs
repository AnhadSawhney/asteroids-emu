// deal with *some* of the Asteroids game inputs

use sdl2::keyboard::Keycode;
use memory::Memory;

pub fn update_from_input(keycode: Keycode, active: bool, memory: &mut Memory) {
    let mem_val = if active {0xFF} else {0};
    match keycode {
        Keycode::S => {
            memory.mapped_io.sw1start = mem_val;
        },
        Keycode::RShift => {
            memory.mapped_io.swfire = mem_val;
        },
        Keycode::Z => {
            memory.mapped_io.swrotleft = mem_val;
        },
        Keycode::X => {
            memory.mapped_io.swrotrght = mem_val;
        },
        Keycode::Slash => {
            memory.mapped_io.swthrust = mem_val;
        },
        Keycode::Space => {
            memory.mapped_io.swhyper = mem_val;
        },
        Keycode::C => {
            memory.set_byte(0x53, 9);  // temp cheat to test bonus ship sound
        },
        _ => {},
    }
}
