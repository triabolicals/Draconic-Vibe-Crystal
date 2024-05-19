use unity::prelude::*;
use engage::{
    mess::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gameuserdata::*,
    random::*,
    gamedata::{*, unit::*, item::*, god::*},
};
use std::sync::Mutex;
use super::CONFIG;
use crate::{skill::EMBLEM_ASSET,person};

// Contains methods of random items, engage items, and jobs

pub static ENGAGE_ITEMS: Mutex<EngageItemList> = Mutex::new(EngageItemList{ item_list: Vec::new(), god_items_list: Vec::new(), engage_weapon: [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66, 258, 0], });

const BLACKLIST_ITEMS: [&str; 25] = [
    "IID_マスタープルフ", "IID_リベラシオン改", "IID_リベラシオン改_ノーマル",
    "IID_リベラシオン", "IID_リベラシオン_M000", "IID_無し", "IID_不明", "IID_エンゲージ枠", "IID_火炎砲台", "IID_牙", "IID_邪竜石_E",
    "IID_邪竜石_E005", "IID_邪竜石_魔法攻撃_E", "IID_イル_反撃", "IID_イル_薙払いビーム", "IID_イル_突進",
    "IID_イル_吸収", "IID_イル_召喚", "IID_火のブレス", "IID_炎塊", "IID_ソンブル_物理攻撃",
    "IID_ソンブル_魔法攻撃", "IID_ソンブル_回転アタック", "IID_ソンブル_ビーム", "IID_ソンブル_エンゲージブレイク", ];

pub struct GodStyleItems {
    pub items: [i32; 27],
}
const STYLE: [&str;9] = ["Default", "Cooperation", "Horse", "Covert", "Heavy", "Fly", "Magic","Prana", "Dragon"];
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
                let index2 = self.god_items_list[x as usize].items[9+z as usize];
                if index2 != -1 {
                    let item2 = self.get_replacement(index2);
                    for y in 10..level_data.len() { level_data[y].style_items.add_item(z, item2); }
                    level_data[0].style_items.add_item(z, item2);
                    self.add_weapon_flag(x as i32, item2);
                }
                let index_3 = self.god_items_list[x as usize].items[18+z as usize];
                if index_3 != -1 {
                    let item3 = self.get_replacement(index_3);
                    for y in 15..level_data.len() { level_data[y].style_items.add_item(z, item3); }
                    level_data[0].style_items.add_item(z, item3);
                    self.add_weapon_flag(x as i32, item3);
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

// Item Randomization and replacement
pub fn is_smash(item: &UnitItem) -> bool {
    let e_skills = item.get_equipped_skills();
    if e_skills.is_some() { return e_skills.unwrap().find_sid("SID_スマッシュ".into()).is_some();  }
    return false;
}

pub fn is_thunder(item: &UnitItem) -> bool {
    let e_skills = item.get_equipped_skills();
    if e_skills.is_some() {
        return e_skills.unwrap().find_sid("SID_轟雷発動可能".into()).is_some();
    }
    return false;
}
pub fn is_slim(item: &UnitItem) -> bool {
    let iid = item.item.iid.get_string().unwrap();
    if iid == "IID_ほそみの剣" { return true; }
    if iid == "IID_ほそみの槍" { return true; }
    if iid == "IID_ショートアクス" { return true; }
    return false; 
}
pub fn is_magic_weapon(item: &UnitItem) -> bool {
    let iid = item.item.iid.get_string().unwrap();
    if iid == "IID_いかづちの剣" { return true; }
    if iid == "IID_ほのおの槍" { return true; }
    if iid == "IID_かぜの大斧" { return true; }
    if iid == "IID_光の弓" { return true;}
    return false; 
}
pub fn is_crit_weapon(item: &UnitItem) -> bool {
    let iid = item.item.iid.get_string().unwrap();
    if iid == "IID_キルソード" { return true; }
    if iid == "IID_キラーランス" { return true; }
    if iid == "IID_キラーアクス" { return true; }
    if iid == "IID_キラーボウ" { return true;}
    if iid == "IID_エルサージ" { return true;}
    return false; 
}
pub fn range_weapon(item: &UnitItem) -> bool {
    let iid = item.item.iid.get_string().unwrap();
    if iid == "IID_手槍" { return true; }
    if iid == "IID_スレンドスピア" { return true; }
    if iid == "IID_手斧" { return true; }
    if iid == "IID_トマホーク" { return true;}
    if iid == "IID_長弓" { return true;}
    return false;
}
pub fn is_effective_weapon(item: &UnitItem) -> bool {
    let iid = item.item.iid.get_string().unwrap();
    if iid == "IID_アーマーキラー" { return true; }
    if iid == "IID_ナイトキラー" { return true; }
    if iid == "IID_ポールアクス" { return true; }
    return false;
}

pub fn replace_weapon(item: &UnitItem, weapon_mask: i32, max_rank: i32) {
    println!("Replace item {}, weapon mask {}, max level {}", item.item.name.get_string().unwrap(), weapon_mask, max_rank);
    let kind = item.item.get_kind();
    let mut level = item.item.get_weapon_level();
    if max_rank < level { level = max_rank; }
    let mut new_weapon_type: i32  = -1;
    for x in 1..9 {
        if weapon_mask & ( 1 << x ) != 0 {
            new_weapon_type = x-1;
            break;
        }
    }
    if new_weapon_type < 0 { 
        let rng = Random::get_game().get_value(3);
        new_weapon_type = rng;
        if new_weapon_type < 0 || new_weapon_type > 2 { new_weapon_type = 0; }
    }
    if new_weapon_type == 7 { new_weapon_type = 6; }
    if weapon_mask & 512 != 0 {
        return;
    }
    let mut weapons: [&str; 7];
    // standard set 
    if level == 1 { weapons = ["IID_鉄の剣", "IID_鉄の槍", "IID_鉄の斧", "IID_鉄の弓", "IID_鉄のナイフ", "IID_ファイアー", "IID_鉄身の法"]; }
    else if level == 2 { weapons = ["IID_鋼の剣", "IID_鋼の槍", "IID_鋼の斧", "IID_鋼の弓", "IID_鋼のナイフ", "IID_エルファイアー", "IID_鋼身の法"]; }
    else if level == 3 { weapons = ["IID_銀の剣", "IID_銀の槍", "IID_銀の斧", "IID_銀の弓", "IID_銀のナイフ", "IID_エルファイアー", "IID_銀身の法"]; }
    else { weapons = ["IID_勇者の剣", "IID_勇者の槍", "IID_勇者の斧", "IID_勇者の弓", "IID_ペシュカド", "IID_ボルガノン", "IID_閃進の法"]; }

    // Tome
    if kind == 6 {
        // Thunder Related Set
        if is_thunder(item) {
            if level == 1 { weapons = ["IID_鉄の剣", "IID_手槍", "IID_ショートアクス", "IID_ショートボウ", "IID_ショートナイフ", "IID_サンダー", "IID_初心の法"]; }
            else if level == 2 { weapons = ["IID_いかづちの剣", "IID_ほのおの槍", "IID_手斧", "IID_長弓", "IID_カルド", "IID_エルサンダー", "IID_護身の法"]; }
            else if level == 3 { weapons = ["IID_ドラゴンキラー", "IID_スレンドスピア", "IID_トマホーク", "IID_光の弓", "IID_スティレット", "IID_エルサンダー", "IID_護身の法"]; }
            else { weapons = ["IID_ドラゴンキラー", "IID_スレンドスピア", "IID_トマホーク", "IID_光の弓", "IID_ペシュカド", "IID_トロン", "IID_閃進の法"]; }
        }
    }
    else if range_weapon(item ) {
        if level == 1 { weapons[1] = "IID_手槍"; weapons[2] = "IID_手斧"; }
        else if level == 2 { weapons[3] = "IID_長弓"; weapons[1] = "IID_手槍"; weapons[2] = "IID_手斧"; }
        else {
            weapons[3] = "IID_長弓";
            weapons[2] = "IID_トマホーク";
            weapons[1] = "IID_スレンドスピア";
        }
    }
    else if is_slim(item) { weapons = ["IID_ほそみの剣","IID_ほそみの槍", "IID_ショートアクス", "IID_ショートボウ", "IID_ショートナイフ" , "IID_サージ", "IID_初心の法"]; }
    else if is_crit_weapon(item) { weapons[0] = "IID_キルソード"; weapons[1] = "IID_キラーランス"; weapons[2] = "IID_キラーアクス"; weapons[3] = "IID_キラーボウ"; weapons[5] = "IID_エルサージ"; }
    else if is_magic_weapon(item) {
        weapons[0] = "IID_いかづちの剣"; weapons[1] = "IID_ほのおの槍"; weapons[2] = "IID_かぜの大斧"; weapons[3] = "IID_光の弓";
    }
    else if is_smash(item) {
        if level == 2 { weapons[0] = "IID_鉄の大剣"; weapons[1] = "IID_鉄の大槍"; weapons[2] = "IID_鉄の大斧"; }
        else if level == 3 { weapons[0] = "IID_鋼の大剣";  weapons[1] = "IID_鋼の大槍"; weapons[2] = "IID_鋼の大斧"; }
        else if level == 4 { weapons[0] = "IID_銀の大剣";  weapons[1] = "IID_銀の大槍";  weapons[2] = "IID_銀の大斧"; }
    }
    else if is_effective_weapon(item) {
        weapons[0] = "IID_アーマーキラー";
        weapons[1] = "IID_ナイトキラー";
        weapons[2] = "IID_ポールアクス";
    }
    if new_weapon_type < 0 || new_weapon_type > 6 { return; }
    let flag = item.flags;
    item.ctor_str(weapons[new_weapon_type as usize]);
    item.set_flags(flag);
}

//Has Healing staff
pub fn replace_staves(item_list: &UnitItemList){
    for x in 0..item_list.get_count()+3 {
        let item = item_list.get_item(x);
        if item.is_some() {
            let staff = &item.unwrap();
            let kind = staff.item.get_kind(); 
            if kind == 7 { 
                let staff_name = staff.item.iid.get_string().unwrap();
                if staff_name == "IID_ライブ" || staff_name == "IID_リブロー" || staff_name == "IID_リライブ" { 
                    staff.ctor_str("IID_傷薬");
                }
                else { staff.ctor_str("IID_特効薬"); }
            }
        }
    }
}
pub fn dispose_staves(item_list: &UnitItemList){
    for x in 0..item_list.get_count()+3 {
        let item = item_list.get_item(x);
        if item.is_some() {
            let staff = &item.unwrap();
            let kind = staff.item.get_kind(); 
            if kind == 7 { staff.dispose(); }
        }
    }
}
pub fn remove_duplicates(item_list: &UnitItemList) {
    for x in 0..item_list.get_count()+3 {
        let item = item_list.get_item(x);
        if item.is_some() {
            let unit_item = item.unwrap();
            if unit_item.is_drop() { continue; }
            let iid1 =  unit_item.item.iid.get_string().unwrap();
            if iid1 == "IID_エンゲージ枠" || iid1 == "IID_不明" { continue; } 
            if iid1 == "IID_無し" { continue; }
            for y in x+1..item_list.get_count()+3 {
                let item2 = item_list.get_item(y);
                if item2.is_some() {
                    let unit_item2 = item2.unwrap(); 
                    if unit_item2.is_drop() { continue; }
                    let iid2 =  unit_item2.item.iid.get_string().unwrap();
                    if iid2 == iid1 { unit_item2.dispose(); }
                }
            }
        }
    }
    for x in 0..item_list.unit_items.len() {
        let item = item_list.unit_items[x];
        let iid1 =  item.item.iid.get_string().unwrap();
        if iid1 == "IID_エンゲージ枠" || iid1 == "IID_不明" { continue; } 
        if item.is_empty() {
            for y in x+1..item_list.unit_items.len() {
                if !item_list.unit_items[y].is_empty() {
                    item_list.move_item(y as i32, x as i32);
                    break;
                }
            }
        }
    }
}
pub fn adjust_staffs(unit: &Unit) {
    let job = unit.get_job();
    let weapon_mask = job.get_weapon_mask();
    remove_duplicates(unit.item_list);
    let is_vander = GameVariableManager::get_string("G_R_PID_ヴァンドレ").get_string().unwrap() == unit.person.pid.get_string().unwrap();
    if weapon_mask.value & 64 != 0 && !is_vander {
        if job.is_low() && unit.level < 15 { unit.item_list.add_item_no_duplicate(ItemData::get("IID_ファイアー").unwrap());  }
        if unit.level >= 15 && job.is_low() { unit.item_list.add_item_no_duplicate(ItemData::get("IID_サンダー").unwrap()); }
        if ( unit.level >= 15 && job.is_low() )|| job.is_high() {  unit.item_list.add_item_no_duplicate(ItemData::get("IID_エルファイアー").unwrap());  }
        if job.is_high() { unit.item_list.add_item_no_duplicate(ItemData::get("IID_エルサンダー").unwrap()); }
    }
    if weapon_mask.value & ( 1 << 7 ) == 0 { replace_staves(unit.item_list); }
    else {
        dispose_staves(unit.item_list);
        if job.is_low() {
            if job.jid.get_string().unwrap() == "JID_スレイプニル下級" {    //Fracture for Wing Tamer Hortensia
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_コラプス").unwrap()); 
            }
            if unit.level < 10 { // Heal
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_ライブ").unwrap()); 
            }
            else if unit.level < 15 { // Mend
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_リライブ").unwrap()); 
            }
            else {  // Mend + Physic
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_リライブ").unwrap()); 
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_リブロー").unwrap()); 
            }
        }
        else {
            if job.jid.get_string().unwrap() == "JID_スレイプニル" {    // Fracture for Sleipnir Rider
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_コラプス").unwrap()); 
            } 
            if job.jid.get_string().unwrap() == "JID_ハイプリースト" {    // Warp/Fortify for High Priest
                  unit.item_list.add_item_no_duplicate(ItemData::get("IID_ワープ").unwrap()); 
                  unit.item_list.add_item_no_duplicate(ItemData::get("IID_リザーブ").unwrap());
            }
            else {
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_リライブ").unwrap());  // mend
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_リブロー").unwrap());  // physic for the rest of staffers instead
            }
        }
        if unit.person.get_asset_force() != 0 {
            let rng = Random::get_game();
            let value = rng.get_value(100);
            if value < 30 {
                if GameVariableManager::get_bool("G_Cleared_M019") {
                    if job.get_max_weapon_level(7) >= 4 {
                        unit.item_list.add_item_no_duplicate(ItemData::get("IID_ドロー").unwrap()); //Entrap
                    }
                    else {  unit.item_list.add_item_no_duplicate(ItemData::get("IID_フリーズ").unwrap());  } // Freeze
                }
                else if GameVariableManager::get_bool("G_Cleared_M009") {
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_フリーズ").unwrap());   // Freeze
                }
                else { unit.item_list.add_item_no_duplicate(ItemData::get("IID_コラプス").unwrap());   }    // Fracture
            }
            else if value < 60 {
                if GameVariableManager::get_bool("G_Cleared_S006") || GameVariableManager::get_bool("G_Cleared_M018") { unit.item_list.add_item_no_duplicate(ItemData::get("IID_ワープ").unwrap()); }  //Warp  
                else { unit.item_list.add_item_no_duplicate(ItemData::get("IID_サイレス").unwrap()); }  //Silence
            }
        }
    }
    let pid = unit.person.pid.get_string().unwrap();
    if pid == "PID_ヴェイル" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_オヴスキュリテ").unwrap()); 
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_ミセリコルデ").unwrap());
        magic_dagger_weapon_change(unit.get_job());
    }
    if pid == "PID_エル" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_邪竜石").unwrap()); 
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_邪竜石_騎馬特効").unwrap()); 
    }
    if pid == "PID_ラファール" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_邪竜石").unwrap()); 
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_邪竜石_騎馬特効").unwrap()); 
    }
    if unit.get_job().jid.get_string().unwrap() == "JID_マージカノン" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_弾_物理").unwrap()); 
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_弾_魔法").unwrap()); 
    }
    remove_duplicates(unit.item_list);
}

pub fn unit_change_to_random_class(unit: &mut Unit){
    let rng = Random::get_game();
    let job_count = JobData::get_count();
    let is_female = unit.person.get_gender() == 2;
    let job_list = JobData::get_list().unwrap();
    let mut is_high = false;
    if unit.get_job().is_low() { is_high = false; }
    if unit.level >= 20 || unit.get_job().is_high() { is_high = true; }
    let unit_level = unit.level as i32;
    let internal_level = unit.internal_level as i32;
    let mut count = 0;
    println!("Unit Level {} / Internal {}", unit_level, internal_level);
    loop {
        let index = rng.get_value(2*job_count);
        if index >= job_count { continue; }
        let job = &job_list[index as usize];
        if job.jid.get_string().unwrap() == "JID_マージカノン" { continue;}
        let job_flags = job.get_flag();
        if ( job_flags.value & 16 != 0 ) && is_female { continue; }
        if job_flags.value & 1 == 0 && job_flags.value & 2 == 0 { count += 1; continue;}
        if job_flags.value == 0 { continue;}
        if job_flags.value & 1 == 1 && job_flags.value & 2 == 0 { 
            if !is_high {
                if index % 4 == 0 {                 
                    if unit.person.get_job().unwrap().get_flag().value & 2 == 0 && unit.person.get_job().unwrap().is_low() {
                        unit.class_change(unit.person.get_job().unwrap());
                    }
                    else { unit.class_change(JobData::get("JID_マージ").unwrap()); }
                }
                else if index % 4 == 1 { unit.class_change(JobData::get("JID_モンク").unwrap()); }
                else if index % 4 == 2 { unit.class_change(JobData::get("JID_アーチャー").unwrap()); }
                else if index % 4 == 3 { unit.class_change(JobData::get("JID_シーフ").unwrap()); }
                else {
                    count += 1;
                    continue;
                }
                unit.set_level(unit_level); 
                unit.set_internal_level(internal_level);
                unit.set_hp(unit.get_capability(0, true));
                unit.set_weapon_mask_from_person();
                person::fixed_unit_weapon_mask(unit);
                return;
            }
            else { 
                count += 1;
                continue;
            }
        }
        if job_flags.value & 1 == 1 && job_flags.value & 2 == 0 { count += 1; continue;}
        if (job_flags.value & 4 == 4 ) && !is_female { count+=1; continue; }  // if female only and not a female
        if (!is_high && job.is_high() ) || (is_high && job.is_low()) {
            count += 1;
            continue; 
        } // if promoted and new class is not promoted
        if unit.get_job().jid.get_string().unwrap() == job.jid.get_string().unwrap() { 
            count += 1;
            continue;
        }
        if job.jid.get_string().unwrap() == "JID_マージカノン" && !GameVariableManager::get_bool("G_CC_マージカノン") { 
            count += 1;
            continue;
        }
        if job.jid.get_string().unwrap() == "JID_エンチャント" && !GameVariableManager::get_bool("G_CC_エンチャント") { 
            count += 1;
            continue;
        }
        unit.class_change(job);
        if unit_level > 20 && job.is_high() { 
            unit.set_level(unit_level - 20); 
            unit.set_internal_level(internal_level+20);
        }
        else if unit_level == 20 && job.is_high() {
            unit.set_level(1); 
            unit.set_internal_level(internal_level+19);
        }
        else { 
            unit.set_level(unit_level); 
            unit.set_internal_level(internal_level);
        }
        println!("{} changed to {} from {} in {} steps (Lv {}/{})", 
            unit.person.get_name().unwrap().get_string().unwrap(), 
            job.name.get_string().unwrap(),  
            unit.get_job().name.get_string().unwrap(), count, unit.level, unit.internal_level);

        unit.set_hp(unit.get_capability(0, true));
        person::fixed_unit_weapon_mask(unit);

        return;
    }
}
pub fn enemy_unit_change_to_random_class(unit: &mut Unit) -> bool {
    let current_job = unit.get_job();
    let current_flags = current_job.get_flag().value;
    if current_flags == 0 || current_job.parent.index < 10 { return false; }  // If 
    if current_job.name.get_string().unwrap() == "MJID_Emblem" { return false; }
    let rng = Random::get_game();
    let job_count = JobData::get_count();
    let is_female = unit.person.get_gender() == 2;
    let job_list = JobData::get_list().unwrap();
    let mut is_high = false;
    if unit.get_job().is_low() { is_high = false; }
    if unit.level >= 20 || unit.get_job().is_high() { is_high = true; }
    let is_flying = unit.get_job().move_type == 3;
    let unit_level = unit.level as i32;
    let internal_level = unit.internal_level as i32;
    let has_emblem = unit.get_god_unit().is_some() || ( GameUserData::get_chapter().cid.get_string().unwrap() != "CID_M011" );
    loop {
        let index = rng.get_value(job_count);
        let job = &job_list[index as usize];
        let job_flags = job.get_flag();
        let jid = job.jid.get_string().unwrap();
        if ( job_flags.value & 16 != 0 ) && is_female { continue; }
        if job_flags.value <= 1 { continue; }
        if (job_flags.value & 4 == 4 ) && !is_female {  continue; } 
        if jid == "JID_異形飛竜" || jid == "JID_幻影飛竜" { continue; } //Wyverns
        if jid == "JID_異形竜" || jid == "JID_幻影竜" { continue; } //Wyrms
        if jid == "JID_村人" { continue; }  // Villager
        if (!is_high && job.is_high() ) || (is_high && job.is_low()) {
            continue; 
        } // if promoted and new class is not promoted
        if unit.get_job().jid.get_string().unwrap() == job.jid.get_string().unwrap() { 
            return false;
        }
        if has_emblem && ( jid == "JID_異形狼" || jid == "JID_幻影狼" ) {   // has emblem and is either wolf class
            continue;
        }
        unit.class_change(job);
        println!("Person #{}: {}:  Class Change to #{} {}", unit.person.parent.index, Mess::get(unit.person.get_name().unwrap()).get_string().unwrap(), job.parent.index, Mess::get(job.name).get_string().unwrap());

        if job.move_type != 3 && is_flying {
            if !unit.private_skill.add_sid("SID_天駆", 10, 0)  { continue; }
            if job.move_type == 2 {
                unit.private_skill.add_sid("SID_移動－１", 10, 0); 
                unit.private_skill.add_sid("SID_移動－３", 10, 0);
            }
            else { unit.private_skill.add_sid("SID_移動－２", 10, 0); }
        }
        if unit_level > 20 && job.is_high() { 
            unit.set_level(unit_level - 20); 
            unit.set_internal_level(internal_level+20);
        }
        else if unit_level == 20 && job.is_high() {
            unit.set_level(1); 
            unit.set_internal_level(internal_level+19);
        }
        else { 
            unit.set_level(unit_level); 
            unit.set_internal_level(internal_level);
        }
        unit.set_hp(unit.get_capability(0, true));
        person::fixed_unit_weapon_mask(unit);
        return true;
    }
}

pub fn random_items_drops(unit: &Unit){
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_some() {
            let u_item = &mut item.unwrap();
            if u_item.is_drop() && !u_item.is_equip() { 
                let new_item = get_random_item(u_item.item.iid, false);
                u_item.ctor_str(&new_item.get_string().unwrap());
                u_item.flags = 2;
            }
        }
    }
}
fn magic_dagger_weapon_change(veyle_job: &JobData){
    // Change Veyle's Dagger to whatever class weapon she has. (low class only)
    if veyle_job.is_high() { 
        GameVariableManager::make_entry("G_Misercode_Type", 5); 
        GameVariableManager::set_number("G_Misercode_Type", 5);
        return; 
    }
    let kinds = veyle_job.get_equippable_item_kinds();
    let mut misercode_type = 5; //Dagger
    for i in 0..kinds.len() {
        if kinds[i] == 7 || kinds[i] >= 9 { continue; }
        if kinds[i] == 0 { continue; }
        misercode_type = kinds[i];
    }
    let misercode = ItemData::get_mut("IID_ミセリコルデ").unwrap();
    misercode.kind = misercode_type as u32;
    misercode.get_give_skills().clear();
    misercode.get_equip_skills().clear();
    if misercode_type == 4 {
        misercode.range_o = 2; misercode.range_i = 2;
        misercode.set_cannon_effect("弓砲台".into());
        misercode.on_complete();
        misercode.get_equip_skills().add_sid("SID_飛行特効",4, 0);
    }
    else if misercode_type == 5 || misercode_type == 6 {
        misercode.range_i = 1; misercode.range_o = 2;
        if misercode_type == 6 {
            misercode.set_cannon_effect("魔砲台炎".into());
            misercode.set_hit_effect( "オヴスキュリテ".into());
        }
        else if misercode_type == 5 { misercode.get_give_skills().add_sid("SID_毒",3, 0); }
        misercode.on_complete();
    }
    else if misercode_type == 8 {  misercode.get_equip_skills().add_sid("SID_２回行動",4,0); }
    else {
        misercode.range_i = 1;
        misercode.range_o = 2;
    }
    GameVariableManager::make_entry("G_Misercode_Type", misercode_type);
    GameVariableManager::set_number("G_Misercode_Type", misercode_type);
}

// For item replacement
pub fn get_random_item(item: &'static Il2CppString, allow_rare: bool) -> &'static Il2CppString {
    let item_list = ItemData::get_list().unwrap();
    let rng = Random::get_game();
    let item_check = ItemData::get(&item.get_string().unwrap());
    // if Item is rare
    if item_check.is_some() {
        let flag = item_check.unwrap().get_flag().value;
        if flag & 1 == 1 { return item;  }
        let iid = item_check.unwrap().iid.get_string().unwrap();
        if BLACKLIST_ITEMS.iter().find(|x| **x == iid).is_some() { return item; }
    }
    else { return item; }
    let item_list_size = item_list.len();
    loop {
        let item_index = rng.get_value( item_list_size as i32 ) as usize;
        let random_item = &item_list[item_index];
        let item_flag = random_item.get_flag().value;
        let mut skip = false;
        let iid = random_item.iid.get_string().unwrap();
        if BLACKLIST_ITEMS.iter().find(|x| **x == iid).is_some() { continue; }
        if !has_name(random_item, true) { continue; }
        if random_item.is_unknown() || random_item.is_inventory() || random_item.is_material() { continue; }
        if crate::utils::str_contains(random_item.name, "MIID_Ring") { continue; }
        if item_flag & 16777216 != 0 { continue; } //Bless
        if item_flag & 33554432 != 0 { continue; } //Breath
        if item_flag & 67108864 != 0 { continue; }  //Dragon
        if item_flag & 134217728 != 0 { continue; } //Bullet
        if item_flag & 131072 != 0 { continue; } // Bento
        if item_flag & 32768 != 0 { continue; } // AI 
        for y in 0..8 {
            if y == 2 { continue; }
            if y == 0 && allow_rare { continue; }
            if item_flag & (1 << y ) != 0 {                
                skip = true;
                break;
            }
        }
        if !skip { return random_item.iid; }
    }
}
pub fn get_random_gift_item(include_rare: bool) -> &'static Il2CppString {
    let item_list = ItemData::get_list().unwrap();
    let rng = Random::get_game();
    let item_list_size = item_list.len() as i32;
    loop {
        let item_index = rng.get_value( item_list_size ) as usize;
        let random_item = &item_list[item_index];
        let item_flag = random_item.get_flag().value;
        let mut skip = false;
        let iid = random_item.iid.get_string().unwrap();
        if crate::utils::str_contains(random_item.name, "MIID_Ring") {  //BondRing Mod Removal
            continue;
        }
        if BLACKLIST_ITEMS.iter().find(|x| **x == iid).is_some() { continue; }
        if !has_name(random_item, true) { continue; }
        if random_item.is_unknown() { continue; }
        if random_item.usetype >= 32 && random_item.usetype <= 39 { continue; }
        if random_item.usetype == 0 && ( random_item.kind != 17 && random_item.kind != 18 ){ continue; }    //Not Bond/Money
        if item_flag & 16777216 != 0 { continue; } //Bless
        if item_flag & 33554432 != 0 { continue; } //Breath
        if item_flag & 67108864 != 0 { continue; }  //Dragon
        if item_flag & 131072 != 0 { continue; }    //Bento
        if item_flag & 134217728 != 0 { continue; } //Bullet
        if item_flag & 32768 != 0 { continue; } // AI 
        for y in 0..8 {
            if y == 2 { continue; }
            if y == 0 && include_rare { continue; }
            if item_flag & (1 << y ) != 0 {
                skip = true;
                break;
            }
        }
        if !skip { return random_item.iid;  }
    }
}

fn has_name(this: &ItemData, include_money: bool) -> bool {
    unsafe {  if crate::utils::is_null_empty(this.name, None) { return false;  }  }
    let item_name = Mess::get(this.name ).get_string().unwrap();
    if item_name.len() != 0 { return true }
    else if include_money {
        return this.kind == 17 || this.kind == 18  ;    // If Money or bond
    }
    return false; 
}

pub fn randomize_well_rewards() {
    if GameVariableManager::get_number("G_Random_Item") == 0  { return; }
    if CONFIG.lock().unwrap().random_gift_items != 0 {
        let rare_item = CONFIG.lock().unwrap().random_gift_items == 2;
        let rlist = RewardData::get_list_mut().unwrap();
        for x in 0..rlist.len() {
            for y in 0..rlist[x].len() {
                let iid = rlist[x][y].iid;
                let price = ItemData::get(&iid.get_string().unwrap()).unwrap().price;
                let mut new_iid = get_random_gift_item(rare_item);  
                let mut new_price = ItemData::get(&new_iid.get_string().unwrap()).unwrap().price;
                let mut count = 0;
                while new_price > 3*price && count < 50  {
                    new_iid = get_random_gift_item(rare_item); 
                    new_price = ItemData::get(&new_iid.get_string().unwrap()).unwrap().price;
                    count += 1;
                }
                rlist[x][y].set_iid(new_iid);  
            }
        }
    }
    let well_reward_list = ["アイテム交換_期待度１", "アイテム交換_期待度２", "アイテム交換_期待度３", "アイテム交換_期待度４", "アイテム交換_期待度５" ];
    for x in well_reward_list {
        let well_list = RewardData::try_get_mut(x);
        if well_list.is_none() { continue; }
        let well_items = well_list.unwrap();
        let mut in_set: [bool; 1000] = [false; 1000];
        for y in 0..well_items.len() {
            let iid = well_items[y as usize].iid;
            let price = ItemData::get(&iid.get_string().unwrap()).unwrap().price;
            let mut new_price; 
            let mut item_index;
            let mut new_iid; 
            let curent_reward = &well_items[y as usize];
            let mut count = 0;
            loop {
                new_iid = get_random_gift_item(true); 
                new_price = ItemData::get(&new_iid.get_string().unwrap()).unwrap().price;
                item_index = ItemData::get(&new_iid.get_string().unwrap()).unwrap().parent.index;
                if new_price > 3*price { count += 1; continue; }
                if count < 50 && in_set[item_index as usize] { count += 1; continue; }
                if count >= 50 { break; }
                if !in_set[item_index as usize] { break; }
            }
            let new_reward = RewardData::instantiate().unwrap();
            new_reward.ctor();
            new_reward.set_iid(new_iid);
            let new_item = ItemData::get(&new_iid.get_string().unwrap()).unwrap();
            if new_item.get_flag().value & 1 != 0 || ( new_item.kind == 18 || new_item.kind == 17 ) {   // If rare or money / bond
                new_reward.ratio = 2.5;
                new_reward.min = 2.5;
                new_reward.max = 2.5;
            }
            else {
                new_reward.ratio = 1.5*curent_reward.ratio;
                new_reward.min = 1.5*curent_reward.min;
                new_reward.max = 1.5*curent_reward.max;
            }
            well_items.add(new_reward);
            in_set[item_index as usize] = true; 
        }
    }
    println!("Complete Randomization of Gift/Well Items");
}

pub struct RandomJobMod;
impl ConfigBasicMenuItemSwitchMethods for RandomJobMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_job, 0, 3, 1);
        if CONFIG.lock().unwrap().random_job != result {
            CONFIG.lock().unwrap().random_job  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_job;
        if selection == 1 {  this.help_text = "Playable units will be in random classes.".into(); }
        else if selection == 2 {  this.help_text = "Enemy/NPC units will be in random classes.".into(); }
        else if selection == 3 { this.help_text = "All units will be in random classes.".into(); }
        else { this.help_text = "Units will be in their default classes".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_job;
        if selection == 1 { this.command_text = "Player".into(); }
        else if selection == 2 { this.command_text = "Enemy / NPC".into(); }
        else if selection == 3 { this.command_text = "Player + Enemy / NPC".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}
pub struct RandomItemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomItemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_item, 0, 3, 1);
        if CONFIG.lock().unwrap().random_item != result {
            CONFIG.lock().unwrap().random_item  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_item;
        if selection == 1 {  this.help_text = "Items obtained from chests/villages will be random.".into(); }
        else if selection == 2 {  this.help_text = "Item drops from enemies will be random.".into(); }
        else if selection == 3 {  this.help_text = "Item obtained from events and enemy drops will be random.".into(); } 
        else { this.help_text = "No changes made to item events or item drops.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_item;
        if selection == 1 { this.command_text = "Events".into(); }
        else if selection == 2 { this.command_text = "Drops".into(); }
        else if selection == 3 { this.command_text = "Events + Drops".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}

pub struct RandomGiftMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGiftMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_gift_items, 0, 2, 1);
        if CONFIG.lock().unwrap().random_gift_items != result {
            CONFIG.lock().unwrap().random_gift_items  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_gift_items;
        if selection == 1 {  this.help_text = "No rare items will be included when randomizing gift item lists. (Toggle)".into(); }
        else if selection == 2 {  this.help_text = "Rare items will be included when randomizing gift item lists. (Toggle)".into(); } 
        else { this.help_text = "No randomization done to gift items. (Toggle)".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_gift_items;
        if selection == 1 { this.command_text = "No Rare Items".into(); }
        else if selection == 2 { this.command_text = "With Rare Items".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}

pub struct RandomEngageWepMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEngageWepMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_engage_weapon);
        if CONFIG.lock().unwrap().random_engage_weapon != result {
            CONFIG.lock().unwrap().random_engage_weapon  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_engage_weapon {  this.help_text = "Engage Items/Weapons are randomized".into(); }
        else { this.help_text = "No changes to Engage items/weapons.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_engage_weapon { this.command_text = "Randomized".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}