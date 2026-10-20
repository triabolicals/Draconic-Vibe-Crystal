use std::collections::HashMap;
use bitflags::bitflags;
use engage::app::{IPersonDataMethods, ISkillArray, ISkillArrayMethods, IUnitItem, IUnitMethods, Transporter, Unit};
use crate::gamedata::items::weapondata::WeaponStats;
use crate::utils::to_opt;
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
        const EngageWeapon = 1 << 12;
    }
}
impl WeaponDataFlag {
    pub fn from_item(data: ItemData) -> Self {
        let e_skills = data.get_equip_skills();
        let item_flag = data.get_flag().m_value();
        let kind = data.get_kind().value;
        let weapon_mask = 1 << kind;
        let ranged = (weapon_mask & 112 != 0 && data.get_range_o() > 2) || (weapon_mask & 14 != 0 && data.get_range_o() > 1);
        let effectiveness =
            data.get_kind().value != 4 &&
                e_skills.m_list().iter().any(|k|{
                    let skill = SkillData::try_get_2((k.value & 0xFFF) as i32);
                    skill.get_efficacy().value > 0 && skill.get_efficacy_value() > 2
                });

        let mut flag = Self::from_bits(0).unwrap();
        flag.set(Self::Smash,  !e_skills.find("SID_スマッシュ").is_null());
        flag.set(Self::MagicWeapon, data.get_attr().value == 1 && item_flag & 65536 != 0 && kind != 6);
        flag.set(Self::Critical, data.get_critical() >= 15);
        flag.set(Self::Surge, data.get_hit() == 255 || data.get_hit() == -1);
        flag.set(Self::NoFollowUp, !e_skills.find("SID_追撃不可").is_null());
        flag.set(Self::Ranged, ranged);
        flag.set(Self::Effective, effectiveness);
        flag.set(Self::Usable, item_flag & 4 != 0);
        flag.set(Self::Rare, !data.get_equip_condition().is_null() || data.get_flag().m_value() & 1 != 0);
        flag.set(Self::EnemyOnly, item_flag & 18 != 0 );
        flag.set(Self::SlimWeapon, data.get_secure() >= 10 && data.m_weapon_level().value < 2);
        flag.set(Self::EngageWeapon, data.get_iid().to_rust_string().ends_with("_通常"));
        let iid = data.get_iid().to_string();
        flag.set(Self::Generic, GENERICS.iter().any(|generic_iid| generic_iid == &iid));
        flag
    }
    pub fn can_use(&self, enemy: bool, chapter_completed: i32) -> bool {
        if !enemy && self.contains(Self::EnemyOnly) { return false; }
        let custom_inventory = DVCVariable::UnitInventory.get_value() & if enemy { 2 } else { 1 } != 0;
        let mut flags = Self::all();
        if custom_inventory {
            if enemy { flags.remove(Self::EngageWeapon); }
            flags.remove(Self::Smash);
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
        if chapter_completed > 5 {
            flags.remove(Self::Surge);
            flags.remove(Self::MagicWeapon);
            flags.remove(Self::NoFollowUp);
            flags.remove(Self::Critical);
        }
        if chapter_completed > 2 {
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
    pub stats: WeaponStats,
    pub rank: u8,
    pub flag: WeaponDataFlag,
}
/*
pub struct DragonStoneData {
    pub item_index: i32,
    pub is_enemy_only: bool,
}
impl DragonStoneData {
    pub fn new(item: ItemData) -> Self {
        Self { item_index: item.index(), is_enemy_only: item.get_flag().m_value() & 2 != 0 }
    }
}
 */
impl WeaponData {
    pub fn new(item: ItemData) -> Self {
        Self {
            item_index: item.index(),
            weapon_type: item.get_kind().value as u8,
            stats: WeaponStats::new(item),
            rank: item.m_weapon_level().value as u8,
            flag: WeaponDataFlag::from_item(item),
        }
    }
    pub fn can_equip(&self, unit: Option<Unit>, enemy: bool) -> bool {
        if let Some(item) = to_opt(ItemData::try_get_2(self.item_index)){
            if let Some(unit) = unit { enemy || unit.can_item_equip_2(item, false, true) }
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
    pub fn new(item: ItemData) -> Self {
        let mut avail: [bool; 5] = [false; 5];
        let staff_kind;
        match item.get_use_type().value {
            2 => {
                let (o, i) = (item.get_range_o(), item.get_range_i());
                staff_kind = 1; //Heal
                if o > 1 { avail = [false, o < 9, i< 9,  o >= 9, o >= 9]; }
                else {
                    let pwr1 = item.get_power() < 11;
                    let pwr2 = item.get_power() < 21;
                    avail = [pwr1,  pwr1, pwr2,  pwr2, o >= 21];
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
            item_index: item.index(),
            staff_type: staff_kind as u8,
            rank: item.m_weapon_level().value as u8,
            rare: item.get_flag().m_value() & 3 != 0 || item.get_use_type().value == 27 || item.get_use_type().value == 7,
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
    pub extra_items: Vec<i32>,
}

impl WeaponDatabase {
    pub fn init() -> Self {
        let mut db = WeaponDatabase {
            magic_weapons: Vec::new(),
            weapon_list: [1,2,3,4,5,6,8,9, 10].into_iter().map(|x| (x, vec![])).collect(),
            staff_list: Vec::new(),
            extra_items: Vec::new(),
        };
        db.initialize();
        db
    }
    pub fn initialize(&mut self) {
        let item_list = ItemData::get_list();
        for x in 3..item_list.count(){
            let item = item_list.get(x);
            let use_type = item.get_use_type().value;
            if !is_vaild_weapon(item) {
                if use_type == 42 || use_type == 28 || use_type == 21 {
                    self.extra_items.push(item.hash());
                }
                if item.get_kind().value == 7 { self.staff_list.push(StaffData::new(item)); }
            }
            self.try_add_weapon(item);
        }
    }
    pub fn try_add_weapon(&mut self, item: ItemData) {
        if item.get_icon().is_null() { return; }
        let kind = item.get_kind().value;
        let flag = item.get_flag().m_value();
        if kind == 7 && flag & 128 == 0 {  //Staff
            self.staff_list.push(StaffData::new(item));
        }
        else if kind < 9 || (kind == 9 && item.get_hit() > 0 && !item.get_icon().to_rust_string().contains("Sombre")){
            let kind = if kind == 9 && item.is_dragon() { 10 } else { kind };
            if let Some(list) = self.weapon_list.get_mut(&kind) {
                let mut data = WeaponData::new(item);
                data.weapon_type = kind as u8;
                if item.get_kind().value != 6 && flag & 65536 != 0 { self.magic_weapons.push(data.clone()); }
                list.push(data);
            }
        }
    }
    /// Iron/Steel/Silver rank: D (1), C (2), B (3), A (4)
    pub fn get_generic_weapon(&self, new_type: i32, rank: i32) -> Option<ItemData> {
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
        GENERICS.get(generic_index).map(|iid| ItemData::get((*iid).into()))
    }

    pub fn get_staff(&self, level: i32, staff_type: i32, job_rank: i32, enemy: bool) -> Option<ItemData> {
        let possible_staffs: Vec<&StaffData> = self.staff_list.iter()
            .filter(|&x| x.can_add(level, staff_type as u8, job_rank as u8, enemy)).collect();

        possible_staffs.get_random_element(Random_2::get_system()).map(|i| ItemData::try_get_2(i.item_index))
    }

    pub fn get_tome(&self, job_rank: i32, total_level: i32, enemy: bool) -> Option<ItemData> {
        let rank_level =
            if total_level < 14 { 1 } else if total_level < 21 { 2 }
            else if total_level < 26 { 3 } else if total_level < 31 { 4 } else { 5 };

        let rank = std::cmp::min(rank_level, job_rank) as u8;
        let chapter_completed = DVCVariables::chapter_number_complete(false);
        self.weapon_list
            .get(&6)
            .and_then(|list|{
                let pool: Vec<i32> = list.iter().filter(|&x| x.flag.can_use(enemy,chapter_completed) && rank == x.rank).map(|x| x.item_index).collect();
                pool.get_random_element(Random_2::get_system()).map(|&index| ItemData::try_get_2(index))
            })
    }
    pub fn get_random_weapon(&self, unit: Option<Unit>, kind: i32, start_rank: i32, enemy: bool) -> Option<ItemData> {
        if unit.is_none() {
            self.weapon_list
                .get(&kind)
                .and_then(|list| list.get_random_element(Random_2::get_system()).map(|data| ItemData::try_get_2(data.item_index)))
        }
        else {
            let chapter_completed = DVCVariables::chapter_number_complete(false);
            let mut search_rank = if start_rank == -1 { 6 } else { start_rank as u8 };
            let mut lower_rank = if search_rank > 2 { search_rank - 1 } else { search_rank };
            if let Some(list) = self.weapon_list.get(&kind) {
                let rng = Random_2::get_system();
                let pool: Vec<_> = list.iter().filter(|x| x.flag.can_use(enemy, chapter_completed) && x.rank <= search_rank)
                    .map(|x| if enemy && x.flag.contains(WeaponDataFlag::EngageWeapon) { (x.item_index, search_rank) } else { (x.item_index, x.rank) })
                    .collect();

                while search_rank > 0 {
                    if let Some(index) = pool.get_filter(rng, |f| f.1 == search_rank || f.1 >= lower_rank){ return Some(ItemData::try_get_2(index.0)); }
                    search_rank -= 1;
                    if lower_rank > 1 { lower_rank -= 1; }
                }
            }
            None
        }
    }
    fn get_closest_match(&self, item: ItemData, avail_weapons: &Vec<(i32, &WeaponData)>) -> Option<ItemData> {
        let rank = item.m_weapon_level().value as u8;
        let mut replace = None;
        // println!("Simple Replacement for: {}", Mess::get(items.name));
        let current_item_flags = WeaponDataFlag::from_item(item);
        if item.get_kind().value != 6 && current_item_flags.contains(WeaponDataFlag::MagicWeapon) {
            replace = avail_weapons.iter()
                .filter(|x| x.1.flag.contains(WeaponDataFlag::Ranged))
                .min_by(|x1, x2| x1.0.cmp(&x2.0)).map(|x| ItemData::try_get_2(x.0));
        }
        else if current_item_flags.contains(WeaponDataFlag::Ranged) {
            replace = avail_weapons.iter()
                .filter(|x| x.1.flag.contains(WeaponDataFlag::MagicWeapon))
                .min_by(|x1, x2| x1.0.cmp(&x2.0)).map(|x| ItemData::try_get_2(x.0))
        }
        else if current_item_flags.contains(WeaponDataFlag::SlimWeapon) {
            replace = avail_weapons.iter()
                .filter(|(_, x)| x.flag.contains(WeaponDataFlag::SlimWeapon))
                .map(|(dm, x)| (x.item_index, dm))
                .min_by(|x1, x2| x1.1.cmp(&x2.1))
                .map(|x| ItemData::try_get_2(x.0));
        }
        else if current_item_flags.contains(WeaponDataFlag::Smash) {
            replace = avail_weapons.iter()
                .filter(|(_, x)| ((x.rank == rank) || ((x.rank + 1) == rank)) &&
                    (x.flag.contains(WeaponDataFlag::Smash) || (x.flag.contains(WeaponDataFlag::NoFollowUp) && x.weapon_type == 6))
                )
                .map(|x| (x.1.item_index, x.0))
                .min_by(|x1, x2| x1.1.cmp(&x2.1))
                .map(|x| ItemData::try_get_2(x.0));
        }
        if replace.is_none() && current_item_flags.contains(WeaponDataFlag::Effective) {
            replace = avail_weapons.iter()
                .filter(|(_, x)| x.flag.contains(WeaponDataFlag::Effective))
                .map(|(dm, x)| (x.item_index, dm))
                .min_by(|x1, x2| x1.1.cmp(&x2.1))
                .map(|x| ItemData::try_get_2(x.0));
        }
        if replace.is_none() && current_item_flags.contains(WeaponDataFlag::Critical) {
            let critical = item.get_critical();
            replace = avail_weapons.iter()
                .map(|(_, x)| (x.item_index, (x.stats.crit - critical) * (x.stats.crit - critical)))
                .min_by(|x1, x2| x1.1.cmp(&x2.1))
                .map(|x| ItemData::try_get_2(x.0));
        }
        replace.or_else(|| {
            let rank = item.m_weapon_level().value;
            avail_weapons.iter()
                .filter(|(_, x)| {
                    x.flag.is_normal() || x.flag.contains(WeaponDataFlag::Generic)
                })
                .map(|x| {
                    let drank = rank - x.1.rank as i32;
                    let score = drank * drank * 10 + x.0;
                    // println!("Replacement Candidate: #{} {} [{}]", x.1.item_index, Mess::get(ItemData::try_get_2(x.1.item_index).get_name()), score);
                    (score, x.1.item_index)
                })
                .min_by(|x1, x2| x1.0.cmp(&x2.0))
                .and_then(|x| to_opt(ItemData::try_get_2(x.1)))
        })
    }
    pub fn do_simple_replacement(&self, unit: Unit, transporter: bool) {
        let mut weapon_levels = [0; 11];
        let mut weapon_types = vec![];
        let job = unit.get_job();
        let mask_skills = job.get_mask_skill();
        let can_bullet = !mask_skills.find("SID_弾丸装備").is_null() && job.get_max_weapon_level(9).value > 0;
        let can_dragon = !mask_skills.find("SID_竜石装備").is_null() &&  job.get_max_weapon_level(9).value > 0;
        let weapon = job.weapons();
        for x in 1..9 {
            if x == 7 { continue; } // Skip Rod
            if weapon.get(x) == 1 || (weapon.get(x) > 1 && unit.get_selected_weapon_mask().m_value() & (1 << x) != 0) {
                let is_upgrade = if job.get_level_plus_weapon_mask().m_value() & unit.get_original_aptitude().m_value() != 0 { 1 } else { 0 };
                let level = (job.get_max_weapon_level(x as i32).value + is_upgrade) as i32;
                weapon_levels[x] = level;
                weapon_types.push((x as i32, level));
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
        let special_level = job.get_max_weapon_level(9).value;
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
        let enemy = unit.get_person().get_asset_force().value  == 1;
        weapon_types.sort_by(|x1, x2| x2.1.cmp(&x1.1));
        let chapter_max_weapon_level = dvc::unit::item::get_unit_avail_weapon_levels(unit).iter().max().cloned().unwrap_or(1);
        let mut current_item_weapon_ranks = vec![];
        weapon_types.iter_mut().for_each(|(i, l)|{ if *i < 9 { *l = chapter_max_weapon_level; } });
        let mut max_level = 0;
        for x in 0..8 {
            if let Some(unit_item) = to_opt(unit.get_item(x))
                .filter(|x|{
                    let item = x.m_item();
                    item.index() > 2 && item.get_flag().m_value() & 128 == 0 && item.is_weapon()
                })
            {
                let weapon_level = unit_item.m_item().m_weapon_level().value;
                if max_level < weapon_level { max_level = weapon_level; }
                if unit_item.m_item().get_kind().value == 9 && job.get_max_weapon_level(9).value == 0 {
                    let kind = if unit_item.m_item().is_dragon() { 10 } else { 9 };
                    if current_item_weapon_ranks.iter_mut().find(|(k, _)| *k == kind).is_none() {
                        current_item_weapon_ranks.push((kind, weapon_level));
                    }
                }
                else {
                    let weapon_level = unit_item.m_item().m_weapon_level().value;
                    let kind =  unit_item.m_item().get_kind().value;
                    if let Some((_, level)) = current_item_weapon_ranks.iter_mut().find(|(k, _)| *k == kind){
                        if *level < weapon_level { *level = weapon_level; }
                    }
                    else { current_item_weapon_ranks.push((kind, weapon_level)); }
                }
            }
        }
        weapon_levels.iter_mut().enumerate().
            for_each(|(i, v)|{
                if i < 9 {
                    let min = std::cmp::min(*v, max_level);
                    if *v >= max_level { *v = min; }
                }
        });
        current_item_weapon_ranks.sort_by(|x1, x2| x2.1.cmp(&x1.1));
        let mut removed_from_pool = vec![];
        for x in 0..8 {
            if let Some(unit_item) =  to_opt(unit.get_item(x))
                .filter(|x|{
                    let item = x.m_item();
                    item.index() > 2 && item.get_flag().m_value() & 128 == 0 && x.is_weapon() &&
                        unit.can_item_equip_2(item, false, true)
                })
            {
                removed_from_pool.push(unit_item.m_item().index());
            }
        }
        let chapter_completed = DVCVariables::chapter_number_complete(false);
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
            if let Some(unit_item) = to_opt(unit.get_item(x))
                .filter(|x|{
                    let item = x.m_item();
                    item.index() > 2 && item.get_flag().m_value() & 128 == 0 && x.is_weapon() &&
                        !unit.can_item_equip_2(item, false, true)
                })
            {
                let item = unit_item.m_item();
                let might = item.get_power();
                let flags = item.get_flag().m_value();
                let rank = item.m_weapon_level().value as u8;
 
                let kind = if item.get_kind().value == 9 && flags & 0x4000000 != 0 { 10 } else { item.get_kind().value as usize };
                if let Some((replacement_kind, weapon_list)) = kind_replace.get_mut(&kind).and_then(|list| list.get(0)
                    .map(|v| v.weapon_type as usize).zip(Some(list)))
                {
                    let search_rank = if (weapon_levels[replacement_kind] as u8) <= rank { rank } else { weapon_levels[replacement_kind] as u8 };
                    let avail_weapon_kinds = weapon_list.iter()
                        .filter(|data| data.rank <= search_rank)
                        .map(|data|{
                            let dm = might as i32 - data.stats.might as i32;
                            (dm*dm, data)
                        })
                        .collect();

                    if let Some(item) = self.get_closest_match(unit_item.m_item(), &avail_weapon_kinds) {
                        // println!("Simple Replacement {} -> {}", Mess::get(unit_item.m_item().get_name()), Mess::get(item.get_name()));
                        if Transporter::can_add() && transporter { Transporter::add(unit_item); }
                        unit_item.ctor_2(item);
                        // unit_item.flags = flags;
                        weapon_list.retain_mut(|f| f.item_index != item.index());
                    }
                    else { unit_item.dispose(); }
                }
            }
        }
        unit.item_close_up();
    }
}

pub fn is_vaild_weapon(item: ItemData) -> bool {
    let iid = item.get_iid().to_rust_string();
    if item.get_icon().is_null() { return false; }
    if item.get_flag().m_value() & 128 != 0  { return false; }
    if Mess::get(item.get_name()).to_string().len() <= 1 { return false;}
    if !item.is_weapon() { return false; }
    if iid == "IID_メティオ" || iid == "IID_ミセリコルデ" || iid == "IID_リベラシオン" { false } else { true }
}

pub fn get_magic_staff_by_level(level: i32) -> usize {
    if level > 32 { 4}
    else if level > 25 { 3}
    else if level > 15 { 2}
    else if level > 10 { 1}
    else { 0 }
}
