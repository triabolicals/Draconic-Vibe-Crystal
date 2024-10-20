use utils::str_contains;
use std::sync::Mutex;
use super::*;
const EFFECTIVE_SIDS : &[&str] = &["SID_馬特効", "SID_鎧特効", "SID_飛行特効", "SID_竜特効", "SID_邪竜特効", "SID_異形特効" ];

pub static WEAPONDATA: Mutex<WeaponDatabase> =  Mutex::new(
    WeaponDatabase {
        generic_weapons: Vec::new(),
        magic_weapons: Vec::new(),
        weapon_list: Vec::new(),
        bullet_list: Vec::new(),
        staff_list: Vec::new(),
        dragonstones: Vec::new(),
        intialize: false,
    }
);

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
    pub is_magic: bool,
    pub is_smash: bool,
    pub is_range: bool,
    pub is_crit: bool,
    pub is_slim: bool,
    pub is_rare: bool,
    pub is_effective: bool,
}

impl WeaponData {
    pub fn new(item: &ItemData) -> Self {
        let flags = item.get_flag().value;
        let magic = if item.attr == 2 { true }
            else if item.attr == 1 && flags & 65536 != 0 { true }
            else { false }; 

        let e_skills = item.get_equip_skills();
        let smash = e_skills.find_sid("SID_スマッシュ".into()).is_some();
        let slim = str_contains(item.iid, "IID_ほそみの");
        let crit = item.critical > 15;
        let range = 
            if magic && item.range_o > 2 { true }
            else if !magic && item.range_o > 1 { true }
            else { false };

        Self {
            item_index: item.parent.index,
            weapon_type: item.kind as u8 - 1,
            might: item.power,
            weight: item.weight,
            hit: item.hit,
            crit: item.critical,
            avo: item.avoid,
            secure: item.secure,
            rank: item.get_weapon_level() as u8,
            is_magic: magic,
            is_smash: smash,
            is_slim: slim,
            is_crit: crit,
            is_range: range,
            is_effective: check_effectiveness(item),
            is_rare: flags & 3 != 0,
        }
    }

    pub fn can_replace(&self, item2: &WeaponData, kind: u8, enemy: bool) -> bool {
        if item2.weapon_type != kind { return false; }
        if self.is_slim == item2.is_slim { return true; }
        if enemy && item2.is_rare { return false; }
        if item2.rank == self.rank { return true; }
        if item2.is_rare && !GameVariableManager::get_bool("G_Cleared_M007") { return false; }
        else {
            let might_diff = item2.might as i8 - self.might as i8;
            if might_diff > 15 || might_diff < -2 { return false;}
        }
        return true;
    }

    pub fn is_valid_tome(&self, tome_rank: i32, enemy: bool) -> bool {
        if self.weapon_type != 6 { return false; }
        if enemy && self.is_rare { return true; }
        return self.rank == tome_rank as u8;
    }

}

pub struct StaffData {
    pub item_index: i32,
    pub staff_type: u8,
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
                    if item.range_o < 9 {
                        avail[1] = true;
                        avail[2] = true;
                    }
                    else {  // Longer Range Physic
                        avail[3] = true;
                        avail[4] = true;
                    }
                }
                else if item.power < 11 {
                    avail[0] = true;
                    avail[1] = true;
                }
                else if item.power < 21 {
                    avail[1] = true;
                    avail[2] = true;
                }
                else {
                    avail[3] = true;
                    avail[4] = true;
                }
            },
            3 => {
                staff_kind = 1; //Fortify
                avail[3] = true;
                avail[4] = true;
            },
            5 => {  //Warp
                staff_kind = 2;
                avail[2] = true;
                avail[3] = true;
                avail[4] = true;
            },
            6 => {  //Rescue
                staff_kind = 3;
                avail[2] = true;
                avail[3] = true;
                avail[4] = true;
            },
            9|10 => {  // Freeze
                staff_kind = 2;
                avail[1] = true;
                avail[2] = true;
                avail[3] = true;
                avail[4] = true;
            },
            15|27 => {
                staff_kind = 2;
                avail[3] = true;
                avail[4] = true;
            },
            29 => { // Fracture
                staff_kind = 2;
                avail[0] = true;
                avail[1] = true;
                avail[2] = true;
                avail[3] = true;
                avail[4] = true;
            },
            _ => { staff_kind = 0; }
        }
        Self {
            item_index: item.parent.index,
            staff_type: staff_kind as u8,
            rank: item.get_weapon_level() as u8,
            availibility: avail,
        }
    }
    pub fn can_add(&self, staff_type: u8, max_rank: u8) -> bool {
        let avail = get_magic_staff();
        return self.availibility[avail] && self.staff_type == staff_type && self.rank <= max_rank;
    }
}


pub struct WeaponDatabase {
    pub generic_weapons: Vec<WeaponData>,
    pub magic_weapons: Vec<WeaponData>,
    pub weapon_list: Vec<WeaponData>,
    pub bullet_list: Vec<WeaponData>,
    pub staff_list: Vec<StaffData>,
    pub dragonstones: Vec<i32>,
    intialize: bool, 
}

impl WeaponDatabase {
    pub fn intitalize(&mut self) {
        if self.intialize { return; }
        let item_list = ItemData::get_list().unwrap();
        for x in 3..item_list.len() {
            let item = &item_list[x];
            if !is_vaild_weapon(item) { continue; }
            self.try_add_weapon(item);
        }
        self.intialize = true;
        println!("Total of {} weapons in the database.", self.weapon_list.len());
        println!("Total of {} staffs in the database.", self.staff_list.len());
        println!("Total of {} dragonstones in database", self.dragonstones.len());
    }

    pub fn try_add_weapon(&mut self, item: &ItemData) {
        if is_generic(item) { self.generic_weapons.push(WeaponData::new(item)); }
        if item.kind == 7 { //Staff
            self.staff_list.push(StaffData::new(item));
            return;
        }
        if str_contains(item.name, "MIID_Bullet") {
            self.bullet_list.push(WeaponData::new(item));
            return;
        }
        else if item.kind < 9 {
            let flag = item.flag.value;
            if item.kind < 5 && flag & 65536 != 0 { self.magic_weapons.push(WeaponData::new(item)); }
            self.weapon_list.push(WeaponData::new(item));
        }
        else if item.kind == 9 {
            let icon = item.icon.unwrap();
            if let Some(equip) = item.equip_condition {
                if equip.get_string().unwrap() == "SID_竜石装備" && !str_contains(icon, "Sombre") {
                    self.dragonstones.push(item.parent.index);
                }
            }
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
            println!("Weapons of Type: {} = {}", new_type, possible_weapons.len());
            let rng = Random::get_system();
            let mut index;
            let selection = rng.get_value(possible_weapons.len() as i32) as usize;
            index = possible_weapons[selection].item_index;
            return ItemData::try_index_get(index);
        }
        return None;
    }

    pub fn get_generic_weapon(&self, new_type: i32, rank: i32) -> Option<&'static ItemData> {
        let possible_weapons: Vec<&WeaponData> = self.generic_weapons.iter().filter(|&x|
            x.rank == rank as u8 && 
            x.weapon_type == new_type as u8).collect();

        if possible_weapons.len() == 1 {
            return ItemData::try_index_get(possible_weapons[0].item_index);
        }
        if possible_weapons.len() > 1 {
            let rng = Random::get_system();
            let selection = rng.get_value(possible_weapons.len() as i32) as usize;
            let index = possible_weapons[selection].item_index;
            return ItemData::try_index_get(index);
        }
        else { return None; }
    }

    pub fn get_range_melee(&self, new_type: i32, max_rank: i32) -> Option<&'static ItemData> {
        let possible_weapons: Vec<&WeaponData> = self.generic_weapons.iter().filter(|&x|
            x.rank <= max_rank as u8 && 
            x.weapon_type == new_type as u8 &&
            x.is_range == true).collect();

        if possible_weapons.len() == 1 {
            return ItemData::try_index_get(possible_weapons[0].item_index);
        }
        if possible_weapons.len() > 1 {
            let rng = Random::get_system();
            let selection = rng.get_value(possible_weapons.len() as i32) as usize;
            let index = possible_weapons[selection].item_index;
            return ItemData::try_index_get(index);
        }
        return None;
    }

    pub fn get_staff(&self, staff_type: i32, job_rank: i32) -> Option<&'static ItemData> {
        let possible_staffs: Vec<&StaffData> = self.staff_list.iter().filter(|&x| x.can_add(staff_type as u8, job_rank as u8)).collect();
        if possible_staffs.len() == 1 { return ItemData::try_index_get(possible_staffs[0].item_index);   }
        if possible_staffs.len() > 1 {
            let rng = Random::get_system();
            let selection = rng.get_value(possible_staffs.len() as i32) as usize;
            let index = possible_staffs[selection].item_index;
            return ItemData::try_index_get(index);
        }
        else { return None; }
    }

    pub fn get_tome(&self, job_rank: i32, enemy: bool) -> Option<&'static ItemData> {
        let magic_level = get_magic_staff();
        let mut tome_rank = 
            match magic_level {
                0|1 => { 2 }
                2 => { 3 }
                3 => { 4 }
                _ => { job_rank }
            };
        tome_rank = if tome_rank > job_rank { job_rank } else { tome_rank };

        let possible_weapons: Vec<&WeaponData> = self.weapon_list.iter().filter(|&x| x.is_valid_tome(tome_rank, enemy) ).collect();
        if possible_weapons.len() == 1 { return ItemData::try_index_get(possible_weapons[0].item_index);   }
        if possible_weapons.len() > 1 {
            let rng = Random::get_system();
            let selection = rng.get_value(possible_weapons.len() as i32) as usize;
            let index = possible_weapons[selection].item_index;
            return ItemData::try_index_get(index);
        }
        else { return None; }
    }
    pub fn get_dragon_stone(&self) -> Option<&'static ItemData> {
        if self.dragonstones.len() == 0 { return None; }
        let rng = Random::get_system();
        let selection = rng.get_value(self.dragonstones.len() as i32) as usize;
        return ItemData::try_index_get(self.dragonstones[selection]);
    }

    pub fn get_random_weapon(&self, kind: i32) -> Option<&'static ItemData> {
        let weapon = ( kind - 1 ) as u8;
        let possible_weapons: Vec<&WeaponData>  = self.weapon_list.iter().filter(|x| x.weapon_type == weapon).collect();
        if possible_weapons.len() == 0 {
            return None;
        }
        let rng = Random::get_system();
        let selection = rng.get_value(possible_weapons.len() as i32) as usize;
        let index = possible_weapons[selection].item_index;
        return ItemData::try_index_get(index);
    }


}


pub fn is_generic(item: &ItemData) -> bool {
    let iid = item.iid;
    // check if Slim/Iron/Steel/
    if item.kind == 6 {
        return str_contains(iid, "IID_ファイアー") || str_contains(iid, "IID_エルファイアー") || str_contains(iid, "IID_ボルガノン");
    }
    return str_contains(iid, "IID_ほそみの") || str_contains(iid, "IID_鉄の") || str_contains(iid, "IID_鋼の") || str_contains(iid, "IID_銀の") || str_contains(iid, "IID_勇者の");
}
pub fn is_vaild_weapon(item: &ItemData) -> bool {
    if item.iid.get_string().unwrap() == "IID_メティオ" { return false; }
    if !item.is_weapon() && item.kind != 7 { return false; }
    if item.icon.is_none() { return false; }
    if item.kind == 0 || item.kind > 9 { return false; }
    let flags = item.get_flag().value;
    if flags & 128 != 0  { return false; }
    return enums::ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == item.parent.index).is_none();
}

pub fn check_effectiveness(item: &ItemData) -> bool {
    let equipped_skills = item.get_equip_skills();
    for sid in EFFECTIVE_SIDS {
        if equipped_skills.find_sid(sid.into()).is_some() {
            return true;
        }
    }
    return false; 
}
pub fn get_min_rank() -> u8 {
    if GameVariableManager::get_bool("G_Cleared_M025") { return 5;}
    if GameVariableManager::get_bool("G_Cleared_M017") { return 3;}
    if GameVariableManager::get_bool("G_Cleared_M006") { return 2;}
    return 1;
}

pub fn get_magic_staff() -> usize {
    if GameVariableManager::get_bool("G_Cleared_M021") { return 4;}
    if GameVariableManager::get_bool("G_Cleared_M017") { return 3;}
    if GameVariableManager::get_bool("G_Cleared_M011") { return 2;}
    if GameVariableManager::get_bool("G_Cleared_M006") { return 1;}
    return 0;
}