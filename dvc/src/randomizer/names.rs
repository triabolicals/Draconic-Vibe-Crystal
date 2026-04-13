use std::collections::HashMap;
use outfit_core::{get_outfit_data, PersonalDressData};
use super::*;
use crate::utils::*;
pub struct AppearanceRandomizer {
    pub playables: [i32; 42],
    pub emblem: [i32; 30],
    pub npcs: [i32; 30],
    pub summons: HashMap<i32, i32>,
}

impl AppearanceRandomizer {
    pub fn init() -> Self { 
        let summons = PersonData::get_list().unwrap().iter().filter(|x| x.summon_rate != 0 && x.summon_rank < 2).map(|x| (x.parent.hash, -1)).collect();
        Self { playables: [-1; 42], emblem: [-1; 30], npcs: [-1; 30], summons }
    }
    pub fn reset(&mut self) {
        self.playables = [-1; 42];
        self.emblem = [-1; 30];
        self.npcs = [-1; 30];
    }
    pub fn randomize(&mut self, only_summons: bool) {
        let appearances = &get_outfit_data().dress.personal;
        let rng = Random::get_system();

        let mut total_m: Vec<_> = appearances.iter().enumerate()
            .filter(|x| !x.1.is_female)
            .map(|x| (x.0 as i32, PersonData::try_get_hash(x.1.hash).and_then(|j| j.get_job()).is_some() && !x.1.generic))
            .collect();

        let mut total_f: Vec<_> = appearances.iter().enumerate()
            .filter(|x| x.1.is_female)
            .map(|x| (x.0 as i32, PersonData::try_get_hash(x.1.hash).and_then(|j| j.get_job()).is_some() && !x.1.generic))
            .collect();
        
        self.summons.iter_mut().for_each(|(hash, appearance)|{
            if let Some(person) = PersonData::try_get_hash(*hash){
                if person.gender == 1 { *appearance = total_m.get_random_element(rng).map(|v| v.0).unwrap_or(-1); }
                else if person.gender == 2 { *appearance = total_f.get_random_element(rng).map(|v| v.0).unwrap_or(-1); }
                else { *appearance = -1; }
            }
        });
        if only_summons { return; }
        let mut count = 0;
        let rng = get_rng();
        EMBLEM_GIDS.iter().enumerate().for_each(|(x,gid)|{
            if x != 19 {
                if let Some(god) = GodData::get(gid) {
                    let set = if god.female == 1 { &mut total_f } else { &mut total_m };
                    if let Some(m) = get_random_filter_remove(set, rng, |f|f.1) {
                        self.emblem[count] = m;
                        count += 1;
                    }
                }
            }
        });
        if let Some(male_emblem_lueur) = get_random_filter_remove(&mut total_m, rng, |f| f.1) { self.emblem[25] = male_emblem_lueur; }
        if let Some(female_emblem_lueur) =get_random_filter_remove(&mut total_f, rng, |f| f.1) {  self.emblem[26] = female_emblem_lueur; }
        
        // Playable Characters will not contain other playable characters
        let rng = get_rng();
        for x in 0..41 {
            let female = PersonData::get(PIDS[x])
                .map(|person| person.gender == 2 || (person.flag.value & 32 != 0 && person.gender == 1))
                .unwrap_or(false);
            if x == 0 || !female {
                if let Some(m) = get_random_filter_remove(&mut total_m, rng, |f| f.0 > 42){ // remove from playable pool
                    if x == 0 { self.playables[0] = m; } else { self.playables[1 + x] = m; }
                }
            }
            if x == 0 || female {
                if let Some(m) = get_random_filter_remove(&mut total_f, rng,|f| f.0 > 42){ // remove from playable pool
                    if x == 0 { self.playables[1] = m; } else { self.playables[1 + x] = m; }
                }
            }
        }
        let game_data = GameData::get();

        for i in 0..30 {
            if let Some(is_female) = game_data.units.iter()
                .find(|x| *x.1 == 41 + i)
                .and_then(|p| PersonData::try_get_hash_mut(*p.0))
                .map(|v| v.gender == 2)
            {
                let set = if is_female { &mut total_f } else { &mut total_m };
                if let Some(m) =  set.get_remove(rng){ self.npcs[i as usize] = m.0; }
            }
        }
    }
    pub fn get_npc_name(&self, index: i32) -> Option<&String> {
        get_outfit_data().dress.personal.get(self.npcs[index as usize] as usize).map(|a| &a.mpid)
    }
    pub fn get_unit_appearance(&self, unit: &Unit) -> Option<&PersonalDressData> { self.get_person_appearance(unit.person) }
    pub fn get_person_appearance(&self, person: &PersonData) -> Option<&PersonalDressData> {
        let db = get_outfit_data();
        if let Some(s) = self.summons.get(&person.parent.hash).filter(|&&x| x >= 0) {
            return db.dress.personal.get(*s as usize)
        }
        if person.flag.value & 512 != 0 { None 
            /*
            if let Some(person) = PersonData::try_get_hash(person.hometown) {
                let index =
                    if person.parent.index == 1 { GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) - 1 }
                    else { GameData::get().units.get(&person.parent.hash).map(|v| *v).unwrap_or(100) + 1};
                if index < 42 { return self.playables.get(index as usize).and_then(|&x| SEARCH_LIST.get().unwrap().appearances.get(x as usize)); }
            }
            None
            
             */
        }
        else {
            let index =
                if person.parent.index == 1 { GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) - 1 }
                else { GameData::get().units.get(&person.parent.hash).map(|v| *v).unwrap_or(100) + 1};
            println!("{} has appearance #{} [NPC: {}]", Mess::get_name(person.pid), index, DVCFlags::RandomBossesNPCs.get_value() );
            if index >= 100 || index < 0 { return None;}
            if index < 42 { self.playables.get(index as usize) } else if !DVCFlags::RandomBossesNPCs.get_value() { None }
            else { self.npcs.get(index as usize - 42) }
                .and_then(|&x| db.dress.personal.get(x as usize))
        }

    }
    pub fn get_emblem_appearance(&self, index: i32) -> Option<&PersonalDressData> {
        if index >= 30 { None }
        else { get_outfit_data().dress.personal.get(self.emblem[index as usize] as usize) }
    }
    pub fn get_emblem_app_person_index(emblem_index: i32) -> Option<(i32, &'static PersonData)> {
        if emblem_index >= 30 { None }
        else {
            let i = if emblem_index == 19 { if DVCVariables::is_lueur_female() { 26 } else { 25 } } else { emblem_index };
            let data = RandomizedGameData::get_read();
            let index = data.person_appearance.emblem[i as usize];
            data.person_appearance.get_emblem_appearance(i).and_then(|p| Some(index).zip(PersonData::try_get_hash(p.hash)))
        }

    }
    pub fn get_alias(person: &PersonData) -> &'static mut Il2CppString {
        let name = person.name.map(|v| v.to_string()).unwrap();
        let alias = name.replace("MPID_", "MPID_alias_");
        let s = 
        if Mess::is_exist(alias.as_str()) { Mess::get(alias ) }
        else if name.contains("ItemShop") { Mess::get("MID_MENU_ITEM_SHOP") }
        else if name.contains("WeaponShop") { Mess::get("MID_MENU_WEAPON_SHOP") }
        else if name.contains("AccessoriesShop") { Mess::get("MID_MENU_ACCESSORY_SHOP") }
        else if name.contains("BlackSmith") { Mess::get("MID_MENU_REFINE_SHOP") }
        else if name.contains("Eve") || person.belong.is_some_and(|bid| bid.str_contains("フィレネ")){ Mess::get("MID_PLACE_Filene") }
        else if name.contains("Morion") || person.belong.is_some_and(|bid| bid.str_contains("ブロディア")) { Mess::get("MID_PLACE_Brodia") }
        else if name.contains("Sfoglia") || person.belong.is_some_and(|bid| bid.str_contains("ソルム")) { Mess::get("MID_PLACE_Solm") }
        else if name.contains("Hyacinth") || person.belong.is_some_and(|bid| bid.str_contains("イルシオン")) { Mess::get("MID_PLACE_Ircion") }
        else if person.get_job().is_some() { person.get_job().map(|j| Mess::get_name(j.jid)).unwrap() }
        else { Mess::get_name(person.pid) };
        s
    }
}
pub fn get_emblem_person(mid: &Il2CppString) -> Option<&'static PersonData> {
    if !DVCFlags::GodNames.get_value() { return None; }
    if let Some(s) = RINGS.iter().position(|x| mid.str_contains(*x)){
        RandomizedGameData::get_read().person_appearance.get_emblem_appearance(s as i32)
            .and_then(|p| PersonData::try_get_hash(p.hash))
    }
    else if mid.str_contains("MGID_Lueur") {
        let index = if DVCVariables::is_lueur_female() { 26 } else { 25 };
        RandomizedGameData::get_read().person_appearance.get_emblem_appearance(index).
            and_then(|p| PersonData::try_get_hash(p.hash))
    }
    else { None }
}
pub fn get_random_filter_remove(set: &mut Vec<(i32, bool)>, rng: &Random, filter: impl Fn(&(i32, bool)) -> bool ) -> Option<i32> {
    let mut v = set.iter().filter(|f| filter(f)).collect::<Vec<_>>();
    v.get_remove(rng)
        .and_then(| random_index | set.iter().position(| random_idx | random_index.0 == random_idx.0 ))
        .map(|index_position | set.swap_remove(index_position))
        .map(|v| v.0 )
}