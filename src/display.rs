// Emulate Atari Digital Vector Generator / display

use sdl2::pixels;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::video::Window;
use sdl2::render::Canvas;

use memory::Memory;

#[derive(Debug)]
enum Instruction {
    VCTR,    // 0x0 - 0x9
    LABS,    // 0xA     - only instruction to use aobsolute coords
    HALT,    // 0xB
    JSRL,    // 0xC
    RTSL,    // 0xD
    JMPL,    // 0xE
    SVEC,    // 0xF
}

pub struct Dvg {
    pc: u16,
    x: i16,
    y: i16,
    sf: i16,
    stack: [u16; 4],
    sp: usize,
    debug_mode: bool,
}

impl Dvg {
    pub fn new(debug_mode: bool) -> Dvg {
        Dvg {pc: 0, x: 0, y: 0, sf: 0, stack: [0; 4], sp: 0, debug_mode}
    }

    fn reset(&mut self) {
        self.pc = 1;
        self.x = 0;
        self.y = 0;
        self.sf = 0;
        self.stack = [0; 4];
        self.sp = 0;
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

    fn line(&mut self, x: i16, y: i16, z: u16, canvas: &mut Canvas<Window>) {
        if z != 0 {
            let color = pixels::Color::RGBA(255, 255, 255, z as u8 * 17); 
            let (h,w) = canvas.output_size().unwrap();

            // we find we have to flip y
            // Hitch-Hacker's guide suggests that 0,0 is the top left corner
            // However, the Computer Archeology documentation seems to be correct
            // in its assertion that 0,0 is the bottom left corner
            if x == self.x && y == self.y {
                // this is quite unsatisfactory compared with the vector display
                // where a single point can be extremely bright
                let _ = canvas.pixel(
                    (x as i32 * w as i32 / 1024) as i16,
                    (h as i32 - (y as i32 * h as i32 / 1024)) as i16,
                    color);
            }
            else {
                let _ = canvas.line(
                    (self.x as i32 * w as i32 / 1024) as i16,
                    (h as i32 - (self.y as i32 * h as i32 / 1024)) as i16,
                    (x as i32 * w as i32 / 1024) as i16,
                    (h as i32 - (y as i32 * h as i32 / 1024)) as i16,
                    color);
            }
        }
        self.x = x;
        self.y = y;
    }

    fn shift(a: u16, s: i16) -> u16 {
        if s >= 0 {
            a >> s
        }
        else {
            a << (-s)
        }
    }

    pub fn render(&mut self, memory: &mut Memory, canvas: &mut Canvas<Window>) {
        memory.mapped_io.halt = 0xFF;
        memory.mapped_io.godvg = 0;
        self.reset();
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        while memory.mapped_io.halt != 0 {
            self.execute_instruction(memory, canvas);
        }
        canvas.present();
    }

    fn execute_instruction(&mut self, memory: &mut Memory, canvas: &mut Canvas<Window>) {
        let instr_addr = self.pc;
        let op_word1 = self.load_from_pc(memory);
        let op = Dvg::instruction_from_word(op_word1);
        let op_word2 = match op {
            Instruction::VCTR | Instruction::LABS => self.load_from_pc(memory),
            _ => 0,
        };
        if self.debug_mode {
            println!("---DVG X: {}, Y: {}, SF: {}, SP: {}, PC: {}", self.x, self.y, self.sf, self.sp, self.pc);
            if op_word2 == 0 {
                println!("---DVG {:04X} {:016b} {:?}", instr_addr, op_word1, op);
            }
            else {
                println!("---DVG {:04X} {:016b} {:016b} {:?}", instr_addr, op_word1, op_word2, op);
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
                let x = self.x + Dvg::shift(delta_x, shift_bits) as i16 *
                    if xs {-1} else {1};
                let y = self.y + Dvg::shift(delta_y, shift_bits) as i16 *
                    if ys {-1} else {1};
                self.line(x, y, z, canvas);
            },
            Instruction::LABS => {
                let ys = (0x400 & op_word1) != 0;
                let y = 0x3FF & op_word1;
                let xs = (0x400 & op_word2) != 0;
                let x = 0x3FF & op_word2;
                let sf = (op_word2 & 0xF000) >> 12;
                self.sf = if sf & 0x8 == 0 {- (sf as i16)} else {16 - sf as i16};
                self.y = if ys {0 - ((y ^ 0x3FF) + 1) as i16} else {y as i16};
                self.x = if xs {0 - ((x ^ 0x3FF) + 1) as i16} else {x as i16};
            },
            Instruction::HALT => {
                memory.mapped_io.halt = 0;
            },
            Instruction::JSRL => {
                if self.sp > 3 {
                    panic!("DVG stack overflow");
                }
                let addr = op_word1 & 0xFFF;
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = addr;
            },
            Instruction::RTSL => {
                if self.sp == 0 {
                    panic!("DVG stack underflow");
                }
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            },
            Instruction::JMPL => {
                let a = op_word1 & 0xFFF;
                self.pc = a;
            },
            Instruction::SVEC => {
                let sf = ((op_word1 & 0x800) >> 11) + ((op_word1 & 0x8) >> 2);
                // Hitch Hacker's guide seems to mix up X and Y here whereas
                // Computer Archeology seems to get it right
                let ys = (op_word1 & 0x400) != 0;
                let delta_y = op_word1 & 0x300;
                let xs = (op_word1 & 0x4) != 0;
                let delta_x = (op_word1 & 0x3) << 8;
                let z = (op_word1 & 0xF0) >> 4;

                let shift_bits = (7 - sf as i16) + self.sf;
                let x = self.x + Dvg::shift(delta_x, shift_bits) as i16 *
                    if xs {-1} else {1};
                let y = self.y + Dvg::shift(delta_y, shift_bits) as i16 *
                    if ys {-1} else {1};
                self.line(x, y, z, canvas);
            },
        };
    }
}
