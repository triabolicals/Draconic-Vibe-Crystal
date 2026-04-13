use unity::prelude::*;
use engage::{
    random::*,
    gamedata::{*, item::*, god::*, skill::SkillData},
};
use std::collections::HashMap;
use skyline::patching::Patch;
use crate::utils;
use crate::config::{DVCFlags};
use crate::randomizer::data::engage_attacks::EngageAttackRandomizer;
use crate::randomizer::data::GameData;
use crate::randomizer::data::items::ItemPool;
use crate::randomizer::Randomizer;


#[derive(PartialEq, Clone)]
pub enum EngageRandoSet {
    MustBow,
    Any,
    MustWeapon,
    AnyBow,
    WeaponBow,
}

pub struct EngageItemRandomizer  {
    pub emblem_new_weapon_mask: Vec<i32>,
    pub random_engage_atk: HashMap<i32, i32>,
    pub random_non_engage_atk: HashMap<i32, i32>,
}

impl EngageItemRandomizer {
    pub fn init() -> Self {
        Self {
            emblem_new_weapon_mask: Vec::new(),
            random_engage_atk: HashMap::new(),
            random_non_engage_atk: HashMap::new(),
        }
    }
    pub fn randomize(&mut self, data: &GameData, engage_attacks: &EngageAttackRandomizer) {
        println!("Randomizing Engage Weapons...");
        let types_non_rand: Vec<_> = data.emblem_pool.emblem_data.iter().map(|data| {
            SkillData::try_get_hash(data.engage_atk).map(|skill| {
                if skill.sid.str_contains("リンエンゲージ技") { EngageRandoSet::MustBow }
                else if !engage_atk_can_equip_non_weapons(Some(skill.sid)) { EngageRandoSet::MustWeapon }
                else { EngageRandoSet::Any }
            }).unwrap_or(EngageRandoSet::Any)
        }).collect();
        let types_rand: Vec<_> = engage_attacks.atks.iter().map(|x| {
            let type1 =
                SkillData::try_get_hash(x.engage_atk).map(|skill| {
                    if skill.sid.str_contains("リンエンゲージ技") { EngageRandoSet::MustBow }
                    else if !engage_atk_can_equip_non_weapons(Some(skill.sid)) { EngageRandoSet::MustWeapon }
                    else { EngageRandoSet::Any }
                }).unwrap_or(EngageRandoSet::Any);
            let type2 =
                SkillData::try_get_hash(x.linked_engage_atk).map(|skill| {
                    if skill.sid.str_contains("リンエンゲージ技") { EngageRandoSet::MustBow }
                    else if !engage_atk_can_equip_non_weapons(Some(skill.sid)) { EngageRandoSet::MustWeapon }
                    else { EngageRandoSet::Any }
                }).unwrap_or(EngageRandoSet::Any);
                match (type1, type2) {
                    (EngageRandoSet::MustBow, _) => { EngageRandoSet::MustBow },
                    (EngageRandoSet::Any, EngageRandoSet::MustBow) => { EngageRandoSet::AnyBow },
                    (EngageRandoSet::MustWeapon, EngageRandoSet::MustBow) => { EngageRandoSet::WeaponBow },
                    (EngageRandoSet::Any, EngageRandoSet::MustWeapon) => { EngageRandoSet::MustWeapon },
                    (EngageRandoSet::Any, _) => { EngageRandoSet::Any },
                    (EngageRandoSet::MustWeapon, _) => { EngageRandoSet::MustWeapon }
                    (_, _) => { EngageRandoSet::Any }
                }
        }).collect();
        self.random_engage_atk.clear();
        self.random_non_engage_atk.clear();
        let mut weapon_pool = WeaponPool::new(&data.item_pool);
        data.emblem_pool.emblem_data.iter().zip(types_non_rand.iter()).for_each(|(data, ty)| {
            randomize_by_weapon_rand_type(&mut self.random_non_engage_atk, data, &mut weapon_pool, ty.clone());
        });
        let mut weapon_pool = WeaponPool::new(&data.item_pool);
        data.emblem_pool.emblem_data.iter().zip(types_rand.iter()).for_each(|(data, ty)| {
            randomize_by_weapon_rand_type(&mut self.random_engage_atk, data, &mut weapon_pool, ty.clone());
        });
        // Replacement for Enemy Versions
        let mut add_enemy: Vec<(i32, i32)> = vec![];
        self.random_engage_atk.iter().for_each(|item| {
            let iid = ItemData::try_get_hash(*item.0).unwrap().iid.to_string();
            data.item_pool.engage_items.enemy.iter()
                .filter(|x| x.0.contains(iid.as_str()))
                .for_each(|x| { add_enemy.push( ( x.1, *item.1) ); });
        });
        add_enemy.iter().for_each(|x| { self.random_engage_atk.insert(x.0, x.1); });
        add_enemy.clear();
        self.random_non_engage_atk.iter().for_each(|item| {
            let iid = ItemData::try_get_hash(*item.0).unwrap().iid.to_string();
            data.item_pool.engage_items.enemy.iter()
                .filter(|x| x.0.contains(iid.as_str()))
                .for_each(|x| { add_enemy.push( ( x.1, *item.1) ); });
        });
        add_enemy.iter().for_each(|x| { self.random_non_engage_atk.insert(x.0, x.1); });
        
    }
    pub fn get_replacement(&self, hash: i32) -> &'static mut ItemData {
        if DVCFlags::EngageWeapons.get_value(){
            if DVCFlags::EngageAttacks.get_value() { self.random_engage_atk.get(&hash).and_then(|v| ItemData::try_get_hash_mut(*v)) }
            else { self.random_non_engage_atk.get(&hash).and_then(|v| ItemData::try_get_hash_mut(*v)) }
                .or_else(|| ItemData::try_get_hash_mut(hash))
        }
        else { ItemData::try_get_hash_mut(hash) }.unwrap()
    }
    pub fn get_replacement_iid(&self, iid: &'static Il2CppString) -> &'static Il2CppString {
        if let Some(item) = ItemData::get(iid) {
            let item_index = item.parent.index;
            let replacement_item = self.get_replacement(item_index);
            replacement_item.iid
        } else { iid }
    }
    pub fn commit(&self, data: &GameData) {
        let engage_atk_random = DVCFlags::EngageAttacks.get_value();
        println!("Engage Atk: {}", engage_atk_random);
        data.emblem_pool.emblem_data.iter().for_each(|data| { data.reset_weapons(); });
        if !DVCFlags::EngageWeapons.get_value() {
            if engage_atk_random {
                data.emblem_pool.emblem_data.iter().for_each(|x|{
                    let mut mask = 0;
                    x.level_data[0].style_items.iter().for_each(|item| {
                        mask |= ItemData::try_get_hash(*item).map(|i| (1 << i.kind)).unwrap_or(0);
                    });
                    mask &= 1023;
                    if let Some(engage_atk) = x.get_god().engage_attack.and_then(|sid| SkillData::get_mut(sid)){
                        if engage_atk.weapon_prohibit.value != 0 {
                            engage_atk.weapon_prohibit.value &= !mask;
                            engage_atk.style_skills.iter_mut().for_each(|style|{style.weapon_prohibit.value &= !mask});
                        }
                    }
                    if let Some(engage_atk) = x.get_god().engage_attack_link.and_then(|sid| SkillData::get_mut(sid)){
                        if engage_atk.weapon_prohibit.value != 0 {
                            engage_atk.weapon_prohibit.value &= !mask;
                            engage_atk.style_skills.iter_mut().for_each(|style|{ style.weapon_prohibit.value &= !mask});
                        }
                    }
                });
            }
            else {
                data.skill_pool.engage_attacks.iter().map(|x| (SkillData::try_get_hash_mut(x.0), x.1)).for_each(|(x, y)|
                    if let Some(engage_atk) = x {
                        engage_atk.weapon_prohibit.value = y;
                        engage_atk.style_skills.iter_mut()
                            .for_each(|style|{style.weapon_prohibit.value = y});
                    }
                );
            }
        }
        else {
            let mut new_mask = vec![];
            let map = if engage_atk_random { &self.random_engage_atk } else { &self.random_non_engage_atk };
            for x in 0..data.emblem_pool.emblem_data.len() {
                let mut mask = 0;
                data.emblem_pool.emblem_data[x].get_god().get_level_data().unwrap().iter_mut()
                    .flat_map(|data| data.style_items.iter_mut())
                    .flat_map(|list| list.iter_mut())
                    .for_each(|i| {
                        let hash = i.parent.hash;
                        if let Some(item) = map.get(&hash).and_then(|hash| ItemData::try_get_hash_mut(*hash)) {
                            let kind = item.kind;
                            *i = item;
                            if kind > 0 && kind < 10 { mask |= 1 << kind }
                        }
                    });
                new_mask.push(mask & 1023);
            }
            data.emblem_pool.emblem_data.iter().zip(new_mask.iter()).map(|data| (data.0.get_god_mut(), data.1))
                .for_each(|(god, mask)| {
                    god.flag.value &= !16;
                    if let Some(engage_atk) = god.engage_attack.and_then(|sid| SkillData::get_mut(sid)) {
                        if engage_atk.weapon_prohibit.value != 0 {
                            engage_atk.weapon_prohibit.value &= !mask;
                            engage_atk.style_skills.iter_mut().for_each(|style| { style.weapon_prohibit.value &= !mask });
                        }
                    }
                    if let Some(engage_atk) = god.engage_attack_link.and_then(|sid| SkillData::get_mut(sid)) {
                        if engage_atk.weapon_prohibit.value != 0 {
                            engage_atk.weapon_prohibit.value &= !mask;
                            engage_atk.style_skills.iter_mut().for_each(|style| { style.weapon_prohibit.value &= !mask });
                        }
                    }
                });
            if let Some(echo) = SkillData::get_mut("SID_重唱") {
                echo.style_skills.iter_mut().for_each(|style|{ style.weapon_prohibit.value = 0; })
            }
            Patch::in_text(0x01dee3a8).bytes(&[0x22, 0x00, 0x80, 0x52]).unwrap();
        }
    }
}
pub struct WeaponPool {
    pub non_weapons: Vec<i32>,
    pub weapons: Vec<i32>,
    pub  bows: Vec<i32>,
}
impl WeaponPool {
    pub fn new(pool: &ItemPool) -> Self {
        Self {
            non_weapons: pool.engage_items.non_weapons.clone(),
            bows: pool.engage_items.non_weapons.clone(),
            weapons: pool.engage_items.weapons.clone(),
        }
    }
}
pub fn add_to_hash(map: &mut HashMap<i32, i32>, item: i32, rng: &Random, selection: &mut Vec<i32>, other_selection: Option<&mut Vec<i32>>, weight: i32, remove: &mut bool) -> bool {
    if !map.contains_key(&item) {
        let s_len = selection.len() as i32;
        let o_len = other_selection.as_ref().map(|v| v.len() as i32).unwrap_or(s_len);
        let total = weight * s_len + o_len;
        let new =
            if weight == 0 || other_selection.is_none() {
                if *remove { selection.get_remove(rng) }
                else { selection.get_random_element(rng).map(|v| *v) }
            }
            else if rng.get_value(total) < weight * s_len {
                if *remove { utils::get_random_and_remove(selection, rng) }
                else { selection.get(rng.get_value(selection.len() as i32) as usize).map(|x| *x) }
            }
            else {
                other_selection.and_then(|v|
                    if *remove { utils::get_random_and_remove(v, rng) }
                    else { v.get(rng.get_value(o_len) as usize).map(|x| *x) }
                )
            }.unwrap_or(item);
        map.insert(item, new);
        *remove = false;
        return true;
    }
    false
}
fn randomize_by_weapon_rand_type(map: &mut HashMap<i32, i32>, data: &crate::randomizer::data::EmblemData, pool: &mut WeaponPool, ty: EngageRandoSet) {
    let mut replace = true;
    let rng = utils::create_rng(data.hash, 2);
    match ty {
        EngageRandoSet::MustBow => {
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 != 1).for_each(|item| {
                add_to_hash(map, *item.1, &rng, &mut pool.bows, None, 0, &mut replace);
            });
            replace = true;
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 == 1).for_each(|item| {
                add_to_hash(map, *item.1, &rng, &mut pool.weapons, Some(&mut pool.non_weapons), 1, &mut replace);
            });
        }
        EngageRandoSet::MustWeapon => {
            let non_weapon = rng.get_value(2) as usize + 1;
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 != non_weapon ).for_each(|item| {
                if add_to_hash(map, *item.1, &rng, &mut pool.weapons, None, 0, &mut replace) { replace = false; }
            });
            replace = true;
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 == non_weapon).for_each(|item| {
                add_to_hash(map, *item.1, &rng, &mut pool.bows, Some(&mut pool.non_weapons), 3, &mut replace);
            });
        }
        EngageRandoSet::AnyBow | EngageRandoSet::WeaponBow => {
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 == 0).for_each(|item| {
                add_to_hash(map, *item.1, &rng, &mut pool.weapons, None, 0, &mut replace);
            });
            replace = true;
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 == 1).for_each(|item| {
                add_to_hash(map, *item.1, &rng, &mut pool.bows, Some(&mut pool.weapons), 1, &mut replace);
            });
            replace = true;
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 == 2).for_each(|item| {
                if ty == EngageRandoSet::WeaponBow { add_to_hash(map, *item.1, &rng, &mut pool.weapons, None, 1, &mut replace);}
                else { add_to_hash(map, *item.1, &rng, &mut pool.weapons, Some(&mut pool.non_weapons), 1, &mut replace);}
            });
        }
        _ => {
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 == 0).for_each(|item| {
                add_to_hash(map, *item.1, &rng, &mut pool.weapons, None, 0, &mut replace);
            });
            replace = true;
            data.level_data[0].style_items.iter().enumerate().filter(|(x, i)| x % 3 != 0).for_each(|item| {
                add_to_hash(map, *item.1, &rng, &mut pool.weapons, None, 1, &mut replace);
            });
        }
    }
}

/*
pub fn can_engage_bow(god: &GodData) -> bool { engage_atk_can_use_bow(god.engage_attack) && engage_atk_can_use_bow(god.engage_attack_link) }
pub fn can_equip_non_weapons(god: &GodData) -> bool { engage_atk_can_equip_non_weapons(god.engage_attack) && engage_atk_can_equip_non_weapons(god.engage_attack_link) }
*/

pub fn engage_atk_can_use_bow(engage_atk: Option<&Il2CppString>) -> bool {
    engage_atk.map_or_else(||true, |atk|SkillData::get(atk).map_or(true, |engage| engage.weapon_prohibit.value & 32 != 0 || engage.weapon_prohibit.value == 0))
}

pub fn engage_atk_can_equip_non_weapons(engage_atk: Option<&Il2CppString>) -> bool {
    engage_atk.map_or_else(||true, |atk| SkillData::get(atk).map_or(true, |engage| engage.weapon_prohibit.value == 0))
}
