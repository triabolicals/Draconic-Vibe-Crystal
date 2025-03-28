use concat_string::concat_string;
use unity::prelude::*;
use engage::{
    mess::*,
    random::*,
    gamedata::{*, item::*, god::*, skill::SkillData},
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
};
use std::{collections::HashSet, sync::Mutex};
use crate::{enums::*, randomizer::emblem::EMBLEM_LIST};
use crate::{DVCVariables, CONFIG};

pub static ENGAGE_ITEMS: Mutex<EngageItemList> = Mutex::new(
    EngageItemList{ 
        item_list: Vec::new(), 
        god_items_list: Vec::new(), 
        engage_weapon: [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66, 258, 0], 
        custom_engage_weapon: [0; 20],
    });

pub struct GodStyleItems {
    pub item: [[i32; 9]; 3],
}

impl GodStyleItems {
    fn new() -> Self { Self { item: [[-1; 9]; 3], } }
}
pub struct EngageItem {
    pub item_index: i32,
    pub replaced_index: i32,
    pub weapon: bool,
    pub is_bow: bool,
}

impl EngageItem {
    fn new(itemdata_index: i32, not_weapon: bool, bow: bool) -> Self {
        Self {
            item_index: itemdata_index, 
            weapon: not_weapon, 
            is_bow: bow, 
            replaced_index: 0, 
        }
    }
}

pub struct EngageItemList {
    pub item_list: Vec<EngageItem>,
    pub god_items_list: Vec<GodStyleItems>,
    pub engage_weapon: [i32; 21],
    pub custom_engage_weapon: [i32; 20],
}

impl EngageItemList {
    pub fn get_weapon_from_god_mut(&mut self, style: usize, god: usize, slot: usize) -> Option<&mut EngageItem> {
        let index = self.god_items_list[god].item[slot][style];
        self.item_list.iter_mut().find(|x| x.item_index == index)
    }
    pub fn get_god_weapon_mask(&self, index: usize) -> i32  {
        if index < 20 { self.engage_weapon[index]  } else { self.custom_engage_weapon[index - 20 ] }
    }

    pub fn add_list(&mut self, item: &ItemData) {
        let iid = item.iid.to_string();
        let index = item.parent.index;
        if item.usetype == 15 || item.usetype >= 36 || item.kind == 9  || item.range_o == 255 || item.icon.is_none() || iid.contains("ディザスタ") || iid.contains("エンゲージ技") { return; }
        if self.item_list.iter_mut().any(|x| x.item_index == index) { return; } // Already in the List or its an Tiki / Engage Atk only Weapon
        let is_bow = item.kind == 4;
        let weapon = !(item.kind == 7 || item.kind >= 9);

        let new_item = EngageItem::new(index, weapon, is_bow);
        self.item_list.push(new_item);
    }
    // Get all engage items from GodGrowthData.LevelData
    pub fn intialize_list(&mut self){
        if self.item_list.len() != 0 { return; }
        EMBLEM_LIST.get().unwrap().iter()
            .flat_map(|&h| GodData::try_get_hash_mut(h))
            .flat_map(|god| god.get_level_data())
            .enumerate()
            .for_each(|(_god, level_data)|{
                let mut style = GodStyleItems::new();
                level_data[0].style_items.iter().enumerate().for_each(|(s, list)|{
                    let mut item_iter = list.iter();
                    style.item[0][s] = item_iter.next().map_or(0, |item| item.parent.index); // 1st Engage Weapon 
                    style.item[1][s] = item_iter.next().map_or(0, |item| item.parent.index); // 2nd Engage Weapon
                    style.item[2][s] = item_iter.next().map_or(0, |item| item.parent.index); // 3rd Engage Weapon
                });
                self.god_items_list.push(style);
            }
        );
        ItemData::get_list().unwrap().iter().filter(|item| item.flag.value & 128 == 128 ).for_each(|item| self.add_list(item));
        self.item_list.iter_mut().for_each(|x| x.replaced_index = 0);
    }
    pub fn randomize_list(&mut self, list: &Vec<&mut GodData>, rng: &Random){
        let s_list = &mut self.item_list;
        let mut avail_weapons: Vec<_> = s_list.iter().map(|x| (x.item_index, x.weapon, x.is_bow)).collect();
        let mut engage_bows: Vec<_> = s_list.iter().filter(|x| x.is_bow).map(|x| x.item_index).collect();
    
        // Change 1st Engage Weapon to match Engage Attack
        // Astra Storm Bow Randomization
        list.iter()
            .enumerate()
            .for_each(|(god, data)|{
                let astra_storm_slot = if data.engage_attack.is_some_and(|atk| atk.to_string().contains("リンエンゲージ技")) { 0 }
                else if data.engage_attack_link.is_some_and(|atk| atk.to_string().contains("リンエンゲージ技")) && god != 13 { 1 }
                else { 3 };
                if astra_storm_slot < 3 {
                    let bow_index = crate::utils::get_random_and_remove(&mut engage_bows, rng).map_or(0, |f| f);
                    for style in 0..9{
                        if let Some(item) = self.get_weapon_from_god_mut(style, god, astra_storm_slot) { item.replaced_index = bow_index;  }
                    }
                }
            }
        );
        list.iter()
            .enumerate()
            .for_each(|(god, data)|{
                for slot in 0..3  {
                    if god != 13 { // Not Tiki
                        let non_weapons = can_equip_non_weapons(data);
                        let can_bow = can_engage_bow(data);
                        let mut selection: Vec<_> = avail_weapons.iter().filter(|x| ( slot == 0 && ( x.1 == !non_weapons || non_weapons) || ( x.2 == can_bow || can_bow ) ) || slot != 0 )
                            .map(|x| x.0).collect();
                        for style in 0..9 {
                            if let Some(item) = self.get_weapon_from_god_mut(style, god, slot).filter(|item| item.replaced_index < 1) {
                                item.replaced_index = crate::utils::get_random_element(&mut selection, rng).map_or(0, |f| *f);
                            }
                        }
                        let replacement_item_index  = self.get_weapon_from_god_mut(0, god, slot).map_or(0, |f| f.replaced_index);
                        // Remove Only the 1st Style Item 
                        if let Some(pos) = avail_weapons.iter().position(|x| x.0 == replacement_item_index){ avail_weapons.remove(pos); }
                    }
                }
            }
        );
    }
    pub fn reset(&mut self){
        self.item_list.iter_mut().for_each(|x| x.replaced_index = 0);
        self.engage_weapon = [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66, 258, 0];
        self.custom_engage_weapon = [0; 20];
    }
    pub fn get_replacement(&self, item_index: i32) -> &'static mut ItemData {
        if let Some(found) = self.item_list.iter().find(|x| x.item_index == item_index) {
            let index = if found.replaced_index <= 3 { found.item_index } else {  found.replaced_index };
            ItemData::try_index_get_mut(index)
        }
        else { ItemData::try_index_get_mut(item_index) }.unwrap()
    }
    pub fn get_replacement_iid(&self, iid: &'static Il2CppString) -> &'static Il2CppString {
        if let Some(item) = ItemData::get(iid) {
            let item_index = item.parent.index; 
            let replacement_item = self.get_replacement(item_index);
            replacement_item.iid
        }
        else {  iid  }
    }
    pub fn add_weapon_flag(&mut self, god_index: i32, item: &ItemData){
        if item.kind == 0 { return; }
        if item.kind == 7 || item.kind >= 9 { return; }
        if god_index < 20 {
            self.engage_weapon[god_index as usize] |= ( 1 << item.kind );
        }
        else {
            let index = god_index - 20;
            self.custom_engage_weapon[index as usize] |= ( 1 << item.kind );
        }
    }
    pub fn commit(&mut self){
        EMBLEM_LIST.get().unwrap().iter()
            .flat_map(|&h| GodData::try_get_hash_mut(h))
            .flat_map(|god| god.get_level_data())
            .enumerate()
            .for_each(|(god, level_data)|{
                if god == 13 {  self.engage_weapon[13] = 512; }
                else {
                    if  god < 20 { self.engage_weapon[god] = 0; } else { self.custom_engage_weapon[god - 20] = 0; }
                    level_data.iter_mut()
                        .flat_map(|level| level.style_items.items.iter_mut())
                        .flat_map(|list| list.iter_mut())
                        .for_each(|item|{
                            *item = self.get_replacement(item.parent.index);
                            self.add_weapon_flag(god as i32, item);
                        }
                    );
                }
            }
        );
    }
    pub fn print(&self, emblem: i32, level: i32) -> String {
        let set: HashSet<i32> = self.god_items_list[emblem as usize].item[level as usize].iter().map(|v| *v).collect();
        if set.len() == 1 {
            let index = set.iter().next().map_or_else(||0, |f|*f);
            let item = self.get_replacement(index);
            if item.parent.index > 2 { return Mess::get(item.name).to_string(); }
            else { return String::from("None");}
        }
        else {
            let mut style_masks: Vec<i32> = Vec::new();
            set.iter()
                .for_each(|&item|{
                    let mut mask = 0;
                    for x in 1..9 {
                        if self.god_items_list[emblem as usize].item[level as usize][x] == item { mask |= (1 << (x as i32)); }
                    }
                    style_masks.push(mask);
                }
            );
            let mut out = String::new();
            set.iter().zip(style_masks.iter())
                .for_each(|(item, style)| {
                    if *style != 0 {
                        let name = Mess::get(self.get_replacement(*item).name);
                        let mut mask_name = String::new();
                        for x in 1..9 {
                            if *style & (1 << x) != 0 {
                                let style_name = Mess::get(concat_string!("MBSID_", STYLE[x as usize])).to_string();
                                if !mask_name.is_empty() { mask_name.push_str("/"); }
                                mask_name.push_str(style_name.as_str());
                            }
                        }
                        if !mask_name.is_empty() {
                            if !out.is_empty() { out.push_str(", "); }
                            out.push_str(name.to_string().as_str()); 
                            out.push_str(" (");
                            out.push_str(mask_name.as_str());
                            out.push_str(")");
                        }
                    }
                }
            );
            return out;
        }
    }
}

pub fn can_engage_bow(god: &GodData) -> bool { engage_atk_can_use_bow(god.engage_attack) && engage_atk_can_use_bow(god.engage_attack_link) }
pub fn can_equip_non_weapons(god: &GodData) -> bool { engage_atk_can_equip_non_weapons(god.engage_attack) && engage_atk_can_equip_non_weapons(god.engage_attack_link) }

pub fn engage_atk_can_use_bow(engage_atk: Option<&Il2CppString>) -> bool {
    engage_atk.map_or_else(||true, |atk|SkillData::get(atk).map_or(true, |engage| engage.weapon_prohibit.value & 32 != 0 || engage.weapon_prohibit.value == 0))
}

pub fn engage_atk_can_equip_non_weapons(engage_atk: Option<&Il2CppString>) -> bool {
    engage_atk.map_or_else(||true, |atk| SkillData::get(atk).map_or(true, |engage| engage.weapon_prohibit.value == 0))
}

pub fn randomized_emblem_apts() {
    if crate::randomizer::RANDOMIZER_STATUS.read().unwrap().emblem_apt_randomized { return; }
    let mode = GameVariableManager::get_number(DVCVariables::WEAPON_PROF_KEY);
    if mode == 0  { return; }
    let rng = crate::utils::get_rng();
    EMBLEM_LIST.get().unwrap().iter()
        .flat_map(|&h| GodData::try_get_hash_mut(h))
        .for_each(|god|{ 
            randomize_god_apts(god, mode, rng); 
        }
    );
    let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.emblem_apt_randomized = true);
}

fn randomize_god_apts(god: &GodData, mode: i32, rng: &Random) {
    let ggid = god.get_grow_table();
    if ggid.is_none() { return; }
    let growth_id = ggid.unwrap().to_string();
    let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
    let grow_data = GodGrowthData::try_get_from_god_data(god).unwrap();
    if mode == 1 {  // Randomized
        let mut weapons_set: [bool; 8] = [false; 8];
        let mut apt: [i32; 25] = [0; 25];
        let mut current_apt = 0;
        let max = crate::utils::min(level_data.len() as i32, 24) as usize;
        let gmax =  crate::utils::min(grow_data.len() as i32, 24) as usize;
        for y in 1..max {
            apt[y] = level_data[y].aptitude.value;
        }
        let mut count = 0;
        let mut kind: usize;

        for y in 2..max {
            if apt[y] == apt[y-1] { 
                level_data[y].aptitude.value = current_apt; 
                continue; 
            }
            loop {
                kind = rng.get_value(8) as usize;
                if !weapons_set[kind] { break; }
            }
            current_apt |= 1 << ( kind + 1);
            level_data[y].aptitude.value = current_apt; 
            weapons_set[kind] = true;
            count += 1;
            if y < gmax { grow_data[y-1].aptitude.value = 1 << ( kind + 1); }
        }
        if count < 3 {
            loop {
                kind = rng.get_value(8) as usize;
                if !weapons_set[kind] { break; }
            }
            current_apt |= 1 << ( kind + 1);
            level_data[max-1].aptitude.value = current_apt; 
            grow_data[gmax-1].aptitude.value = 1 << ( kind + 1); 
        }
        level_data[0].aptitude.value = current_apt;
    }
    else {  // None
        level_data[0].aptitude.value = 0;
        grow_data.iter_mut().for_each(|level|level.aptitude.value = 0);
        level_data.iter_mut().for_each(|level| level.aptitude.value = 0);
    }
}

pub struct EmblemWeaponProfs;
impl ConfigBasicMenuItemSwitchMethods for EmblemWeaponProfs {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_weap_prof_mode, 0, 2, 1);
        if CONFIG.lock().unwrap().emblem_weap_prof_mode != result {
            CONFIG.lock().unwrap().emblem_weap_prof_mode = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().emblem_weap_prof_mode {
            1 => { "Emblems weapon proficiencies will be randomized." },
            2 => { "Emblems will not give any weapon proficiencies." },
            _ => { "Emblem weapon proficiencies will not be changed."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().emblem_weap_prof_mode {
            1 => { "Randomized" },
            2 => { "None" },
            _ => { "Default"},
        }.into();
    }
}