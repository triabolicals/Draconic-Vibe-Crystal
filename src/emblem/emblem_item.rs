use unity::prelude::*;
use engage::{
    mess::*,
    random::*,
    gamedata::{*, item::*, god::*},
};
use std::sync::Mutex;
use crate::enums::*;

pub static ENGAGE_ITEMS: Mutex<EngageItemList> = Mutex::new(EngageItemList{ item_list: Vec::new(), god_items_list: Vec::new(), engage_weapon: [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66, 258, 0], });

pub struct GodStyleItems {
    pub items: [i32; 27],
}

impl GodStyleItems {
    fn new() -> Self { Self { items: [-1; 27], } }
}
pub struct EngageItem {
    pub item_index: i32,
    pub god_index: i32,
    pub weapon: bool,
    pub is_bow: bool,
    pub is_first_item: bool,
    // Stuff for what the Item gets randomized into
    pub god_can_bow: bool,  
    pub god_can_weapon: bool,
    pub replaced_index: i32,
    pub reverse_index: i32,
    pub in_used: bool,
    // Stuff for text replacement
    pub original_emblem: i32,
    pub new_emblem: i32,
    pub miid: String,
}

impl EngageItem {
    fn new(itemdata_index: i32, god: i32, not_weapon: bool, bow: bool, first: bool, emblem_index: i32) -> Self {
        Self {
            item_index: itemdata_index, 
            god_index: god, 
            weapon: not_weapon, 
            is_bow: bow, 
            is_first_item: first,
            god_can_bow: false, god_can_weapon: false, 
            replaced_index: -1, 
            reverse_index: -1,
            in_used: false, 
            original_emblem: emblem_index,
            new_emblem: -1,
            miid: "".to_string(),
        }
    }
}

pub struct EngageItemList {
    pub item_list: Vec<EngageItem>,
    pub god_items_list: Vec<GodStyleItems>,
    pub engage_weapon: [i32; 21],
}

impl EngageItemList {
    pub fn add_list(&mut self, item: &ItemData, god: i32, is_first: bool, emblem_index: i32) {
        let index = item.parent.index;
        let found = self.item_list.iter_mut().find(|x| x.item_index == index);
        if found.is_some() { return; } 
        let weapon;
        let is_bow = item.kind == 4 ;
        if item.kind == 7 || item.kind >= 9  { weapon = false; }
        else { weapon = true; }
        let mut new_item = EngageItem::new(index, god, weapon, is_bow, is_first, emblem_index);
        new_item.miid = item.help.get_string().unwrap();
        self.item_list.push(new_item);
    }
    pub fn bow_randomization(&mut self, rng: &Random) {
        // find all emblems that have astra storm as an engage attack 
        let s_list = &mut self.item_list;
        let mut bow_weapons: Vec<(usize,bool)> = Vec::new();
        for x in 0..s_list.len() { if s_list[x].is_bow { bow_weapons.push( (x, false) ); }  }

        let list_size = bow_weapons.len();
        println!("{} Engage Bows", list_size);

        for x in 0..20 {
            let gid = format!("GID_{}", EMBLEM_ASSET[x]);
            let god = GodData::get(&gid).unwrap();
            if god.get_engage_attack().get_string().unwrap() == "SID_リンエンゲージ技" {
                let starting_index = s_list.iter().position(|r| r.god_index == ( x as i32 ) ).unwrap();
                let mut index = rng.get_value(list_size as i32) as usize;

                while bow_weapons[index].1 { index = rng.get_value(list_size as i32) as usize; }
                bow_weapons[index].1 = true;

                s_list[starting_index].replaced_index = bow_weapons[index].0 as i32;
                s_list[ bow_weapons[index].0 ].in_used = true;
                s_list[ bow_weapons[index].0 ].reverse_index = starting_index as i32;
                s_list[ bow_weapons[index].0 ].new_emblem = s_list[starting_index].original_emblem;
            }
            if god.get_engage_attack_link().is_none() { continue; }

            if god.get_engage_attack_link().unwrap().get_string().unwrap() == "SID_リンエンゲージ技" {
                let starting_index = s_list.iter().position(|r| r.god_index == ( x as i32 ) && !r.is_first_item).unwrap();
                let mut index = rng.get_value(list_size as i32) as usize;

                while bow_weapons[index].1 { index = rng.get_value(list_size as i32) as usize; }
                bow_weapons[index].1 = true;
                s_list[starting_index].replaced_index = bow_weapons[index].0 as i32;

                s_list[ bow_weapons[index].0 ].in_used = true;
                s_list[ bow_weapons[index].0 ].reverse_index = starting_index as i32;
                s_list[ bow_weapons[index].0 ].new_emblem = s_list[starting_index].original_emblem;
            }
        }
    }
    // Get all engage items from GodGrowthData.LevelData
    pub fn intialize_list(&mut self){
        if self.item_list.len() != 0 { return; }
        for x in 0..20 {
            let mut style = GodStyleItems::new();
            let growth_id = format!("GGID_{}", EMBLEM_ASSET[x]);
            let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
            for z in 0..9 {
                if level_data[0].style_items.items[z].len() >= 1 { style.items[z as usize] = level_data[0].style_items.items[z][0].parent.index; }
                if level_data[0].style_items.items[z].len() >= 2 { style.items[9+z as usize] = level_data[0].style_items.items[z][1].parent.index; }
                if level_data[0].style_items.items[z].len() >= 3 { style.items[18+z as usize] = level_data[0].style_items.items[z][2].parent.index; }
            }
            self.god_items_list.push(style);
            if x == 13 { continue; }  //  Ignore adding Tiki items into the randomization pool
            for y in 1..level_data.len() {
                let is_first = y < 10;
                for z in 0..9 {
                    if level_data[y].style_items.items[z].len() != 0 {
                        for aa in 0..level_data[y].style_items.items[z].len() {
                            if x == 9 { //Byleth
                                if z < 2 && is_first { self.add_list(&level_data[y].style_items.items[z][aa], x as i32, is_first, x as i32); }
                                else { self.add_list(&level_data[y].style_items.items[z][aa], x as i32, false, x as i32); }
                            }
                            else { self.add_list(&level_data[y].style_items.items[z][aa], x as i32, is_first, x as i32); }
                        }
                    }
                }
            }
        }
    }
    pub fn randomize_list(&mut self, rng: &Random){
        let list_size = self.item_list.len() as i32;
        let item_list = ItemData::get_list().unwrap();
        for x in 0..20 {
            if x == 13 { continue; }      // ignore Tiki
            let gid = format!("GID_{}", EMBLEM_ASSET[x]);
            let god = GodData::get(&gid).unwrap();
            let can_bow: bool;
            let non_weapons: bool;
            if god.get_engage_attack_link().is_some() {
                can_bow = can_engage_bow(&god.get_engage_attack().get_string().unwrap()) && can_engage_bow(&god.get_engage_attack_link().unwrap().get_string().unwrap());
                non_weapons = can_equip_non_weapons(&god.get_engage_attack().get_string().unwrap()) && can_equip_non_weapons(&god.get_engage_attack_link().unwrap().get_string().unwrap());
            }
            else {
                can_bow = can_engage_bow(&god.get_engage_attack().get_string().unwrap());
                non_weapons = can_equip_non_weapons(&god.get_engage_attack().get_string().unwrap());
            }
            for y in 0..list_size {
                if  self.item_list[y as usize].god_index < x.try_into().unwrap() { continue; }
                else if  self.item_list[y as usize].god_index > x.try_into().unwrap() { break; }
                self.item_list[y as usize].god_can_bow = can_bow;
                self.item_list[y as usize].god_can_weapon = non_weapons;
            }
        }
        self.bow_randomization(rng);
        let s_list = &mut self.item_list;
        for x in 0..list_size {
            let mut index = rng.get_value(list_size) as usize;
            //Randomization of Engage Items
            let mut count = 0;
            if s_list[x as usize].replaced_index != -1 { 
                println!("Engage Item Swap: {} to {}, {} -> {}", x, 
                    index, Mess::get(item_list[ s_list[x as usize].item_index as usize].name).get_string().unwrap(), 
                    Mess::get(item_list[ s_list[index].item_index as usize].name).get_string().unwrap() );
                continue; 
            } //Already Randomized
            loop {
                count += 1;
                if count == 50 { break;}
                if s_list[index].in_used { index = rng.get_value(list_size) as usize; continue; }
                if s_list[index].is_bow && !s_list[x as usize].god_can_bow { 
                    // If not the first engage item, then able to use a bow
                    if !s_list[x as usize].is_first_item { break; }
                    else { index = rng.get_value(list_size) as usize;  continue;  }
                }
                if s_list[index].is_bow && s_list[x as usize].god_can_bow { break; }
                if !s_list[index].weapon {
                    if s_list[x as usize].is_first_item { index = rng.get_value(list_size) as usize; continue; }
                    else if s_list[x as usize].god_can_weapon { break; }
                }
                if s_list[index].weapon { break;}
                index = rng.get_value(list_size) as usize;
            }
            println!("Engage Item Swap: {} to {}, {} -> {}", x, index, 
            Mess::get(item_list[ s_list[x as usize].item_index as usize].name).get_string().unwrap(),
            Mess::get(item_list[ s_list[index].item_index as usize].name).get_string().unwrap() );
            s_list[x as usize].replaced_index = index as i32;
            s_list[index].in_used = true;
            s_list[index].reverse_index = x as i32;
            s_list[index].new_emblem = s_list[x as usize].original_emblem;
        }
    }
    pub fn reset(&mut self){
        let s_list = &mut self.item_list;
        let list_size = s_list.len();
        for x in 0..list_size {
            s_list[x as usize].in_used = false; 
            s_list[x as usize].replaced_index = -1;
            s_list[x as usize].reverse_index = -1;
        }
        self.engage_weapon = [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66, 258, 0];
    }
    pub fn get_replacement(&self, item_index: i32) -> &'static ItemData {
        let item_list = ItemData::get_list().unwrap();
        let found = self.item_list.iter().find(|x| x.item_index == item_index);
        if found.is_some() {
            let new_index = found.unwrap().replaced_index;
            if new_index == -1 {
                return &item_list[item_index as usize];
            }
            let new_item_index = self.item_list[new_index as usize].item_index;
            return &item_list[new_item_index as usize];
        }
        else { return &item_list[item_index as usize]; }
    }
    pub fn get_replacement_iid(&self, iid: &'static Il2CppString) -> &'static Il2CppString {
        let item = ItemData::get(&iid.get_string().unwrap());
        if item.is_none() { return iid; }
        let item_index = item.unwrap().parent.index; 
        let replacement_item = self.get_replacement(item_index);
        return replacement_item.iid;
    }
    pub fn add_weapon_flag(&mut self, god_index: i32, item: &ItemData){
        if item.kind == 0 { return; }
        if item.kind == 7 || item.kind >= 9 { return; }
        self.engage_weapon[god_index as usize] = self.engage_weapon[god_index as usize] | ( 1 << item.kind );
    }
    pub fn commit(&mut self){
        for x in 0..20 {
            if x == 13 { 
                self.engage_weapon[13] = 512;
                continue; 
            }
            self.engage_weapon[x as usize] = 0;
            let growth_id = format!("GGID_{}", EMBLEM_ASSET[x]);
            let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
            for y in 0..level_data.len() { level_data[y].style_items.clear();  }
            for z in 0..9 {
                let index = self.god_items_list[x as usize].items[z as usize];
                if index != -1 {
                    let item = self.get_replacement(index);
                    for y in 0..level_data.len() { level_data[y].style_items.add_item(z, item); }
                    self.add_weapon_flag(x as i32, item);
                }
                else {
                    println!("Emblem {} Weapon 1, {} can't be found", x, z );
                }
                let index2 = self.god_items_list[x as usize].items[9+z as usize];
                if index2 != -1 {
                    let item2 = self.get_replacement(index2);
                    for y in 10..level_data.len() { level_data[y].style_items.add_item(z, item2); }
                    level_data[0].style_items.add_item(z, item2);
                    self.add_weapon_flag(x as i32, item2);
                }
                else {
                    println!("Emblem {} Weapon 2, {} can't be found", x, z );
                }
                let index_3 = self.god_items_list[x as usize].items[18+z as usize];
                if index_3 != -1 {
                    let item3 = self.get_replacement(index_3);
                    for y in 15..level_data.len() { level_data[y].style_items.add_item(z, item3); }
                    level_data[0].style_items.add_item(z, item3);
                    self.add_weapon_flag(x as i32, item3);
                }
                else {
                    println!("Emblem {} Weapon 2, {} can't be found", x, z );
                }
            }
        }
    }
    pub fn print(&self, emblem: i32, level: i32) -> String {
        let mut out = "".to_string();
        let mut unique_items: Vec<(i32,i32)> = Vec::new();
        let start;
        let end;
        if level == 0 { start = 0; end = 9;  }
        else if level == 1 { start = 9; end = 18; }
        else { start = 18; end = 27; }
        for i in start..end {
            let item_i = self.god_items_list[emblem as usize].items[i as usize];
            if unique_items.iter().find(|x| item_i == x.0).is_none() {
                unique_items.push( (item_i, i % 9) );
            }
        }
        for x in unique_items {
            if x.0 == -1 { continue; }
            let item = self.get_replacement(x.0);
            if x.1 == 0 { out = format!("{} {}", out, Mess::get(item.name).get_string().unwrap()); }
            else {
                let style_name = Mess::get(&format!("MBSID_{}", STYLE[x.1 as usize])).get_string().unwrap();
                out = format!("{} {} ({})", out, Mess::get(item.name).get_string().unwrap(), style_name);
            }
        }
        return out;
    }
}

pub fn can_engage_bow(engage_atk: &String) -> bool {
    if engage_atk == "SID_マルスエンゲージ技" { return false; }
    if engage_atk == "SID_シグルドエンゲージ技" { return false; }
    if engage_atk == "SID_ロイエンゲージ技" { return false;}
    if engage_atk == "SID_ルキナエンゲージ技" { return false;}
    if engage_atk == "SID_アイクエンゲージ技" { return false;}
    if engage_atk == "SID_エイリークエンゲージ技" { return false; }
    if engage_atk == "SID_クロムエンゲージ技" {return false;}
    if engage_atk == "SID_ヘクトルエンゲージ技" || engage_atk == "SID_ヘクトルエンゲージ技＋" { return false; }
    if engage_atk == "SID_クロムエンゲージ技" || engage_atk == "SID_クロムエンゲージ技＋" {return false;}
    if engage_atk == "SID_カミラエンゲージ技" || engage_atk == "SID_カミラエンゲージ技＋" { return false;}
    if engage_atk == "SID_リュールエンゲージ技" || engage_atk == "SID_リュールエンゲージ技共同" { return false;}
    return true;
}
pub fn can_equip_non_weapons(engage_atk: &String) -> bool {
    if engage_atk == "SID_マルスエンゲージ技" { return false; }
    if engage_atk == "SID_シグルドエンゲージ技" { return false; }
    if engage_atk == "SID_ロイエンゲージ技" { return false;}
    if engage_atk == "SID_ルキナエンゲージ技" { return false;}
    if engage_atk == "SID_アイクエンゲージ技" { return false;}
    if engage_atk == "SID_エイリークエンゲージ技" { return false; }
    if engage_atk == "SID_リンエンゲージ技" { return false; }
    if engage_atk == "SID_クロムエンゲージ技" || engage_atk == "SID_クロムエンゲージ技＋" {return false;}
    if engage_atk == "SID_カミラエンゲージ技" || engage_atk == "SID_カミラエンゲージ技＋" { return false;}
    if engage_atk == "SID_ヘクトルエンゲージ技" || engage_atk == "SID_ヘクトルエンゲージ技＋" { return false; }
    if engage_atk == "SID_リュールエンゲージ技" || engage_atk == "SID_リュールエンゲージ技共同" { return false;}
    return true;
}