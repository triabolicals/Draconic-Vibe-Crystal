pub struct RandomizerStatus {
    pub alear_person_set: bool,
    pub well_randomized: bool,
    pub enemy_emblem_randomized: bool,
    pub enemy_unit_randomized: bool,
    pub emblem_unit_skill_randomized: bool,
    pub skill_randomized: bool,
    pub emblem_data_randomized: bool,
    pub emblem_apt_randomized: bool,
    pub shop_randomized: bool,
    pub enabled: bool,
    pub stat_caps: bool,
    pub accessory: bool,
    pub kizuna_replacements: bool,
    pub map_tile: bool,
    pub seed: i32,
    pub learn_skill: bool,
    pub continious_random_chapter: String,
    pub enemy_edelgard: bool,
    pub inspectors_set: bool,
}

impl RandomizerStatus {
    pub fn new() -> Self {
        RandomizerStatus{
            alear_person_set: false,
            well_randomized: false,
            enemy_emblem_randomized: false,
            enemy_unit_randomized: false,
            emblem_unit_skill_randomized: false,
            skill_randomized: false,
            emblem_data_randomized: false,
            emblem_apt_randomized: false,
            shop_randomized: false,
            enabled: false,
            stat_caps: false,
            accessory: false,
            kizuna_replacements: false,
            map_tile: false,
            learn_skill: false,
            seed: 0,
            continious_random_chapter: String::new(),
            enemy_edelgard: false,
            inspectors_set: false,
        }
    }
    pub fn reset(&mut self) {
        self.alear_person_set = false;
        self.well_randomized = false;
        self.enemy_emblem_randomized = false;
        self.enemy_unit_randomized = false;
        self.emblem_unit_skill_randomized = false;
        self.skill_randomized = false;
        self.emblem_data_randomized = false;
        self.emblem_apt_randomized = false;
        self.shop_randomized = false;
        self.enabled = false;
        self.stat_caps = false;
        self.accessory = false;
        self.kizuna_replacements = false;
        self.map_tile = false;
        self.seed = 0;
        self.learn_skill = false;
        self.continious_random_chapter = "".to_string();
        self.enemy_edelgard = false;
        self.inspectors_set = false;
    }
    pub fn set_enable(&mut self) { self.enabled = true; }
    pub fn map_complete(&mut self) {
        self.enemy_edelgard = false;
        self.enemy_edelgard = false;
        self.inspectors_set = false;
        self.map_tile = false;
    }

}