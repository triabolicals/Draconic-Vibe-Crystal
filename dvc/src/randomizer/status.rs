use crate::randomizer::RANDOMIZER_STATUS;

pub struct RandomizerStatus {
    pub alear_person_set: bool,
    pub well_randomized: bool,
    pub enabled: bool,
    pub kizuna_replacements: bool,
    pub map_tile: bool,
    pub seed: i32,
    pub learn_skill: bool,
    pub inspectors_set: bool,
    pub init: bool,
    pub tilabolical: [u8; 1024],
}

impl RandomizerStatus {
    pub fn new() -> Self {
        RandomizerStatus{
            alear_person_set: false,
            well_randomized: false,
            enabled: false,
            kizuna_replacements: false,
            map_tile: false,
            learn_skill: false,
            seed: 0,
            init: false,
            inspectors_set: false,
            tilabolical: [0; 1024]
        }
    }
    pub fn reset(&mut self) {
        self.alear_person_set = false;
        self.well_randomized = false;
        self.enabled = false;
        self.kizuna_replacements = false;
        self.map_tile = false;
        self.seed = 0;
        self.init = false;
        self.learn_skill = false;
        self.inspectors_set = false;
        self.tilabolical = [0; 1024];
    }
    pub fn get_tile(x: u8, z: u8) -> i32 {
        if let Some(v) = RANDOMIZER_STATUS.try_read().ok() {
            let index = z as usize * 32 + x as usize;
            v.tilabolical[index] as i32
        }
        else { 0 } 
    }
    pub fn set_enable(&mut self) { self.enabled = true; }
    pub fn is_init() -> bool {
        if let Some(v) = RANDOMIZER_STATUS.try_read().ok() {
            v.init
        }
        else { false }
    }
    pub fn set_init(init: bool) {
        if let Some(mut v) = RANDOMIZER_STATUS.try_write().ok() {
            v.init = init;
        }
    }
    pub fn map_complete(&mut self) {
        self.inspectors_set = false;
        self.map_tile = false;
    }
}