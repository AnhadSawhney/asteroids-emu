// Enulator to run Asteroids game
extern crate find_folder;
extern crate sdl2;

mod cpu;
mod memory;
mod display;
mod input;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::WindowPos;
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::env;

const SCREEN_WIDTH: u32 = 10240;    // i.e. bigger than maximised dimensions
const NMI_CYCLES: u64 = 6000;
const TICKS_PER_SLEEP: u32 = 20;

use cpu::Cpu;
use display::Dvg;
use memory::Memory;

fn main() {
    let args: Vec<String> = env::args().collect();
    let debug = args.len() > 1 && args[1] == "debug";

    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let window = video_subsys.window("Asteroids Emu", SCREEN_WIDTH, SCREEN_WIDTH)
        .resizable()
        .maximized()
        .opengl()
        .build()
        .unwrap();

    let tick_time = Duration::new(0, 1000000000 / 3000 * TICKS_PER_SLEEP);

    let mut canvas = window.into_canvas().build().unwrap();

    let mut events = sdl_context.event_pump().unwrap();

    let mut cpu = Cpu::new(debug);
    let mut dvg = Dvg::new(debug);
    let mut memory = Memory::new();
    cpu.reset(&memory);
    let mut next_nmi = NMI_CYCLES;

    'main: loop {
        let now = Instant::now();
        for _i in 0..TICKS_PER_SLEEP {
            for event in events.poll_iter() {
                match event {
                    Event::Quit {..} => break 'main,

                    Event::KeyDown {keycode: Some(keycode), ..} => {
                        if keycode == Keycode::Escape {
                            break 'main;
                        }
                        else {
                            input::update_from_input(keycode, true, &mut memory);
                        }
                    },

                    Event::KeyUp {keycode: Some(keycode), ..} => {
                        input::update_from_input(keycode, false, &mut memory);
                    },

                    Event::Window {win_event, ..} => {
                        match win_event {
                            WindowEvent::Shown => {
                                // there must be a better way of doing this...
                                // if we start maximised, we can then use the
                                // resulting client area size to discover the
                                // maximum viewable square window
                                let (h, w) = canvas.output_size().unwrap();
                                let s = h.min(w);
                                canvas.window_mut().set_maximum_size(s, s).unwrap();
                                canvas.window_mut().restore();
                                canvas.window_mut().set_position(WindowPos::Centered, WindowPos::Centered);
                            },
                            _ => {},
                        }
                    },

                    _ => {},
                }
            }

            let next_tick = ((cpu.cycle / 500) + 1) * 500;
            while cpu.cycle < next_tick {
                while cpu.cycle < next_tick && memory.mapped_io.godvg == 0 {
                    cpu.execute_instruction(&mut memory);
                    if cpu.cycle >= next_nmi {
                        cpu.initiate_nmi(&mut memory);
                        next_nmi += NMI_CYCLES;
                    }
                }

                if memory.mapped_io.godvg != 0 {
                    dvg.render(&mut memory, &mut canvas);
                }
            }

            memory.mapped_io.clck3khz = ((cpu.cycle / 500) & 0xFF) as u8;  
        }
        // sleeping at every 3khz tick is too frequent as there can still be
        // overruns on my laptop, so we even things out over a number of ticks
        // (of course, if running in debug mode, this is all moot)
        let delta = now.elapsed();
        if delta < tick_time {
            sleep(tick_time - delta);
        }
        else {
            println!("Overrun {:?}", delta - tick_time);
        }
    }
}

