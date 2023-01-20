// Emulate Atari Digital Vector Generator / display
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serialport::prelude::*;
use std::thread::sleep;
use std::time::Duration;

use memory::Memory;

#[derive(Debug)]
enum Instruction {
    VCTR, // 0x0 - 0x9
    LABS, // 0xA     - only instruction to use absolute coords
    HALT, // 0xB
    JSRL, // 0xC
    RTSL, // 0xD
    JMPL, // 0xE
    SVEC, // 0xF
}

pub struct Dvg {
    pc: u16,
    x: i16,
    y: i16,
    sf: i16,
    stack: [u16; 4],
    sp: usize,
    debug_mode: bool,
    serialoutput: bool,
    packet: [u8; 60],
    packetidx: i16,
}

impl Dvg {
    pub fn new(debug_mode: bool, serialoutput: bool) -> Dvg {
        Dvg {
            pc: 0,
            x: 0,
            y: 0,
            sf: 0,
            stack: [0; 4],
            sp: 0,
            debug_mode,
            serialoutput,
            packet: [0; 60],
            packetidx: 0,
        }
    }

    fn reset(&mut self) {
        self.pc = 1;
        self.x = 0;
        self.y = 0;
        self.sf = 0;
        self.stack = [0; 4];
        self.sp = 0;
        self.packet = [0; 60];
        self.packetidx = 0;
    }

    fn load_from_pc(&mut self, memory: &Memory) -> u16 {
        let addr = self.pc * 2 + 0x4000;
        self.pc += 1;
        (memory.get_byte(addr) as u16) | ((memory.get_byte(addr + 1) as u16) << 8)
    }

    fn instruction_from_word(word: u16) -> Instruction {
        let op_code = (word & 0xF000) >> 12;
        match op_code {
            0xA => Instruction::LABS,
            0xB => Instruction::HALT,
            0xC => Instruction::JSRL,
            0xD => Instruction::RTSL,
            0xE => Instruction::JMPL,
            0xF => Instruction::SVEC,
            _ => Instruction::VCTR,
        }
    }

    fn screen_y(y: i16, h: u32) -> i16 {
        // we find we have to flip y
        // also, y 0 thru 95 and 928 thru 1023 are not used
        (h as i32 - ((y - 96) as i32 * h as i32 / 832)) as i16
    }

    fn screen_x(x: i16, w: u32) -> i16 {
        (x as i32 * w as i32 / 1024) as i16
    }

    fn send_command(
        &mut self,
        x: i16,
        y: i16,
        z: u16,
        canvas: &mut Canvas<Window>,
        port: &mut Option<Box<dyn SerialPort>>,
    ) {
        if self.serialoutput {
            //let (w, h) = canvas.output_size().unwrap();

            if let Some(port) = port {
                // x and y are 0 to 1024

                // skip text and other nonsense
                if y > 850 && y < 1000 {
                    return;
                }

                if x > 150 && y > 835 && x < 220 && y < 870 {
                    return; // LIVES
                }

                if x > 395 && y > 785 && x < 640 && y < 825 {
                    return; // PRESS START
                }

                if x > 395 && y > 120 && x < 585 && y < 145 {
                    return; // COPYRIGHT ATARI
                }

                //let dist = (x - self.x) * (x - self.x) + (y - self.y) * (y - self.y);

                //let mut c = z;

                //if dist > 400 {
                //    c = 0;
                //    println!("Long");
                //}

                if z == 15 {
                    // skip drawing bullets
                    return;
                }

                let a = x as u16;
                let b = y as u16;
                let out = [z as u8, (a >> 8) as u8, a as u8, (b >> 8) as u8, b as u8];
                port.write(&out).ok();

                sleep(Duration::from_micros(50));

                //println!(
                //    "Sending: {},{},{},{},{}",
                //    out[0], out[1], out[2], out[3], out[4]
                //);

                // dont packetize. USB bandwith is not the limit, i2c execution is

                /*let i = self.packetidx as usize;
                self.packet[i] = z as u8;
                self.packet[i + 1] = (a >> 8) as u8;
                self.packet[i + 2] = a as u8;
                self.packet[i + 3] = (b >> 8) as u8;
                self.packet[i + 4] = b as u8;
                self.packetidx += 5;
                if (self.packetidx >= 60) {
                    port.write(&self.packet).ok();
                    self.packetidx = 0;
                }*/
            }
        }
    }

    fn line(&mut self, x: i16, y: i16, z: u16, canvas: &mut Canvas<Window>) {
        if z != 0 {
            let mut color = pixels::Color::RGBA(255, 255, 255, z as u8 * 17);

            match z {
                7 => {
                    color = pixels::Color::RGBA(255, 0, 0, 255);
                }
                8 => {
                    color = pixels::Color::RGBA(0, 255, 0, 255);
                }
                9 => {
                    color = pixels::Color::RGBA(0, 0, 255, 255);
                }
                10 => {
                    color = pixels::Color::RGBA(255, 255, 0, 255);
                }
                11 => {
                    color = pixels::Color::RGBA(255, 0, 255, 255);
                }
                12 => {
                    color = pixels::Color::RGBA(0, 255, 255, 255);
                }
                _ => {}
            }

            let (w, h) = canvas.output_size().unwrap();

            if x == self.x && y == self.y {
                // on the vector display, a single point can be extremely
                // bright. we can't do that so we just go bigger.
                let _ = canvas.filled_circle(Dvg::screen_x(x, w), Dvg::screen_y(y, h), 2, color);
            } else {
                let _ = canvas.line(
                    Dvg::screen_x(x, w),
                    Dvg::screen_y(y, h),
                    Dvg::screen_x(self.x, w),
                    Dvg::screen_y(self.y, h),
                    color,
                );

                /*let _ = canvas.rectangle(
                    Dvg::screen_x(150, w),
                    Dvg::screen_y(870, h),
                    Dvg::screen_x(220, w),
                    Dvg::screen_y(835, h),
                    color,
                );*/

                /*let _ = canvas.rectangle(
                    Dvg::screen_x(395, w),
                    Dvg::screen_y(120, h),
                    Dvg::screen_x(585, w),
                    Dvg::screen_y(145, h),
                    color,
                );*/
            }
        }
        self.x = x;
        self.y = y;
    }

    fn shift(a: u16, s: i16) -> u16 {
        if s >= 0 {
            a >> s
        } else {
            a << (-s)
        }
    }

    pub fn render(
        &mut self,
        memory: &mut Memory,
        canvas: &mut Canvas<Window>,
        port: &mut Option<Box<dyn SerialPort>>,
    ) {
        memory.mapped_io.halt = 0xFF;
        memory.mapped_io.godvg = 0;
        self.reset();
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        while memory.mapped_io.halt != 0 {
            self.execute_instruction(memory, canvas, port);
        }
        canvas.present();
        /*self.send_command(0, 95, 0, canvas, port);
        self.send_command(0, 95, 12, canvas, port);
        self.send_command(1023, 95, 12, canvas, port);
        self.send_command(1023, 928, 12, canvas, port);
        self.send_command(0, 928, 12, canvas, port);
        self.send_command(0, 95, 12, canvas, port);
        self.send_command(512, 512, 0, canvas, port);*/
        self.send_command(0, 0, 0, canvas, port);
        self.send_command(0, 0, 11, canvas, port);
        self.send_command(1023, 0, 11, canvas, port);
        self.send_command(1023, 1023, 11, canvas, port);
        self.send_command(0, 1023, 11, canvas, port);
        self.send_command(0, 0, 11, canvas, port);
        self.send_command(512, 512, 0, canvas, port);
    }

    fn execute_instruction(
        &mut self,
        memory: &mut Memory,
        canvas: &mut Canvas<Window>,
        port: &mut Option<Box<dyn SerialPort>>,
    ) {
        let instr_addr = self.pc;
        let op_word1 = self.load_from_pc(memory);
        let op = Dvg::instruction_from_word(op_word1);
        let op_word2 = match op {
            Instruction::VCTR | Instruction::LABS => self.load_from_pc(memory),
            _ => 0,
        };
        if self.debug_mode {
            println!(
                "---DVG X: {}, Y: {}, SF: {}, SP: {}, PC: {}",
                self.x, self.y, self.sf, self.sp, self.pc
            );
            if op_word2 == 0 {
                println!("---DVG {:04X} {:016b} {:?}", instr_addr, op_word1, op);
            } else {
                println!(
                    "---DVG {:04X} {:016b} {:016b} {:?}",
                    instr_addr, op_word1, op_word2, op
                );
            }
            println!("---DVG");
        }
        match op {
            Instruction::VCTR => {
                let ys = (0x400 & op_word1) != 0;
                let delta_y = 0x3FF & op_word1;
                let z = (0xF000 & op_word2) >> 12;
                let xs = (0x400 & op_word2) != 0;
                let delta_x = 0x3FF & op_word2;
                let shift_bits = 9 - ((op_word1 & 0xF000) >> 12) as i16 + self.sf;
                let x = self.x + Dvg::shift(delta_x, shift_bits) as i16 * if xs { -1 } else { 1 };
                let y = self.y + Dvg::shift(delta_y, shift_bits) as i16 * if ys { -1 } else { 1 };
                self.send_command(x, y, z, canvas, port);
                self.line(x, y, z, canvas);
            }
            Instruction::LABS => {
                // CUR
                let ys = (0x400 & op_word1) != 0;
                let y = 0x3FF & op_word1;
                let xs = (0x400 & op_word2) != 0;
                let x = 0x3FF & op_word2;
                let sf = (op_word2 & 0xF000) >> 12;
                self.sf = if sf & 0x8 == 0 {
                    -(sf as i16)
                } else {
                    16 - sf as i16
                };
                self.y = if ys {
                    0 - ((y ^ 0x3FF) + 1) as i16
                } else {
                    y as i16
                };
                self.x = if xs {
                    0 - ((x ^ 0x3FF) + 1) as i16
                } else {
                    x as i16
                };

                self.send_command(self.x, self.y, 0, canvas, port);
            }
            Instruction::HALT => {
                memory.mapped_io.halt = 0;
            }
            Instruction::JSRL => {
                if self.sp > 3 {
                    panic!("DVG stack overflow");
                }
                let addr = op_word1 & 0xFFF;
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = addr;
            }
            Instruction::RTSL => {
                if self.sp == 0 {
                    panic!("DVG stack underflow");
                }
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            }
            Instruction::JMPL => {
                let a = op_word1 & 0xFFF;
                self.pc = a;
            }
            Instruction::SVEC => {
                let sf = ((op_word1 & 0x800) >> 11) + ((op_word1 & 0x8) >> 2);
                let ys = (op_word1 & 0x400) != 0;
                let delta_y = op_word1 & 0x300;
                let xs = (op_word1 & 0x4) != 0;
                let delta_x = (op_word1 & 0x3) << 8;
                let z = (op_word1 & 0xF0) >> 4;

                let shift_bits = (7 - sf as i16) + self.sf;
                let x = self.x + Dvg::shift(delta_x, shift_bits) as i16 * if xs { -1 } else { 1 };
                let y = self.y + Dvg::shift(delta_y, shift_bits) as i16 * if ys { -1 } else { 1 };
                self.send_command(x, y, z, canvas, port);
                self.line(x, y, z, canvas);
            }
        };
    }
}
