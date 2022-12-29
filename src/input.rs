// deal with *some* of the Asteroids game inputs

use memory::Memory;
use sdl2::keyboard::Keycode;

pub fn update_from_input(keycode: Keycode, active: bool, memory: &mut Memory) {
    let mem_val = if active { 0xFF } else { 0 };
    match keycode {
        Keycode::S => {
            memory.mapped_io.sw1start = mem_val;
        }
        Keycode::Space => {
            memory.mapped_io.swfire = mem_val;
        }
        Keycode::Left => {
            memory.mapped_io.swrotleft = mem_val;
        }
        Keycode::Right => {
            memory.mapped_io.swrotrght = mem_val;
        }
        Keycode::Up => {
            memory.mapped_io.swthrust = mem_val;
        }
        Keycode::LShift => {
            memory.mapped_io.swhyper = mem_val;
        }
        _ => {}
    }
}
