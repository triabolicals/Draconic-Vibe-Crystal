use unity::prelude::*;
use engage::{
    mess::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gameuserdata::*,
    random::*,
    gamedata::{*, unit::*, skill::SkillData, item::*, god::*},
};
use std::sync::Mutex;
use super::CONFIG;
use crate::{skill::EMBLEM_ASSET,person};

pub static ENGAGE_ITEMS: Mutex<EngageItemList> = Mutex::new(EngageItemList{ item_list: Vec::new(), god_items_list: Vec::new(), engage_weapon: [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66], });

const BLACKLIST_ITEMS: [&str; 25] = [
    "IID_マスタープルフ", "IID_リベラシオン改", "IID_リベラシオン改_ノーマル",
    "IID_リベラシオン", "IID_リベラシオン_M000", "IID_無し", "IID_不明", "IID_エンゲージ枠", "IID_火炎砲台", "IID_牙", "IID_邪竜石_E",
    "IID_邪竜石_E005", "IID_邪竜石_魔法攻撃_E", "IID_イル_反撃", "IID_イル_薙払いビーム", "IID_イル_突進",
    "IID_イル_吸収", "IID_イル_召喚", "IID_火のブレス", "IID_炎塊", "IID_ソンブル_物理攻撃",
    "IID_ソンブル_魔法攻撃", "IID_ソンブル_回転アタック", "IID_ソンブル_ビーム", "IID_ソンブル_エンゲージブレイク", ];

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
}

impl EngageItem {
    fn new(itemdata_index: i32, god: i32, not_weapon: bool, bow: bool, first: bool) -> Self {
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
        }
    }
}

pub struct EngageItemList {
    pub item_list: Vec<EngageItem>,
    pub god_items_list: Vec<GodStyleItems>,
    pub engage_weapon: [i32; 19],
}

impl EngageItemList {
    pub fn add_list(&mut self, item: &ItemData, god: i32, is_first: bool) {
        let index = item.parent.index;
        let found = self.item_list.iter_mut().find(|x| x.item_index == index);
        if found.is_some() { return; } 
        let weapon;
        let is_bow =  ( item.kind == 4 );
        if item.kind == 7 || item.kind >= 9  { weapon = false; }
        else { weapon = true; }
        self.item_list.push(EngageItem::new(index, god, weapon, is_bow, is_first));

    }
    // Get all engage items from GodGrowthData.LevelData
    pub fn intialize_list(&mut self){
        if self.item_list.len() != 0 { return; }
        for x in 0..19 {
            let mut style = GodStyleItems::new();
            if x == 13 { 
                self.god_items_list.push(style);
                continue; 
            }    // ignore Tiki
            let growth_id = format!("GGID_{}", EMBLEM_ASSET[x]);
            let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
            for y in 1..level_data.len() {
                let is_first = y < 10;
                for z in 0..9 {
                    if level_data[y].style_names.items[z].len() != 0 {
                        for aa in 0..level_data[y].style_names.items[z].len() {
                            self.add_list(&level_data[y].style_names.items[z][aa], x as i32, is_first);
                        }
                    }
                }
            }
            for z in 0..9 {
                if level_data[1].style_names.items[z].len() >= 1 {
                    style.items[z as usize] = level_data[1].style_names.items[z][0].parent.index;
                }
                if level_data[10].style_names.items[z].len() >= 2 {
                    style.items[9+z as usize] = level_data[10].style_names.items[z][1].parent.index;
                }
                if level_data[0].style_names.items[z].len() >= 3 {
                    style.items[18+z as usize] = level_data[0].style_names.items[z][2].parent.index;
                }
            }
            self.god_items_list.push(style);
        }
    }
    pub fn randomize_list(&mut self, rng: &Random){
        let list_size = self.item_list.len() as i32;
        let s_list = &mut self.item_list;
        for x in 0..19 {
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
                if s_list[y as usize].god_index < x.try_into().unwrap() { continue; }
                else if s_list[y as usize].god_index > x.try_into().unwrap() { break; }
                s_list[y as usize].god_can_bow = can_bow;
                s_list[y as usize].god_can_weapon = non_weapons;
            }
        }
        for x in 0..list_size {
            println!("{}, weapon: {}, is_bow: {}, is_firs_item: {}, god_can bow {}, god_can_weapon {}", x, s_list[x as usize].weapon, s_list[x as usize].is_bow,s_list[x as usize].is_first_item, s_list[x as usize].god_can_bow, s_list[x as usize].god_can_weapon);
        }
        for x in 0..list_size {
            let mut index = rng.get_value(list_size) as usize;
            loop {
                if s_list[index].in_used { index = rng.get_value(list_size) as usize; }
                if s_list[index].is_bow && !s_list[x as usize].god_can_bow { index = rng.get_value(list_size) as usize; }
                if s_list[index].is_bow && s_list[x as usize].god_can_bow { break; }
                if !s_list[index].weapon {
                    if s_list[x as usize].is_first_item && s_list[x as usize].god_can_weapon  { break; }
                    else if !s_list[x as usize].is_first_item { break; }
                }
                if s_list[index].weapon { break;}
                index = rng.get_value(list_size) as usize;
            }
            println!("Item Index {} -> {}", x, index);
            s_list[x as usize].replaced_index = index as i32;
            s_list[index].in_used = true;
            s_list[index].reverse_index = x as i32;
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
        self.engage_weapon = [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66];
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
    pub fn add_weapon_flag(&mut self, god_index: i32, item: &ItemData){
        if item.kind == 0 { return; }
        if item.kind == 7 || item.kind >= 9 { return; }
        self.engage_weapon[god_index as usize] = self.engage_weapon[god_index as usize] | ( 1 << item.kind );
    }
    pub fn commit(&mut self){
        for x in 0..19 {
            if x == 13 { 
                self.engage_weapon[13] = 512;
                continue; 
            }
            self.engage_weapon[x as usize] = 0;
            let growth_id = format!("GGID_{}", EMBLEM_ASSET[x]);
            let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
            for y in 0..level_data.len() { level_data[y].style_names.clear();  }
            for z in 0..9 {
                let index = self.god_items_list[x as usize].items[z as usize];
                if index != -1 {
                    let item = self.get_replacement(index);
                    for y in 0..level_data.len() { level_data[y].style_names.add_item(z, item); }
                    self.add_weapon_flag(x as i32, item);
                }
                let index2 = self.god_items_list[x as usize].items[9+z as usize];
                if index2 != -1 {
                    let item2 = self.get_replacement(index2);
                    for y in 10..15 { level_data[y].style_names.add_item(z, item2); }
                    level_data[0].style_names.add_item(z, item2);
                    self.add_weapon_flag(x as i32, item2);
                }
                let index3 = self.god_items_list[x as usize].items[18+z as usize];
                if index3 != -1 {
                    let item3 = self.get_replacement(index2);
                    for y in 15..level_data.len() { level_data[y].style_names.add_item(z, item3); }
                    level_data[0].style_names.add_item(z, item3);
                    self.add_weapon_flag(x as i32, item3);
                }
            }
        }
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
    return true;
}
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
    }
    let name = unit.person.get_name().unwrap().get_string().unwrap();
    if name == "MPID_Veyre"  {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_オヴスキュリテ").unwrap()); 
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_ミセリコルデ").unwrap());
    }
    if name == "MPID_El" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_邪竜石").unwrap()); 
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_邪竜石_騎馬特効").unwrap()); 
    }
    if name == "MPID_Rafale" {
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
        if job_flags.value == 0 { continue; }
        if job_flags.value & 1 == 0 && job_flags.value & 2 == 0 { count += 1; continue;}
        if (job_flags.value & 1 == 1 && job_flags.value & 2 == 0) && index % 3 == 0 { 
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
        println!("{} changed to {} from {} in {} steps (Lv {}/{})", unit.person.get_name().unwrap().get_string().unwrap(), job.name.get_string().unwrap(),  unit.get_job().name.get_string().unwrap(), count, unit.level, unit.internal_level);
        unit.set_hp(unit.get_capability(0, true));
        //unit.set_weapon_mask_from_person();
        person::fixed_unit_weapon_mask(unit);

        return;
    }
}
pub fn enemy_unit_change_to_random_class(unit: &mut Unit) -> bool {
    if unit.get_job().get_flag().value == 0 || unit.get_job().get_flag().value == 1 {
        return false;
    }
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
        let index = rng.get_value(2*job_count);
        if index >= job_count { continue; }
        let job = &job_list[index as usize];
        let job_flags = job.get_flag();
        let jid = job.jid.get_string().unwrap();
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
        if job.move_type != 3 && is_flying {
            unsafe {
                let skill = &SkillData::get("SID_天駆").unwrap();
                if !person::unit_add_private_skill(unit, skill, None) { continue; }
                if job.move_type == 2 {
                    person::unit_add_private_skill(unit, SkillData::get("SID_移動－１").unwrap(), None);
                    person::unit_add_private_skill(unit, SkillData::get("SID_移動－３").unwrap(), None);
                }
                else {
                    person::unit_add_private_skill(unit, SkillData::get("SID_移動－２").unwrap(), None);
                }
            }

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
        //unit.set_weapon_mask_from_person();
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
                let new_item = get_random_item(u_item.item.iid);
                u_item.ctor_str(&new_item.get_string().unwrap());
                u_item.flags = 2;
            }
        }
    }
}
pub fn get_random_item(item: &'static Il2CppString) -> &'static Il2CppString {
    println!("Random Item Called");
    let item_list = ItemData::get_list().unwrap();
    let rng = Random::get_game();
    let item_check = ItemData::get(&item.get_string().unwrap());
    // if Item is rare
    if item_check.is_some() {
        unsafe {
            let flag = item_data_flag(item_check.unwrap(), None).value;
            if flag & 1 == 1 { return item;  }
            let iid = item_check.unwrap().iid.get_string().unwrap();
            if BLACKLIST_ITEMS.iter().find(|x| **x == iid).is_some() { return item; }
        }
    }
    else {
        return item;
    }
    let item_list_size = item_list.len();
    unsafe {
        loop {
            let item_index = rng.get_value( item_list_size as i32 ) as usize;
            let random_item = &item_list[item_index];
            let item_flag = item_data_flag(random_item, None).value;
            let mut skip = false;
            let iid = random_item.iid.get_string().unwrap();
            if BLACKLIST_ITEMS.iter().find(|x| **x == iid).is_some() { continue; }
            if !has_name(random_item) { continue; }
            if item_data_is_unknown(random_item, None) { continue; }
            if item_data_is_inventory(random_item, None) { continue; }
            if item_data_is_material(random_item, None) { continue; }
            if item_flag & 16777216 != 0 { continue; } //Bless
            if item_flag & 33554432 != 0 { continue; } //Breath
            if item_flag & 67108864 != 0 { continue; }  //Dragon
            if item_flag & 134217728 != 0 { continue; } //Bullet
            if item_flag & 131072 != 0 { continue; } // Bento
            if item_flag & 32768 != 0 { continue; } // AI 
            for y in 0..8 {
                if y == 2 { continue; }
                if item_flag & (1 << y ) != 0 {
                    skip = true;
                    break;
                }
            }
            if !skip {
                return random_item.iid;
            }
        }
    }
}
#[unity::class("App", "ItemDataFlag")]
pub struct ItemDataFlag {
    pub value: i32,
}
fn has_name(this: &ItemData) -> bool {
    unsafe {
        let name = item_data_get_name(this, None);
        if name.is_some() {
            let item_name = Mess::get( name.unwrap() ).get_string().unwrap();
            return item_name.len() != 0;
        }
        else { return false; }
    }
}
#[unity::from_offset("App", "ItemData", "get_Help")]
pub fn item_data_get_name(this: &ItemData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "ItemData", "get_Flag")]
pub fn item_data_flag(this: &ItemData, method_info: OptionalMethod) -> &'static ItemDataFlag;

#[unity::from_offset("App", "ItemData", "IsInventory")]
pub fn item_data_is_inventory(this: &ItemData, method_info: OptionalMethod) -> bool;

#[unity::from_offset("App", "ItemData", "IsMaterial")]
pub fn item_data_is_material(this: &ItemData, method_info: OptionalMethod) -> bool;

#[unity::from_offset("App", "ItemData", "IsUnknown")]
pub fn item_data_is_unknown(this: &ItemData, method_info: OptionalMethod) -> bool;

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


#[no_mangle]
extern "C" fn job_rnd() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomJobMod>("Random Classes") } 

pub fn install_rnd_jobs() { cobapi::install_global_game_setting(job_rnd); }