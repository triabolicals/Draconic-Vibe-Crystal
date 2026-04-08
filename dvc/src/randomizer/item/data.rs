use std::collections::HashMap;
use std::hash::Hash;
use utils::{min};
use bitflags::{bitflags, Flags};
use engage::transporter::Transporter;
use crate::continuous::get_story_chapters_completed;
use crate::randomizer::Randomizer;
use super::*;

const GENERICS: [&str; 36] = [
    // Sword, Lance, Axe, Bow, Dagger, Tome, Arts, Bullet, Stone
    "IID_鉄の剣", "IID_鉄の槍", "IID_鉄の斧",	"IID_鉄の弓", "IID_鉄のナイフ", "IID_エルファイアー", "IID_鉄身の法", "IID_弾_物理", "IID_邪竜石",
    "IID_鋼の剣", "IID_鋼の槍", "IID_鋼の斧",	"IID_鋼の弓", "IID_鋼のナイフ", "IID_エルウィンド", "IID_鋼身の法", "IID_弾_物理", "IID_邪竜石",
    "IID_銀の剣", "IID_銀の槍", "IID_銀の斧",	"IID_銀の弓", "IID_銀のナイフ", "IID_ボルガノン", "IID_銀身の法", "IID_弾_物理_強", "IID_邪竜石",
    "IID_勇者の剣", "IID_勇者の槍",	"IID_勇者の斧",	"IID_勇者の弓",	"IID_ペシュカド",	"IID_ボルガノン",	"IID_閃進の法", "IID_弾_魔法_強", "IID_邪竜石",
];


bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WeaponDataFlag: i32 {
        const EnemyOnly = 1 << 0;
        const MagicWeapon = 1 << 1;
        const Ranged = 1 << 2;
        const SlimWeapon = 1 << 3;
        const Smash = 1 << 4;
        const Rare = 1 << 5;
        const Effective = 1 << 6;
        const Critical = 1 << 7;
        const Usable = 1 << 8; // You can eat it
        const Surge = 1 << 9;
        const NoFollowUp = 1 << 10;
        const Generic = 1 << 11;
    }
}
impl WeaponDataFlag {
    pub fn from_item(data: &ItemData) -> Self {
        let e_skills = &data.equip_skills;
        let item_flag = data.flag.value;
        let weapon_mask = 1 << data.kind;
        let ranged = (weapon_mask & 112 != 0 && data.range_o > 2) || (weapon_mask & 14 != 0 && data.range_o > 1);
        let effectiveness = e_skills.iter().any(|k| k.get_skill().is_some_and(|skill| skill.efficacy > 0 && skill.efficacy_value > 2 )) && data.kind != 4;
        let mut flag = Self::from_bits(0).unwrap();
        flag.set(Self::MagicWeapon, data.attr == 1 && item_flag & 65536 != 0 && data.kind < 6);
        flag.set(Self::Smash,  e_skills.find_sid("SID_スマッシュ").is_some());
        flag.set(Self::Critical, data.critical >= 15);
        flag.set(Self::Surge, data.hit == 255 || data.hit == -1);
        flag.set(Self::NoFollowUp, e_skills.find_sid("SID_追撃不可").is_some());
        flag.set(Self::Ranged, ranged);
        flag.set(Self::Effective, effectiveness);
        flag.set(Self::Usable, item_flag & 4 != 0);
        flag.set(Self::Rare, data.equip_condition.is_some_and(|k| SkillData::get(k).is_some()) || data.flag.value & 1 != 0);
        flag.set(Self::EnemyOnly, item_flag & 18 != 0 );
        flag.set(Self::SlimWeapon, data.secure >= 15 && data.get_weapon_level() < 2);
        let iid = data.iid.to_string();
        flag.set(Self::Generic, GENERICS.iter().any(|generic_iid| generic_iid == &iid));
        flag
    }
    pub fn can_use(&self, enemy: bool, chapter_completed: i32) -> bool {
        if !enemy && self.contains(Self::EnemyOnly) { return false; }
        let custom_inventory = DVCVariables::UnitInventory.get_value() & if enemy { 2 } else { 1 } != 0;
        let mut flags = Self::all();
        if custom_inventory {
            flags.remove(Self::Effective);
            flags.remove(Self::Ranged);
            flags.remove(Self::Usable);
            flags.remove(Self::Critical);
            flags.remove(Self::Surge);
            flags.remove(Self::NoFollowUp);
            flags.remove(Self::SlimWeapon);
        }
        if chapter_completed > 20 && (enemy || custom_inventory) { flags.remove(Self::Rare) }
        if chapter_completed > 10 && custom_inventory && enemy { flags.remove(Self::EnemyOnly) }
        if chapter_completed > 6 {
            flags.remove(Self::Smash);
            flags.remove(Self::Surge);
            flags.remove(Self::MagicWeapon);
            flags.remove(Self::NoFollowUp);
        }
        if chapter_completed > 5 {
            flags.remove(Self::Usable);
            flags.remove(Self::Critical);
        }
        if chapter_completed > 3 {
            flags.remove(Self::Effective);
            flags.remove(Self::Ranged);
            flags.remove(Self::SlimWeapon);
        }
        !self.intersects(flags) || self.contains(Self::Generic)
    }
    pub fn is_normal(&self) -> bool {
        !self.contains(Self::EnemyOnly) && !self.contains(Self::Rare) && !self.contains(Self::Smash)
        && !self.contains(Self::Ranged) && !self.contains(Self::NoFollowUp) && !self.contains(Self::SlimWeapon)
    }
}
#[derive(Clone)]
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
    pub flag: WeaponDataFlag,
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
    pub fn new(item: &ItemData) -> Self {
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
            flag: WeaponDataFlag::from_item(&item),
        }
    }
    pub fn can_equip(&self, unit: Option<&Unit>, enemy: bool) -> bool {
        if let Some(item) = ItemData::try_index_get(self.item_index) {
            if let Some(unit) = unit { enemy || unit.can_equip_item(item, false, true) }
            else { true }
        }
        else { false }
    }
}

pub struct StaffData {
    pub item_index: i32,
    pub staff_type: u8,
    pub rare: bool,
    pub rank: u8,
    pub availability: [bool; 5],
}

impl StaffData {
    pub fn new(item: &ItemData) -> Self {
        let mut avail: [bool; 5] = [false; 5];
        let staff_kind; 
        match item.use_type {
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
            rare: item.flag.value & 3 != 0 || item.use_type == 27 || item.use_type == 7,
            availability: avail,
        }
    }
    pub fn can_add(&self, level: i32, staff_type: u8, max_rank: u8, enemy: bool) -> bool {
        let avail = get_magic_staff_by_level(level);
        if self.rare && !enemy { false }
        else { self.availability[avail] && self.staff_type == staff_type && self.rank <= max_rank }
    }
}

pub struct WeaponDatabase {
    pub magic_weapons: Vec<WeaponData>,
    pub weapon_list: HashMap<i32, Vec<WeaponData>>,
    pub staff_list: Vec<StaffData>,
    pub dragonstones: Vec<DragonStoneData>,
    pub extra_items: Vec<i32>,
}

impl WeaponDatabase {
    pub fn init() -> Self {
        let mut db = WeaponDatabase {
            magic_weapons: Vec::new(),
            weapon_list: [1,2,3,4,5,6,8,9, 10].into_iter().map(|x| (x, vec![])).collect(),
            staff_list: Vec::new(),
            dragonstones: Vec::new(),
            extra_items: Vec::new(),
        };
        db.initialize();
        // println!("Weapon DB initialized.");
        db
    }
    pub fn initialize(&mut self) {
        let item_list = ItemData::get_list().unwrap();
        for x in 3..item_list.len() {
            let item = &item_list[x];
            if !is_vaild_weapon(item) {
                if item.use_type == 42 || item.use_type == 28 || item.use_type == 21 {
                    self.extra_items.push(item.parent.hash);
                }
                if item.kind == 7 { self.staff_list.push(StaffData::new(item)); }
            }
            self.try_add_weapon(item);
        }
    }
    pub fn try_add_weapon(&mut self, item: &ItemData) {
        if item.icon.is_none() { return; }
        if item.kind == 7 && item.flag.value & 128 == 0 {  //Staff
            self.staff_list.push(StaffData::new(item));
        }
        else if item.kind < 9 || (item.kind == 9 && item.hit > 0 && item.icon.is_some_and(|s| !s.to_string().contains("Sombre"))) {
            let flag = item.flag.value;

            let kind = if item.kind == 9 && item.is_dragon() { 10 } else { item.kind as i32 };

            if let Some(mut list) = self.weapon_list.get_mut(&kind) {
                let mut data = WeaponData::new(item);
                data.weapon_type = kind as u8;
                if item.kind != 6 && flag & 65536 != 0 {
                    self.magic_weapons.push(data.clone());
                }
                list.push(data);
            }
        }
        if item.kind == 9 && item.flag.value & 0x4000000 != 0 && item.icon.is_some_and(|s| !s.to_string().contains("Sombre")) && item.hit > 0 {
            self.dragonstones.push(DragonStoneData::new(item));
        }
    }
    /// Iron/Steel/Silver rank: D (1), C (2), B (3), A (4)
    pub fn get_generic_weapon(&self, new_type: i32, rank: i32) -> Option<&'static ItemData> {
        if new_type == 0 { return None; }
        let type_index =
            match new_type {
                8 => 6,
                9|10 => new_type - 2,
                _ => new_type - 1,
            } as usize;

        // converting to the &str array index from kind/weapon level values
        let rank = if rank < 2 { 0 } else if rank > 4 { 3 } else { rank - 1 } as usize;
        let generic_index = 9*rank + type_index;
        println!("Type: {} -> {}, {}", new_type, type_index, generic_index);
        GENERICS.get(generic_index).and_then(|iid| ItemData::get(iid))
    }

    pub fn get_staff(&self, level: i32, staff_type: i32, job_rank: i32, enemy: bool) -> Option<&'static ItemData> {
        let possible_staffs: Vec<&StaffData> = self.staff_list.iter()
            .filter(|&x| x.can_add(level, staff_type as u8, job_rank as u8, enemy)).collect();

        possible_staffs.get_random_element(Random::get_system())
            .and_then(|i| ItemData::try_index_get(i.item_index))
    }

    pub fn get_tome(&self, job_rank: i32, total_level: i32, enemy: bool) -> Option<&'static ItemData> {
        let rank_level =
        if total_level < 14 { 1 }
        else if total_level < 21 { 2 }
        else if total_level < 26 { 3 }
        else if total_level < 31 { 4 }
        else { 5 };
        let rank = min(rank_level, job_rank) as u8;
        let chapter_completed = get_story_chapters_completed();
        self.weapon_list
            .get(&6)
            .and_then(|list|{
                let pool: Vec<i32> = list.iter().filter(|&x| x.flag.can_use(enemy,chapter_completed) && rank == x.rank).map(|x| x.item_index).collect();
                pool.get_random_element(Random::get_system()).and_then(|&index| ItemData::try_index_get(index))
            })
    }
    pub fn get_dragon_stone(&self, is_enemy: bool) -> Option<&'static ItemData> {
        let possible_weapons: Vec<&DragonStoneData> = self.dragonstones.iter().filter(|&x|  (is_enemy == x.is_enemy_only) || is_enemy).collect();
        possible_weapons.get_random_element(Random::get_system())
            .and_then(|d| ItemData::try_index_get(d.item_index))
    }

    pub fn get_random_weapon(&self, unit: Option<&Unit>, kind: i32, start_rank: i32, enemy: bool) -> Option<&'static ItemData> {
        if unit.is_none() {
            self.weapon_list
                .get(&kind)
                .and_then(|list| list.get_random_element(Random::get_system()).and_then(|data| ItemData::try_index_get(data.item_index)))
        }
        else {
            let chapter_completed = get_story_chapters_completed();
            let mut search_rank = if start_rank == -1 { 6 } else { start_rank as u8 };
            let mut lower_rank = if search_rank > 2 { search_rank - 1 } else { search_rank };
            if let Some(list) = self.weapon_list.get(&kind) {
                let rng = Random::get_system();
                let pool: Vec<_> = list.iter().filter(|x| x.flag.can_use(enemy, chapter_completed) && x.rank <= search_rank)
                    .map(|x| (x.item_index, x.rank))
                    .collect();
                
                while search_rank > 0 {
                    if let Some(index) = pool.get_filter(rng, |f| f.1 == search_rank || f.1 >= lower_rank){ return ItemData::try_index_get(index.0); }
                    search_rank -= 1;
                    if lower_rank > 1 { lower_rank -= 1; }
                }
            }
            None
        }
    }
    fn get_closest_match(&self, item: &ItemData, avail_weapons: &Vec<(i32, &WeaponData)>) -> Option<&'static ItemData> {
        let rank = item.get_weapon_level() as u8;
        let mut replace = None;
        // println!("Simple Replacement for: {}", Mess::get(item.name));
        let current_item_flags = WeaponDataFlag::from_item(item);
        if item.kind != 6 && current_item_flags.contains(WeaponDataFlag::MagicWeapon) {
            replace = avail_weapons.iter()
                .filter(|x| x.1.flag.contains(WeaponDataFlag::Ranged))
                .min_by(|x1, x2| x1.0.cmp(&x2.0)).and_then(|x| ItemData::try_index_get(x.0));
        }
        else if current_item_flags.contains(WeaponDataFlag::Ranged) {
            replace = avail_weapons.iter()
                .filter(|x| x.1.flag.contains(WeaponDataFlag::MagicWeapon))
                .min_by(|x1, x2| x1.0.cmp(&x2.0)).and_then(|x| ItemData::try_index_get(x.0))
        }
        else if current_item_flags.contains(WeaponDataFlag::SlimWeapon) {
            replace = avail_weapons.iter()
                .filter(|(dm, x)| x.flag.contains(WeaponDataFlag::SlimWeapon))
                .map(|(dm, x)| (x.item_index, dm))
                .min_by(|x1, x2| x1.1.cmp(&x2.1))
                .and_then(|x| ItemData::try_index_get(x.0));
        }
        else if current_item_flags.contains(WeaponDataFlag::Smash) {
            replace = avail_weapons.iter()
                .filter(|(might, x)| ((x.rank == rank) || ((x.rank + 1) == rank)) &&
                    (x.flag.contains(WeaponDataFlag::Smash) || (x.flag.contains(WeaponDataFlag::NoFollowUp) && x.weapon_type == 6))
                )
                .map(|x| (x.1.item_index, x.0))
                .min_by(|x1, x2| x1.1.cmp(&x2.1))
                .and_then(|x| ItemData::try_index_get(x.0));
        }
        if replace.is_none() && current_item_flags.contains(WeaponDataFlag::Effective) {
            replace = avail_weapons.iter()
                .filter(|(dm, x)| x.flag.contains(WeaponDataFlag::Effective))
                .map(|(dm, x)| (x.item_index, dm))
                .min_by(|x1, x2| x1.1.cmp(&x2.1))
                .and_then(|x| ItemData::try_index_get(x.0));
        }
        if replace.is_none() && current_item_flags.contains(WeaponDataFlag::Critical) {
            let critical = item.critical;
            replace = avail_weapons.iter()
                .map(|(dm, x)| (x.item_index, (x.crit - critical) * (x.crit - critical)))
                .min_by(|x1, x2| x1.1.cmp(&x2.1))
                .and_then(|x| ItemData::try_index_get(x.0));
        }
        // let weapon = Mess::get(item.name);
        replace.or_else(|| {
            // println!("Item {} has no replacement", weapon);
            let rank = item.get_weapon_level();
            avail_weapons.iter()
                .filter(|(dm, x)| {
                    /*
                    println!("Possible {} Replacement {}: {} || {}",
                             weapon,
                             Mess::get_name(ItemData::try_index_get(x.item_index).unwrap().iid),
                             x.flag.contains(WeaponDataFlag::Generic),
                             x.flag.is_normal()
                    );
                    */
                    x.flag.is_normal() || x.flag.contains(WeaponDataFlag::Generic)
                })
                .map(|x| {
                    let drank = rank - x.1.rank as i32;
                    (drank * drank * 4 + x.0, x.1.item_index)
                })
                .min_by(|x1, x2| x1.0.cmp(&x2.0))
                .and_then(|x| ItemData::try_index_get(x.0))
        })
    }
    pub fn do_simple_replacement(&self, unit: &Unit, transporter: bool) {
        let mut weapon_levels = [0; 11];
        let mut weapon_types = vec![];
        let can_bullet = unit.job.mask_skills.find_sid("SID_弾丸装備").is_none() && unit.job.get_max_weapon_level(9) > 0;
        let can_dragon = unit.job.mask_skills.find_sid("SID_竜石装備").is_none() && unit.job.get_max_weapon_level(9) > 0;
        for x in 1..9 {
            if x == 7 { continue; } // Skip Rod
            if unit.job.weapons[x] == 1 || (unit.job.weapons[x] > 1 && unit.selected_weapon_mask.value & (1 << x) != 0) {
                let is_upgrade = if unit.job.weapon_mask_plus.value & unit.original_aptitude.value != 0 { 1 } else { 0 };
                let level = (unit.job.get_max_weapon_level(x as i32) + is_upgrade) as u8;
                weapon_levels[x] = level;
                weapon_types.push((x, level));
            }
        }
        let job_equip_kinds = weapon_types.len();
        if job_equip_kinds == 0 { return; }
        if job_equip_kinds < 3 {
            if job_equip_kinds > 1 {
                let highest = weapon_types.iter().max_by(|x1, x2| x1.1.cmp(&x2.1)).cloned().unwrap();
                for _ in weapon_types.len()..3 { weapon_types.push(highest); }
            } else if job_equip_kinds == 1 {
                let highest = weapon_types[0].clone();
                for _ in 0..2 { weapon_types.push(highest.clone()); }
            }
        }
        let special_level = unit.job.get_max_weapon_level(9) as u8;
        if special_level > 0 {
            if can_bullet {
                weapon_levels[9] = special_level;
                for _ in 0..3 { weapon_types.push((10, special_level)); }

            }
            else if can_dragon {
                weapon_levels[10] = special_level;
                for _ in 0..3 { weapon_types.push((10, special_level)); }
            }
        }
        if weapon_types.len() == 0 { return; }
        let enemy = unit.person.get_asset_force() == 1;
        weapon_types.sort_by(|x1, x2| x2.1.cmp(&x1.1));
        let mut current_item_weapon_ranks = vec![];
        let chapter_max_weapon_level = get_unit_avail_weapon_levels(unit).iter().max().cloned().unwrap_or(2);
        for x in 0..8 {
            if let Some(unit_item) = unit.item_list.get_item(x)
                .filter(|x| x.item.parent.index > 2 && x.item.flag.value & 128 == 0 && x.is_weapon() && unit.can_equip_item(x.item, false, true))
            {
                if unit_item.item.kind == 9 && unit.job.get_max_weapon_level(9) == 0 {
                    let kind = if unit_item.item.is_dragon() { 10 } else { 9 };
                    if !current_item_weapon_ranks.iter_mut().find(|(k, level)| *k == kind).is_none() {
                        current_item_weapon_ranks.push((kind, chapter_max_weapon_level));
                    }
                }
                else {
                    if let Some((_, level)) = current_item_weapon_ranks.iter_mut().find(|(k, level)| *k == unit_item.item.kind) {
                        if *level < unit_item.item.get_weapon_level() { *level = unit_item.item.get_weapon_level(); }
                    }
                    else { current_item_weapon_ranks.push((unit_item.item.kind, unit_item.item.get_weapon_level())); }
                }
            }
        }
        current_item_weapon_ranks.sort_by(|x1, x2| x2.1.cmp(&x1.1));

        let mut removed_from_pool = vec![];
        for x in 0..8 {
            if let Some(unit_item) = unit.item_list.get_item(x)
                .filter(|x| x.item.parent.index > 2 && x.item.flag.value & 128 == 0 && x.is_weapon() &&
                    unit.can_equip_item(x.item, false, true))
            {
                removed_from_pool.push(unit_item.item.parent.index);
            }
        }
        let chapter_completed = get_story_chapters_completed();
        let mut kind_replace: HashMap<usize, Vec<WeaponData>> =
            weapon_types.iter().zip(current_item_weapon_ranks.iter())
                .flat_map(|(x1, x2)| Some(x2.0 as usize).zip(self.weapon_list.get(&(x1.0 as i32))))
                .map(|(x1, x2)| (x1, x2.iter().filter(|data|
                     !removed_from_pool.contains(&data.item_index) &&
                         data.can_equip(Some(unit), enemy) && data.flag.can_use(enemy, chapter_completed) &&
                         !data.flag.contains(WeaponDataFlag::Rare) && (!data.flag.contains(WeaponDataFlag::EnemyOnly) || data.flag.contains(WeaponDataFlag::EnemyOnly) == enemy)
                 ).map(|v| v.clone()).collect::<Vec<_>>()
            )).collect();

        for x in 0..8 {
            if let Some(unit_item) = unit.item_list.get_item(x)
                .filter(|x| x.item.parent.index > 2 && x.item.flag.value & 128 == 0 && x.is_weapon() &&
                    unit.can_equip_item(x.item, false, true))
            {
                let might = unit_item.item.power;
                let flags = unit_item.flags;
                let kind =
                    if unit_item.item.kind == 9 && unit_item.item.flag.value & 0x4000000 != 0 { 10 }
                    else { unit_item.item.kind as usize };

                if let Some((replacement_kind, mut weapon_list)) = kind_replace.get_mut(&kind).and_then(|list| list.get(0)
                    .map(|v| v.weapon_type as usize).zip(Some(list)))
                {
                    let avail_weapon_kinds = weapon_list.iter()
                        .filter(|data| data.rank <= weapon_levels[replacement_kind])
                        .map(|data|{
                            let dm = might as i32 - data.might as i32;
                            (dm*dm, data)
                        })
                        .collect();

                    if let Some(item) = self.get_closest_match(unit_item.item, &avail_weapon_kinds) {
                        if Transporter::can_add() && transporter { Transporter::add_unit_item(unit_item); }
                        unit_item.ctor(item);
                        unit_item.flags = flags;
                        weapon_list.retain_mut(|f| f.item_index != item.parent.index);
                    }
                    else { unit_item.dispose(); }
                }
            }
        }
    }
}

pub fn is_vaild_weapon(item: &ItemData) -> bool {
    let iid = item.iid.to_string(); 
    if item.icon.is_none() { return false; }
    if item.flag.value & 128 != 0  { return false; }
    if Mess::get(item.name).to_string().len() <= 1 { return false;}
    if item.kind == 0 || item.kind > 9 { return false; }
    // No Meteor / Liberation / Misercode
    if iid == "IID_メティオ" || iid == "IID_ミセリコルデ" || iid == "IID_リベラシオン" { false } else { true } 
}

pub fn get_magic_staff_by_level(level: i32) -> usize {
    if level > 32 { 4}
    else if level > 25 { 3}
    else if level > 15 { 2}
    else if level > 10 { 1}
    else { 0 }
}
