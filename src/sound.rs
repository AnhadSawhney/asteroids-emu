use find_folder;
use sdl2::mixer::{Chunk, Channel};
use memory::Memory;

struct SoundEffect {
    signal: u8,
    chunk: Chunk,
    channel: Option<Channel>,
}

impl SoundEffect {
    fn new(file_name: &str) -> SoundEffect {
        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();
        let path = assets.join(file_name);
        let sound_chunk_res = Chunk::from_file(&path);
        let chunk = match sound_chunk_res {
            Ok(sound_chunk) => sound_chunk,
            _ => {panic!("Failed to load sound file {:?}", path);},
        };
        SoundEffect {signal: 0, chunk, channel: None}
    }

    fn play(&self) {
        let _play_res = Channel::all().play(&self.chunk, 0);
    }

    fn play_continuous(&mut self) {
        let play_res = Channel::all().play(&self.chunk, 1000);
        self.channel = if let Ok(ch) = play_res {Some(ch)} else {None};
    }

    fn stop(&mut self) {
        if let Some(ch) = self.channel {
            ch.halt();
            self.channel = None;
        }
    }
}

pub struct Sounds {
    ship_fire: SoundEffect,
    explosion: SoundEffect,
    large_ufo: SoundEffect,
    small_ufo: SoundEffect,
    ufo_fire: SoundEffect,
    extra_life: SoundEffect,
    thump_low: SoundEffect,
    thump_high: SoundEffect,
    thrust: SoundEffect,
    extra_life_countdown: u32,
}

impl Sounds {
    pub fn new() -> Sounds {
        Sounds {
            ship_fire: SoundEffect::new("ship_fire.ogg"),
            explosion: SoundEffect::new("explosion.ogg"),
            large_ufo: SoundEffect::new("large_ufo.ogg"),
            small_ufo: SoundEffect::new("small_ufo.ogg"),
            ufo_fire: SoundEffect::new("ufo_fire.ogg"),
            extra_life: SoundEffect::new("extra_life.ogg"),
            thump_low: SoundEffect::new("thump_low.ogg"),
            thump_high: SoundEffect::new("thump_high.ogg"),
            thrust: SoundEffect::new("thrust.ogg"),
            extra_life_countdown: 0,
        }
    }

    pub fn play(&mut self, memory: &Memory) {
        // generally a sound effect is off at zero signal and endures for
        // a non-zero signal. we use that transition from low to high to
        // initiate a pre-prepared sound 
        let signal = memory.mapped_io.sndfire;
        if self.ship_fire.signal < signal {
            self.ship_fire.play();
        }
        self.ship_fire.signal = signal;

        // explosion
        let signal = memory.mapped_io.sndexp & 0x3F;
        if self.explosion.signal < signal {
            self.explosion.play();
        }
        self.explosion.signal = signal;

        // ufo (use large ufo to store last signal)
        let signal = memory.mapped_io.sndsaucr;
        if self.large_ufo.signal < signal {
            // start ufo sound
            if memory.mapped_io.sndselsau == 160 {
                self.large_ufo.play_continuous();
            }
            else {
                self.small_ufo.play_continuous();
            }
        }
        else if self.large_ufo.signal > signal {
            self.large_ufo.stop();
            self.small_ufo.stop();
        }
        self.large_ufo.signal = signal;

        // ufo fire
        let signal = memory.mapped_io.sndsfire;
        if self.ufo_fire.signal < signal {
            self.ufo_fire.play();
        }
        self.ufo_fire.signal = signal;

        // extra life
        let signal = memory.mapped_io.sndbonus;
        if signal > 0 && self.extra_life_countdown == 0 {
            self.extra_life.play();
            self.extra_life_countdown = 10000;
        }
        if self.extra_life_countdown > 0 {
            self.extra_life_countdown -= 1;
        }

        // thump
        let signal = memory.mapped_io.sndthump;
        if signal > 4 && self.thump_low.signal <= 4 {
            if signal == 16 {
                self.thump_low.play();
            }
            else {
                self.thump_high.play();
            }
        }
        self.thump_low.signal = signal;

        // thrust
        let signal = memory.mapped_io.sndthrust;
        if self.thrust.signal < signal {
            self.thrust.play_continuous();
        }
        else if self.thrust.signal > signal {
            self.thrust.stop();
        }
        self.thrust.signal = signal;
    }
}
