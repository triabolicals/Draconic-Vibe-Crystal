use utils::{clamp_value, min};
use std::sync::OnceLock;
use super::*;

pub static WEAPONDATA: OnceLock<WeaponDatabase> =  OnceLock::new();

pub struct WeaponData {
    pub item_index: i32,
    pub weapon_type: u8,
    pub might: u8,
    pub weight: u8,
    pub hit: i16,
    pub crit: i16,
    pub avo: i16,
    pub secure: i16,
    pub rank: u8,
    pub flag: i32,
    /*
    pub is_magic: bool, 0
    pub is_smash: bool, 1
    pub is_range: bool, 2
    pub is_crit: bool, 3 8|64|128|32
    pub is_slim: bool, 4
    pub is_rare: bool, 5
    pub is_effective: bool, 6
    pub no_follow_up: bool, 7
    pub edible: bool, 8
     */
}
pub struct DragonStoneData {
    pub item_index: i32,
    pub is_enemy_only: bool,
}
impl DragonStoneData {
    pub fn new(item: &ItemData) -> Self {
        Self {
            item_index: item.parent.index,
            is_enemy_only: item.flag.value & 2 != 0,
        }
    }
}
impl WeaponData {
    pub fn new(item: &ItemData, effectivness: bool) -> Self {
        let flags = item.get_flag().value;
        let magic = if item.attr == 2 { true }
            else if item.attr == 1 && flags & 65536 != 0 { true }
            else { false }; 

        let e_skills = item.get_equip_skills();
        let mut flag = 0;
        if magic { flag |= 1; }
        if e_skills.find_sid("SID_スマッシュ").is_some() { flag |= 2; }
        if ((item.kind > 3 || item.kind < 7) && item.range_o > 2) ||
            ((item.kind < 4 || item.kind > 6 )&& item.range_o > 1) {
            flag |= 4;
        }
        if item.critical >= 15 { flag |= 8; }
        if item.secure >= 15 { flag |= 16; }
        if item.price == 100 || flags & 18 != 0 || item.range_o > 5 || flags & 3 != 0  { flag |= 32; }
        if effectivness { flag |= 64; }
        if e_skills.find_sid("SID_追撃不可").is_some() { flag |= 128; }
        if item.flag.value & 4 != 0 { flag |= 256 }
        if e_skills.find_sid("SID_２回行動").is_some() { flag |= 512; }
        Self {
            item_index: item.parent.index,
            weapon_type: item.kind as u8,
            might: item.power,
            weight: item.weight,
            hit: item.hit,
            crit: item.critical,
            avo: item.avoid,
            secure: item.secure,
            rank: item.get_weapon_level() as u8,
            flag,
        }
    }

    pub fn can_replace(&self, item2: &WeaponData, kind: u8, enemy: bool) -> bool {
        if item2.flag & 32 != 0 { 
            if !enemy || !DVCVariables::is_main_chapter_complete(8) { return false; }
        }
        if item2.weapon_type != kind { return false; }
        if (self.flag & item2.flag) & 16 != 0 { return true; }
        if item2.rank == self.rank { return true; }
        else {
            let might_diff = item2.might as i8 - self.might as i8;
            might_diff < 15 && might_diff > -2
        }
    }
    pub fn is_valid_tome(&self, tome_rank: i32, enemy: bool) -> bool {
        self.weapon_type == 6 && ( enemy == (self.flag & 32 != 0)) && self.rank == tome_rank as u8
    }
}

pub struct StaffData {
    pub item_index: i32,
    pub staff_type: u8,
    pub rare: bool,
    pub rank: u8,
    pub availibility: [bool; 5],
}

impl StaffData {
    pub fn new(item: &ItemData) -> Self {
        let mut avail: [bool; 5] = [false; 5];
        let staff_kind; 
        match item.usetype {
            2 => {
                staff_kind = 1; //Heal
                if item.range_o > 1 {   // Physic
                    avail = [false, item.range_o < 9, item.range_o < 9,  item.range_o >= 9, item.range_o >= 9];
                }
                else {
                    let pwr1 = item.power < 11;
                    let pwr2 = item.power < 21;
                    avail = [pwr1,  pwr1, pwr2,  pwr2, item.range_o >= 21];
                }
            },
            3 => {
                staff_kind = 1; //Fortify
                avail = [false, false, true, true, true];
            },
            5 => {  //Warp
                staff_kind = 2;
                avail = [false, false, true, true, true];
            },
            6 => {  //Rescue
                staff_kind = 3;
                avail = [false, false, true, true, true];
            },
            9|10 => {  // Rewarp /Freeze
                staff_kind = 2;
                avail = [false, true, true, true, true];
            },
            15|27 => {
                staff_kind = 2;
                avail = [false, false, false, true, true];
            },
            8|29 => { // Fracture
                staff_kind = 2;
                avail = [false, true, true, true, true];
            },
            _ => { staff_kind = 0; }
        }
        Self {
            item_index: item.parent.index,
            staff_type: staff_kind as u8,
            rank: item.get_weapon_level() as u8,
            rare: item.flag.value & 3 != 0 || item.usetype == 27 || item.usetype == 7,
            availibility: avail,
        }
    }
    pub fn can_add(&self, level: i32, staff_type: u8, max_rank: u8, enemy: bool) -> bool {
        let avail = get_magic_staff_by_level(level);
        if self.rare && !enemy { false }
        else { self.availibility[avail] && self.staff_type == staff_type && self.rank <= max_rank }
    }
}


pub struct WeaponDatabase {
    pub generic_weapons: Vec<WeaponData>,
    pub magic_weapons: Vec<WeaponData>,
    pub weapon_list: Vec<WeaponData>,
    pub bullet_list: Vec<WeaponData>,
    pub staff_list: Vec<StaffData>,
    pub dragonstones: Vec<DragonStoneData>,
    pub effective_sids: Vec<i32>,
    pub extra_items: Vec<i32>,
    pub base_might: [[u8; 9]; 6],   
}

impl WeaponDatabase {
    pub fn new() -> Self {
        WeaponDatabase {
            generic_weapons: Vec::new(),
            magic_weapons: Vec::new(),
            weapon_list: Vec::new(),
            bullet_list: Vec::new(),
            staff_list: Vec::new(),
            dragonstones: Vec::new(),
            extra_items: Vec::new(),
            effective_sids: SkillData::get_list().unwrap().iter()
                .filter(|x| x.efficacy_value > 1 && x.efficacy !=  0).map(|x| x.parent.hash).collect(),
            base_might: [[0; 9]; 6],
        }
    }
    pub fn initialize(&mut self) {
        let item_list = ItemData::get_list().unwrap();
        for x in 3..item_list.len() {
            let item = &item_list[x];
            if !is_vaild_weapon(item) {
                if item.usetype == 42 || item.usetype == 28 || item.usetype == 21 {
                    self.extra_items.push(item.parent.hash);
                }
                if item.kind == 7 {

                }
            }
            self.try_add_weapon(item);
        }
    }
    pub fn try_add_weapon(&mut self, item: &ItemData) {
        let effectiveness = self.check_effectiveness(item);
        if item.icon.is_none() { return; }
        if item.kind == 7 && item.flag.value & 128 == 0{  //Staff
            self.staff_list.push(StaffData::new(item));
            return;
        }
        else if item.kind < 9 {
            let flag = item.flag.value;
            if item.kind != 6 && flag & 65536 != 0 { self.magic_weapons.push(WeaponData::new(item, effectiveness)); }
            self.weapon_list.push(WeaponData::new(item, effectiveness));
            if is_generic(item) {
                self.generic_weapons.push(WeaponData::new(item, effectiveness)); 
                let level = item.get_weapon_level();
                if level < 5 && flag & 65536 == 0 && item.get_equip_skills().find_sid("SID_スマッシュ").is_none() {
                    if self.base_might[level as usize ][ item.kind as usize ] < item.power  {
                        self.base_might[level as usize ][ item.kind as usize ] = item.power;
                    }
                }
            }
        }
        else if item.kind == 9 && item.icon.is_some_and(|s| !s.to_string().contains("Sombre")) {
            if item.flag.value & 0x4000000 != 0 && item.hit > 0 { self.dragonstones.push(DragonStoneData::new(item));  }
            else if item.flag.value & 0x8000000 != 0 { self.bullet_list.push(WeaponData::new(item, effectiveness)); }
        }
    }

    pub fn get_new_weapon(&self, item: &UnitItem, new_type: i32, enemy: bool) -> Option<&'static ItemData> {
        let kind = new_type as u8;
        let generic = self.weapon_list.iter().find(|&x| x.item_index == item.item.parent.index);
        let gen = self.generic_weapons.iter().find(|&x| x.item_index == item.item.parent.index);
        let min_rank = if gen.is_some() { get_min_rank() } else { 1 };
        if generic.is_some() {
            let g_weapon = generic.unwrap();
            let possible_weapons: Vec<&WeaponData> = self.weapon_list.iter().filter(|&x|
                g_weapon.can_replace(x, kind, enemy) &&
                x.rank >= min_rank
            ).collect();
            if possible_weapons.len() == 0 { return None;   }
            let rng = Random::get_system();
            let selection = rng.get_value(possible_weapons.len() as i32) as usize;
            let index = possible_weapons[selection].item_index;
            ItemData::try_index_get(index)
        }
        else { None }
    }

    pub fn get_generic_weapon(&self, new_type: i32, rank: i32) -> Option<&'static ItemData> {
        let kind = new_type;
        let possible_weapons: Vec<&WeaponData> = self.generic_weapons.iter().filter(|&x|
            x.rank == rank as u8 && 
            x.weapon_type == kind as u8).collect();

        if possible_weapons.len() == 1 {
            return ItemData::try_index_get(possible_weapons[0].item_index);
        }
        if possible_weapons.len() > 1 {
            let rng = Random::get_system();
            let selection = rng.get_value(possible_weapons.len() as i32) as usize;
            let index = possible_weapons[selection].item_index;
            ItemData::try_index_get(index)
        }
        else {  None }
    }

    pub fn get_range_melee(&self, new_type: i32, max_rank: i32) -> Option<&'static ItemData> {
        let possible_weapons: Vec<&WeaponData> = self.generic_weapons.iter().filter(|&x|
            x.rank <= max_rank as u8 && 
            x.weapon_type == new_type as u8 && (x.flag & 4 != 0) == true).collect();

        if possible_weapons.len() == 1 { ItemData::try_index_get(possible_weapons[0].item_index) }
        else if possible_weapons.len() > 1 {
            let rng = Random::get_system();
            let selection = rng.get_value(possible_weapons.len() as i32) as usize;
            let index = possible_weapons[selection].item_index;
            ItemData::try_index_get(index)
        }
        else { None }
    }

    pub fn get_staff(&self, level: i32, staff_type: i32, job_rank: i32, enemy: bool) -> Option<&'static ItemData> {
        let possible_staffs: Vec<&StaffData> = self.staff_list.iter()
            .filter(|&x| x.can_add(level, staff_type as u8, job_rank as u8, enemy)).collect();

        if possible_staffs.len() >= 1 {
            let rng = Random::get_system();
            let selection = rng.get_value(possible_staffs.len() as i32) as usize;
            let index = possible_staffs[selection].item_index;
            ItemData::try_index_get(index)
        }
        else { None }
    }

    pub fn get_tome(&self, job_rank: i32, total_level: i32, enemy: bool) -> Option<&'static ItemData> {
        let non_basic = total_level >= 10;
        let rank_level = 
        if total_level < 14 { 1 }
        else if total_level < 21 { 2 }
        else if total_level < 26 { 3 }
        else if total_level < 31 { 4 }
        else { 5 };
        let rank = min(rank_level, job_rank) as u8;

        let mut possible_weapons: Vec<_> = 
            self.weapon_list.iter()
                .filter(|&x|{
                    x.weapon_type == 6 && x.rank == rank &&
                    ((non_basic == (x.flag & 232 != 0)) || non_basic) &&
                    ((enemy == (x.flag & 32 != 0)) || enemy)
                })
                .map(|x| x.item_index)
                .collect();

        utils::get_random_element(&mut possible_weapons, Random::get_system()).and_then(|&index| ItemData::try_index_get(index))
    }
    pub fn get_dragon_stone(&self, is_enemy: bool) -> Option<&'static ItemData> {
        let possible_weapons: Vec<&DragonStoneData> = self.dragonstones.iter().filter(|&x|  (is_enemy == x.is_enemy_only) || is_enemy).collect();
        if possible_weapons.len() > 1 {
            let rng = Random::get_system();
            let selection = rng.get_value(possible_weapons.len() as i32) as usize;
            ItemData::try_index_get(possible_weapons[selection].item_index)
        }
        else { None }
    }

    pub fn get_random_weapon(&self, kind: i32, rank: i32, enemy: bool) -> Option<&'static ItemData> {
        let weapon = kind  as u8;
        let possible_weapons: Vec<&WeaponData>  = self.weapon_list.iter().filter(|x|
            x.weapon_type == weapon && ( (x.flag & 32 != 0 && enemy) || (!x.flag & 32 != 0 && (rank == -1 || (x.rank == rank as u8))))
        )
        .collect();
        if possible_weapons.len() == 0 {
            println!("no weapons of kind: {} and enemy: {}", kind, enemy);
            return None;
        }
        let rng = Random::get_system();
        let selection = rng.get_value(possible_weapons.len() as i32) as usize;
        let index = possible_weapons[selection].item_index;
        ItemData::try_index_get(index)
    }
    pub fn get_simple_replacement(&self, item: &ItemData, weapon_mask: i32, weapon_levels: &Array<i32>) -> Option<&'static ItemData> {
        if weapon_mask & (1 << item.kind) != 0 && item.get_weapon_level() <= weapon_levels[item.kind as usize] { return None; }
        let is_rare = item.flag.value & 32 != 0;
        let mut weapon_order = [0; 4];
        let mut search_mask = weapon_mask;
        for w in 0..4 {
            let mut index = 0;
            let mut level = 0;
            for x in 1..9 {
                if search_mask & (1 << x) != 0 {
                    if level < weapon_levels[x as usize] {
                        index = x;
                        level = weapon_levels[x as usize];
                    }
                }
            }
            if index != 0 {
                weapon_order[w] = index;
                search_mask ^= 1 << index;
            }
        }
        if let Some(weapon) = self.weapon_list.iter().find(|x| x.item_index == item.parent.index) {
            for kind in weapon_order {
                let conditions = if weapon.weapon_type == 4 { 0 } else { 32 } | 538 |
                    if weapon.flag & 4 != 0 && (kind != 1 && kind != 8) { 4 } else { 0 };
                let target_conditions = if weapon.flag & 512 != 0 && weapon.weapon_type != 8 { 606 } else {  94 };
                let mut search_rank = clamp_value(item.get_weapon_level(), 1,  weapon_levels[kind as usize]) as u8;
                while search_rank != 0 {
                    let dmight = self.base_might[search_rank as usize][item.kind as usize] -  self.base_might[search_rank as usize][kind as usize];
                    let mut selection: Vec<_> = self.weapon_list.iter()
                        .filter(|w| {
                            ((weapon.flag & conditions) == (w.flag & target_conditions)) &&
                                ((w.might - weapon.might) * (w.might - weapon.might) <= 2*(dmight + 1) * dmight) &&
                                search_rank == w.rank && w.flag & 256 == 0 &&
                                kind == w.weapon_type &&
                                ((is_rare == (w.flag & 32 != 0)) || is_rare)
                        }
                        ).collect();
                    if selection.len() >= 1 {
                        if let Some(new_weapon) = utils::get_random_element(&mut selection, Random::get_game()) {
                            println!("Simple Replacement: {} to {}", Mess::get_name(item.iid), Mess::get_name(ItemData::try_index_get(new_weapon.item_index).unwrap().iid));
                            return ItemData::try_index_get(new_weapon.item_index);
                        }
                    }
                    let mut selection: Vec<_> = self.weapon_list.iter()
                        .filter(|w|
                            w.flag & 66 == 0 && ((weapon.flag & 512 != 0) == (w.flag & 512 != 0)) &&
                            search_rank == w.rank &&
                            kind == w.weapon_type &&
                            w.flag & 256 == 0 &&
                            ((is_rare == (w.flag & 32 != 0)) || is_rare )
                        ).collect();
                    if selection.len() >= 1 {
                        if let Some(new_weapon) = utils::get_random_element(&mut selection, Random::get_game()) {
                            // println!("Simple Replacement 2: {} to {}", Mess::get_name(item.iid), Mess::get_name(ItemData::try_index_get(new_weapon.item_index).unwrap().iid));
                            return ItemData::try_index_get(new_weapon.item_index);
                        }
                    }
                    search_rank -= 1;
                }
            }
        }
        None
    }
    pub fn check_effectiveness(&self, item: &ItemData) -> bool {
        let equipped_skills = item.get_equip_skills();
        equipped_skills.iter()
            .any(|x| 
                x.get_skill().is_some_and(|skill|{
                    let hash = skill.parent.hash;
                    self.effective_sids.iter().any(|&effective_skill_hash| effective_skill_hash == hash)
                }
            )
        )
    }
    pub fn get_additional_weapon(&self, item: &ItemData) -> Option<&'static ItemData> {
        if let Some(weapon) = self.weapon_list.iter().find(|x| x.item_index == item.parent.index) {
            let mut search_rank = weapon.rank;
            while search_rank > 1 {
                let mut selection: Vec<_> = self.weapon_list.iter()
                    .filter(|w|
                        w.item_index != item.parent.index &&
                        (w.flag & 78) ^ (weapon.flag & 78) != 0 &&
                        search_rank == w.rank &&
                        weapon.weapon_type == w.weapon_type && 
                        w.flag & 32 == 0
                    ).collect();
                if selection.len() > 1 {
                    if let Some(weapon) = utils::get_random_element(&mut selection, Random::get_game()) {
                        return ItemData::try_index_get(weapon.item_index);
                    }
                }
                search_rank -= 1;
            }
        }
        None
    }
}


pub fn is_generic(item: &ItemData) -> bool { item.price > 100 && item.flag.value & 135 == 0 && item.equip_condition.is_none()  }
pub fn is_vaild_weapon(item: &ItemData) -> bool {
    let iid = item.iid.to_string(); 
    if item.icon.is_none() { return false; }
    if item.flag.value & 128 != 0  { return false; }
    if Mess::get(item.name).to_string().len() <= 1 { return false;}
    if item.kind == 0 || item.kind > 9 { return false; }
    if iid == "IID_メティオ" || iid == "IID_ミセリコルデ" || iid == "IID_リベラシオン" { return false; }    // No Meteor / Liberation / Misercode
    ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == item.parent.index).is_none()
}

pub fn get_min_rank() -> u8 {
    let story_chapter = crate::continuous::get_story_chapters_completed();
    let continous = DVCVariables::is_random_map();
    if DVCVariables::is_main_chapter_complete(25) || (continous && story_chapter >= 25 ) { 5 }
    else if DVCVariables::is_main_chapter_complete(17) || (continous && story_chapter >= 16 && DVCVariables::is_main_chapter_complete(11)) { 3 }
    else if DVCVariables::is_main_chapter_complete(6) || (continous && story_chapter >= 6)  { 2 }
    else { 1 }
}

pub fn get_magic_staff() -> usize {
    let story_chapter = crate::continuous::get_story_chapters_completed();
    let continous = DVCVariables::is_random_map();
    if DVCVariables::is_main_chapter_complete(21) { 4 }
    else if DVCVariables::is_main_chapter_complete(17) || (continous && story_chapter >= 16 && DVCVariables::is_main_chapter_complete(11)) { 3}
    else if DVCVariables::is_main_chapter_complete(11) { 2 }
    else if DVCVariables::is_main_chapter_complete(6) || (continous && story_chapter >= 6) { 1}
    else { 0 }
}

pub fn get_magic_staff_by_level(level: i32) -> usize {
    if level > 32 { 4}
    else if level > 25 { 3}
    else if level > 15 { 2}
    else if level > 10 { 1}
    else { 0 }
}