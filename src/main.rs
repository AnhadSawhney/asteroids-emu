// combine roms cat 035127-02.np3 035145-02.ef2 035144-02.h2 035143-02.j2 > asteroids.rom

// Enulator to run Asteroids game
extern crate find_folder;
extern crate sdl2;
extern crate serialport;

use serialport::prelude::*;

mod cpu;
mod display;
mod input;
mod memory;
mod sound;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mixer::{Channel, AUDIO_S16LSB, DEFAULT_CHANNELS, INIT_OGG, MAX_VOLUME};
use sdl2::video::WindowPos;
use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: u32 = 10240; // i.e. bigger than maximised dimensions
const NMI_CYCLES: u64 = 6000;
const TICKS_PER_SLEEP: u32 = 20;

use cpu::Cpu;
use display::Dvg;
use memory::Memory;
use sound::Sounds;

fn main() {
    let args: Vec<String> = env::args().collect();
    let debug = args.len() > 1 && args[1] == "debug";

    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let window = video_subsys
        .window("Asteroids Emu", SCREEN_WIDTH, SCREEN_WIDTH)
        .resizable()
        .maximized()
        .opengl()
        .build()
        .unwrap();

    let _audio = sdl_context.audio().unwrap();
    sdl2::mixer::open_audio(
        44_100,       // frequency
        AUDIO_S16LSB, //format
        DEFAULT_CHANNELS,
        1_024, // chunk size
    )
    .unwrap();
    let _mixer_context = sdl2::mixer::init(INIT_OGG).unwrap();
    Channel::all().set_volume(MAX_VOLUME / 2);

    let tick_time = Duration::new(0, 1000000000 / 3000 * TICKS_PER_SLEEP);

    let mut canvas = window.into_canvas().build().unwrap();

    let mut events = sdl_context.event_pump().unwrap();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = 115200;

    let mut serialoutput = true;

    // Open the serial port
    let mut port: Option<Box<dyn SerialPort>> =
        match serialport::open_with_settings("/dev/ttyUSB0", &settings) {
            Ok(port) => Some(port),
            Err(e) => {
                println!("Error opening serial port: {}", e);
                serialoutput = false;
                None
            }
        };

    let mut cpu = Cpu::new(debug);
    let mut dvg = Dvg::new(debug, serialoutput);
    let mut memory = Memory::new();
    let mut sounds = Sounds::new();
    cpu.reset(&memory);
    let mut next_nmi = NMI_CYCLES;

    'main: loop {
        let now = Instant::now();
        for _i in 0..TICKS_PER_SLEEP {
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main,

                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        if keycode == Keycode::Escape {
                            break 'main;
                        } else {
                            input::update_from_input(keycode, true, &mut memory);
                        }
                    }

                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        input::update_from_input(keycode, false, &mut memory);
                    }

                    Event::Window { win_event, .. } => {
                        match win_event {
                            WindowEvent::Shown => {
                                //println!("Get resolution");
                                // there must be a better way of doing this...
                                // if we start maximised, we can then use the
                                // resulting client area size to discover the
                                // maximum viewable 1024:832 window
                                let (w, h) = canvas.output_size().unwrap();
                                let best_w_fit = (w / 16) < (h / 13);
                                let nw = if best_w_fit { w } else { 1024 * h / 832 };
                                let nh = if best_w_fit { 832 * w / 1024 } else { h };
                                canvas.window_mut().set_maximum_size(nw, nh).unwrap();
                                canvas.window_mut().restore();
                                canvas
                                    .window_mut()
                                    .set_position(WindowPos::Centered, WindowPos::Centered);
                                // leave it so as user can go bigger if they
                                // want to - albeit with incorrect aspect ratio
                                canvas.window_mut().set_maximum_size(w, h).unwrap();
                                if debug {
                                    println!("Screen size {} x {}", nw, nh);
                                }
                            }
                            _ => {}
                        }
                    }

                    _ => {}
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
                    dvg.render(&mut memory, &mut canvas, &mut port);
                }
            }
            sounds.play(&memory);

            memory.mapped_io.clck3khz = ((cpu.cycle / 500) & 0xFF) as u8;
        }
        // sleeping at every 3khz tick is too frequent as there can still be
        // overruns on my laptop, so we even things out over a number of ticks
        // (of course, if running in debug mode, this is all moot)
        let delta = now.elapsed();
        if delta < tick_time {
            sleep(tick_time - delta);
        } else {
            println!("Overrun {:?}", delta - tick_time);
        }
    }
}
