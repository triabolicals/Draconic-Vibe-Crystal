use unity::prelude::*;
use engage::{
    mess::*,
    menu::{BasicMenuResult, config::*},
    gamevariable::*,
    gameuserdata::*,
    random::*,
    gamedata::{*, unit::*, item::*},
};
use engage::gamedata::skill::SkillData;
use std::sync::Mutex;

use super::CONFIG;
use crate::{enums::*, person};
pub static RANDOM_ITEM_POOL: Mutex<Vec<i32>> = Mutex::new(Vec::new());
// Contains methods of random items, and jobs
// standard set 
pub const STANDARD_WEPS: [&str; 35] = [
    "IID_鉄の剣", "IID_鉄の槍", "IID_鉄の斧", "IID_鉄の弓", "IID_鉄のナイフ", "IID_ファイアー", "IID_鉄身の法" ,    //D
    "IID_鋼の剣", "IID_鋼の槍", "IID_鋼の斧", "IID_鋼の弓", "IID_鋼のナイフ", "IID_エルファイアー", "IID_鋼身の法", //C
    "IID_銀の剣", "IID_銀の槍", "IID_銀の斧", "IID_銀の弓", "IID_銀のナイフ", "IID_エルファイアー", "IID_銀身の法", //B
    "IID_勇者の剣", "IID_勇者の槍", "IID_勇者の斧", "IID_勇者の弓", "IID_ペシュカド", "IID_ボルガノン", "IID_閃進の法", //A
    "IID_クラドホルグ", "IID_ブリューナク", "IID_フラガラッハ", "IID_レンダウィル", "IID_シンクエディア", "IID_ノヴァ", "IID_覇神の法", //S
];
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
    if item.item.get_flag().value & 128 != 0 { return; }
    if item.item.get_flag().value & 2 != 0 { return; }

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
    println!("Replace item {} , weapon mask {}, max level {}", item.item.name.get_string().unwrap(), weapon_mask, max_rank);
    item.ctor_str(weapons[new_weapon_type as usize]);
    item.set_flags(flag);
    println!("Flags set");
}

//Has Healing staff
pub fn replace_staves(item_list: &UnitItemList){
    for x in 0..8 {
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
    for x in 0..8 {
        let item = item_list.get_item(x);
        if item.is_some() {
            let staff = &item.unwrap();
            let kind = staff.item.get_kind(); 
            if kind == 7 { staff.dispose(); }
        }
    }
}
pub fn remove_duplicates(item_list: &UnitItemList) {
    for x in 0..8 {
        let item = item_list.get_item(x as i32);
        if item.is_some() {
            let unit_item = item.unwrap();
            if unit_item.is_drop() { continue; }
            let iid1 =  unit_item.item.iid.get_string().unwrap();
            if iid1 == "IID_エンゲージ枠" || iid1 == "IID_不明" { continue; } 
            if iid1 == "IID_無し" { continue; }
            for y in x+1..8 {
                let item2 = item_list.get_item(y as i32 );
                if item2.is_some() {
                    let unit_item2 = item2.unwrap(); 
                    if unit_item2.is_drop() { continue; }
                    let iid2 =  unit_item2.item.iid.get_string().unwrap();
                    if iid2 == iid1 { 
                        unit_item2.dispose(); 
                        println!("Dispose of {}", unit_item2.item.name.get_string().unwrap());
                    }
                    // remove vuls if exlixir/antitoxin exists
                    else if (iid1 == "IID_毒消し" || iid1 == "IID_特効薬") && iid2 == "IID_傷薬" { unit_item2.dispose();  }
                    else if iid1 == "IID_傷薬" && (iid2 == "IID_特効薬" || iid2 == "IID_毒消し")  { unit_item.dispose(); break;}
                    
                }
            }
        }
    }
    for x in 0..8 {
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
    let jid = job.jid.get_string().unwrap();
    remove_duplicates(unit.item_list);
    let is_vander = GameVariableManager::get_string("G_R_PID_ヴァンドレ").get_string().unwrap() == unit.person.pid.get_string().unwrap();
    let is_veyle = unit.person.pid.get_string().unwrap() == "PID_ヴェイル";
    if weapon_mask.value & 64 != 0 && (!is_vander && !is_veyle) {
        if job.is_low() { 
            if GameVariableManager::get_bool("G_Cleared_M009") { unit.item_list.add_item_no_duplicate(ItemData::get("IID_エルファイアー").unwrap()); } //Elfire 
            else { unit.item_list.add_item_no_duplicate(ItemData::get("IID_ファイアー").unwrap()); }    //Fire
            if GameVariableManager::get_bool("G_Cleared_M007") { unit.item_list.add_item_no_duplicate(ItemData::get("IID_サンダー").unwrap()); } // Thunder
        }
        if unit.level < 10 && job.is_high()  {  unit.item_list.add_item_no_duplicate(ItemData::get("IID_エルファイアー").unwrap());  }    // Elfire
        else if job.is_high() && unit.level >= 10 { 
            if job.get_max_weapon_level(6) >= 4 { unit.item_list.add_item_no_duplicate(ItemData::get("IID_ボルガノン").unwrap());  }    //Bolg
            else {  unit.item_list.add_item_no_duplicate(ItemData::get("IID_エルファイアー").unwrap()); }   //Elfire
        }
        if job.is_high() && ( unit.level < 10 || job.get_max_weapon_level(6) <= 3 ) { unit.item_list.add_item_no_duplicate(ItemData::get("IID_エルサンダー").unwrap()); }   //Elthunder
        else if job.is_high() && unit.level >= 10 && job.get_max_weapon_level(6) >= 4 { unit.item_list.add_item_no_duplicate(ItemData::get("IID_トロン").unwrap()); }   //Thron
    }
    if weapon_mask.value & ( 1 << 7 ) == 0 { replace_staves(unit.item_list); }
    else if weapon_mask.value & ( 1 << 7 ) != 0 || (person::has_sid(unit, "SID_杖使い＋＋") || person::has_sid(unit, "SID_杖使い＋") || person::has_sid(unit, "SID_杖使い")) {
        dispose_staves(unit.item_list);
        if job.is_low() {
            if jid == "JID_スレイプニル下級" {    //Fracture for Wing Tamer Hortensia
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_コラプス").unwrap()); 
            }
            if unit.level < 10 { // Heal
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_ライブ").unwrap()); 
            }
            else if unit.level < 15 { // Mend
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_リライブ").unwrap()); 
            }
            else {  // Physic
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_リブロー").unwrap()); 
            }
        }
        else {
            if jid == "JID_スレイプニル" {    // Fracture for Sleipnir Rider
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_コラプス").unwrap()); 
            } 
            if jid == "JID_ハイプリースト" {    // Warp/Fortify for High Priest
                  unit.item_list.add_item_no_duplicate(ItemData::get("IID_ワープ").unwrap()); 
                  unit.item_list.add_item_no_duplicate(ItemData::get("IID_リザーブ").unwrap());
            }
            else {
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_リブロー").unwrap());  // physic for the rest of staffers instead
            }
        }
        if unit.person.get_asset_force() != 0 {
            let rng = Random::get_game();
            let value = rng.get_value(100);
            if value < 25 {
                if GameVariableManager::get_bool("G_Cleared_M019") {
                    if job.get_max_weapon_level(7) >= 4 || person::has_sid(unit, "SID_杖使い＋＋")  {
                        unit.item_list.add_item_no_duplicate(ItemData::get("IID_ドロー").unwrap()); //Entrap
                    }
                    else {  
                        unit.item_list.add_item_no_duplicate(ItemData::get("IID_サイレス").unwrap());
                        unit.item_list.add_item_no_duplicate(ItemData::get("IID_フリーズ").unwrap());  } // Freeze
                }
                else if GameVariableManager::get_bool("G_Cleared_M009") {
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_サイレス").unwrap());
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_フリーズ").unwrap());   // Freeze
                }
                else { 
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_サイレス").unwrap());
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_コラプス").unwrap());   }    // Fracture
            }
            else if value < 60 {
                if value < 40 && ( GameVariableManager::get_bool("G_Cleared_S006") || GameVariableManager::get_bool("G_Cleared_M018") ) { 
                    if job.get_max_weapon_level(7) >= 3 {
                        unit.item_list.add_item_no_duplicate(ItemData::get("IID_ワープ").unwrap());
                    }
                    else {
                        unit.item_list.add_item_no_duplicate(ItemData::get("IID_フリーズ").unwrap());  
                        unit.item_list.add_item_no_duplicate(ItemData::get("IID_サイレス").unwrap());
                    }
                }  //Warp  
                else if GameVariableManager::get_bool("G_Cleared_M009") {
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_サイレス").unwrap());
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_フリーズ").unwrap());   // Freeze
                }
                else { 
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_サイレス").unwrap());
                    unit.item_list.add_item_no_duplicate(ItemData::get("IID_コラプス").unwrap());   
                }    // Fracture
            }
            else { 
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_サイレス").unwrap());
                unit.item_list.add_item_no_duplicate(ItemData::get("IID_フリーズ").unwrap()); 
            }
        }
    }
    let pid = unit.person.pid.get_string().unwrap();
    remove_duplicates(unit.item_list);
    if jid == "JID_裏邪竜ノ娘" || jid == "JID_裏邪竜ノ子" {
        unit.private_skill.add_sid("SID_オヴスキュリテ装備可能", 10, 0);    //Equip Obscurite
    }
    if jid == "JID_神竜ノ子" || jid == "JID_神竜ノ王" {
        unit.private_skill.add_sid("SID_リベラシオン装備可能", 10, 0);
        unit.private_skill.add_sid("SID_ヴィレグランツ装備可能", 10, 0);
    }
    fix_weapons_by_rank(unit);
    if is_veyle {
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
    if jid == "JID_マージカノン" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_弾_物理").unwrap()); 
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_弾_魔法").unwrap()); 
    }
    if jid == "JID_邪竜ノ娘" && !is_veyle {
        unit.put_off_all_item();
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_オヴスキュリテ").unwrap()); 
        unit.private_skill.add_sid("SID_オヴスキュリテ装備可能", 10, 0); //Equip Obscurite
        unit.private_skill.add_sid("SID_ミセリコルデ装備可能", 10, 0);  //Equip Misercode
        if unit.person.get_asset_force() == 0 {
            if unit.get_capability(0, false) >= 45 { unit.item_list.add_item_no_duplicate(ItemData::get("IID_特効薬").unwrap());   }
            else { unit.item_list.add_item_no_duplicate(ItemData::get("IID_傷薬").unwrap());  }
        }
    }
    if GameVariableManager::get_bool("G_Cleared_M017") {  add_generic_weapons(unit); }
    remove_duplicates(unit.item_list);
}

pub fn unit_change_to_random_class(unit: &mut Unit){
    let rng = Random::get_game();
    let job_count = JobData::get_count();
    let is_female = if unit.edit.is_enabled() { unit.edit.gender == 2 }  else { unit.person.get_gender() == 2 };
    let job_list = JobData::get_list().unwrap();
    let mut is_high = false;
    if unit.get_job().is_low() { is_high = false; }
    if unit.level >= 20 || unit.get_job().is_high() { is_high = true; }
    let unit_level = unit.level as i32;
    let internal_level = unit.internal_level as i32;
    let mut count = 0;
    println!("Unit Level {} / Internal {}", unit_level, internal_level);
    loop {
        let index = rng.get_value(job_count);
        if index >= job_count { continue; }
        let job = &job_list[index as usize];
        if job.jid.get_string().unwrap() == "JID_マージカノン" { continue;}
        let job_flags = job.get_flag();
        if ( job_flags.value & 16 != 0 ) && ( is_female || unit.person.get_flag().value & 32 != 0 ) { continue; }
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
        if (!is_high && job.is_high() ) || (is_high && job.is_low() && job.jid.get_string().unwrap() != "JID_ダンサー") {
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
        crate::autolevel::unit_update_learn_skill(unit);
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
    let is_female;
    if unit.edit.is_enabled() { is_female = unit.edit.gender == 2; }
    else { is_female = unit.person.get_gender() == 2; }

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
        if ( job_flags.value & 16 != 0 ) && ( is_female || unit.person.get_flag().value & 32 != 0 ) { continue; }
        if job_flags.value <= 1 { continue; }
        if (job_flags.value & 4 == 4 ) && !is_female {  continue; } 
        if jid == "JID_異形飛竜" || jid == "JID_幻影飛竜" { continue; } //Wyverns
        if jid == "JID_異形竜" || jid == "JID_幻影竜" { continue; } //Wyrms
        if jid == "JID_村人" { continue; }  // Villager

        if (!is_high && job.is_high() ) || (is_high && (job.is_low() && job.max_level == 20 ) ) {
            continue; 
        } // if promoted and new class is not promoted
        if unit.get_job().jid.get_string().unwrap() == job.jid.get_string().unwrap() { 
            return false;
        }
        if jid == "JID_異形狼" || jid == "JID_幻影狼"  {
            if has_emblem { continue; }   // has emblem and is either wolf class
            if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M011" { continue; }
        }
        unit.class_change(job);
        println!("Person #{}: {}:  Class Change to #{} {}", 
            unit.person.parent.index, 
            Mess::get(unit.person.get_name().unwrap()).get_string().unwrap(), 
            job.parent.index, Mess::get(job.name).get_string().unwrap()
        );

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
        crate::autolevel::unit_update_learn_skill(unit);
        return true;
    }
}

pub fn fix_weapons_by_rank(unit: &Unit) {
    println!("Fixing weapons by rank for {}", Mess::get(unit.person.get_name().unwrap()).get_string().unwrap());
    let unit_level = unit.level as i32 + unit.internal_level as i32;
    let weapon_rank;
    if !GameVariableManager::get_bool("G_Cleared_M017") { weapon_rank = 0; }
    else if unit_level < 25 { weapon_rank = 2; }
    else { weapon_rank = 3; }

    let job = unit.get_job();
    let pid = unit.person.pid.get_string().unwrap();
    for x in 0..unit.item_list.unit_items.len() {
        let item = unit.item_list.get_item(x as i32);
        if item.is_none() { continue;}
        let weapon = item.unwrap();
        if !weapon.item.is_weapon() { continue; }
        let iid = weapon.item.iid.get_string().unwrap();
        if iid == "IID_エンゲージ枠" || iid == "IID_不明" { continue; } // if engage weapon slot or none skip
        if iid == "IID_ミセリコルデ" && pid != "PID_ヴェイル" {
            weapon.dispose();
            continue;
        }
        let flag = weapon.item.get_flag().value; 
        if flag & 128 != 0 || flag & 2 != 0 { continue; }
        let kind = weapon.item.kind;
        println!("Item {}: {} Kind: {}", x, weapon.item.name.get_string().unwrap(), kind);
        if kind >= 9 || kind == 0 { continue; }
        let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
        if weapon_level == 0 && !weapon.is_drop(){ weapon.dispose(); continue;  }
        if weapon_level < weapon.item.get_weapon_level()  { // Weapon is higher than the Job's rank
            let weapon_index;
            if kind > 8 || weapon_rank == 0 { continue; }
            if kind == 8 { weapon_index = 6;}
            else { weapon_index = (kind - 1 ) as i32;}
            if weapon_rank < weapon_level && weapon_rank > 0 {
                weapon.ctor_str(  STANDARD_WEPS[ ( (7* (weapon_rank - 1) ) + weapon_index ) as usize ] );
            }
            else {
                weapon.ctor_str(  STANDARD_WEPS[ ( (7* (weapon_level - 1) ) + weapon_index ) as usize ] );
            }
        }
        if weapon_rank > 0 && ( weapon.item.get_weapon_level() < weapon_rank && weapon_level >= weapon_rank ) {    // Weapon is lower than level rank and Job can use it
            let weapon_index;
            if kind == 8 { weapon_index = 6;}
            else { weapon_index = (kind - 1 ) as i32;}
            weapon.ctor_str(  STANDARD_WEPS[ ( (7* (weapon_rank - 1) ) + weapon_index ) as usize ] );
        }
    }
}
fn add_generic_weapons(unit: &Unit) {
    //if unit.person.get_asset_force() != 0 { // Making sure enemies have weapons
    println!("Adding Weapons for {}", Mess::get(unit.person.get_name().unwrap()).get_string().unwrap());
        let job = unit.get_job();
        let jid = job.jid.get_string().unwrap();
        if jid == "JID_ボウナイト" { unit.item_list.add_item_no_duplicate(ItemData::get("IID_銀の弓").unwrap());  }
        if jid == "JID_エンチャント" {
            unit.item_list.add_item_no_duplicate(ItemData::get("IID_HPの薬").unwrap());  
            unit.item_list.add_item_no_duplicate(ItemData::get("IID_力の薬").unwrap());  
        }
        if jid == "JID_マージカノン" {
            unit.item_list.add_item_no_duplicate(ItemData::get("IID_弾_竜特効").unwrap());
            unit.item_list.add_item_no_duplicate(ItemData::get("IID_弾_フリーズ").unwrap());
            unit.item_list.add_item_no_duplicate(ItemData::get("IID_弾_騎馬特効").unwrap());
        }
        let weapon_rank;
        let unit_level = unit.level as i32 + unit.internal_level as i32;
        if unit_level < 25 { weapon_rank = 2; }
        else { weapon_rank = 3; }
        let mut has_weapon: [bool; 10] = [false; 10];
        for x in 0..unit.item_list.unit_items.len() {
            let item = unit.item_list.get_item(x as i32);
            if item.is_none() { continue;}
            let weapon = item.unwrap();
            if !weapon.item.is_weapon() { continue; }
            let kind = weapon.item.kind;
            if kind >= 10 || kind == 0 { continue; }
            let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
            if weapon.item.get_weapon_level() <= weapon_level && ( ( unit.selected_weapon_mask.value & ( 1 << kind ) != 0 || unit.selected_weapon_mask.value == 0 ) ) {
                has_weapon[kind as usize] = true;
            }
        }
        for i in 1..9 {
            if i ==  7 { continue; } // Rod
            if has_weapon[i as usize] { continue; }
            if i == 8 {
                if job.get_max_weapon_level(i) >= weapon_rank && ( unit.selected_weapon_mask.value & ( 1 << i ) != 0 || unit.selected_weapon_mask.value == 0 ) {
                    let index = 7*(weapon_rank - 1) + i - 2;
                    unit.item_list.add_item_no_duplicate(ItemData::get(STANDARD_WEPS[index as usize]).unwrap());
                }
            }
            else {
                if job.get_max_weapon_level(i) >= weapon_rank && ( unit.selected_weapon_mask.value & ( 1 << i ) != 0 || unit.selected_weapon_mask.value == 0 ) {
                    let index = 7*(weapon_rank - 1) + i - 1;
                    unit.item_list.add_item_no_duplicate(ItemData::get(STANDARD_WEPS[index as usize]).unwrap());
                }
            } 
        } 
        if job.get_max_weapon_level(4) >= 2 { unit.item_list.add_item_no_duplicate(ItemData::get("IID_長弓").unwrap());  } 
}

pub fn random_items_drops(unit: &Unit){
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_none() { continue;}  
        let u_item = &mut item.unwrap();
        if u_item.is_drop() && !u_item.is_equip() { 
            let new_item = get_random_item(u_item.item.iid, false);
            u_item.ctor_str(&new_item.get_string().unwrap());
            u_item.flags = 2;
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
        if kinds[i] == 5 {
            misercode_type = kinds[i];
            break;
        }
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

pub fn create_item_pool() {
    if RANDOM_ITEM_POOL.lock().unwrap().len() != 0 { return; }
    let item_list = ItemData::get_list().unwrap();
    for x in 0..item_list.len() {
        let random_item = &item_list[x];
        let iid = random_item.iid.get_string().unwrap();
        let item_flag = random_item.get_flag().value;
        if ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == iid).is_some() { continue; }
        if crate::utils::str_contains(random_item.name, "MIID_Ring") { continue; }
        if !has_name(random_item, true) { continue; }
        if random_item.is_unknown() { continue; }
        if item_flag & 16777216 != 0 { continue; } //Bless
        if item_flag & 33554432 != 0 { continue; } //Breath
        if item_flag & 67108864 != 0 { continue; }  //Dragon
        if item_flag & 134217728 != 0 { continue; } //Bullet
        if item_flag & 131072 != 0 { continue; } // Bento
        if item_flag & 32768 != 0 { continue; } // AI 
        let mut skip = false; 
        for y in 1..8 {
            if y == 2 { continue; }
            if item_flag & (1 << y ) != 0 {                
                skip = true;
                break;
            }
        }
        if !skip { RANDOM_ITEM_POOL.lock().unwrap().push(random_item.parent.index); }
    }
    println!("{} items are in the Random Item Pool", RANDOM_ITEM_POOL.lock().unwrap().len());
}
pub fn random_item(item_type: i32, allow_rare: bool) -> &'static Il2CppString {
    let item_list_size = RANDOM_ITEM_POOL.lock().unwrap().len();
    let rng = Random::get_system();
    loop {
        let item_index = RANDOM_ITEM_POOL.lock().unwrap()[rng.get_value( item_list_size as i32 ) as usize];
        let item = ItemData::try_index_get(item_index);
        if item.is_none() { continue; }
        let random_item = item.unwrap();
        if item_type == 0 { //Item Script Replacement
            if random_item.is_inventory() || random_item.is_material() { continue; }
        }
        else if item_type == 1 {    // Gift/Reward Items
            if random_item.usetype >= 32 && random_item.usetype <= 39 { continue; }
            if random_item.usetype == 0 && ( random_item.kind != 17 && random_item.kind != 18 ){ continue; }  
        }
        else if item_type == 2 {    // Exploration Drops
            let exploration = GameVariableManager::get_number("G_HubItem");
            let iid = random_item.iid.get_string().unwrap();
            if iid == "IID_スキルの書・離" || iid == "IID_スキルの書・破" {  continue; }    // No Adept/Expert Book
            let kind = random_item.kind;
            if kind == 17 && random_item.price > 5000 { continue; }     // Bond limited to 1000
            if kind == 18 && random_item.price >= 1000 { continue; }    // Limit Money to 1000g
            if kind == 13 || ( kind < 10 && kind != 0 ) { continue; }   // No Key Item or Weapon/Staff Related Items
            if ( kind < 17 && kind > 13 ) || (kind == 10 && random_item.usetype == 21) { continue; } // No Ores or Stat Boosters
            if exploration == 1  && random_item.usetype == 33 { continue; } 
            if exploration == 2 && random_item.usetype == 32 { continue; }
            if exploration == 3 && ( random_item.usetype == 33 || random_item.usetype == 32 ) { continue; }
            if random_item.usetype == 35 { continue; }
        }
        if random_item.get_flag().value & 1 != 0 && !allow_rare { continue; }
        return random_item.iid;
    }
}

// For item replacement
pub fn get_random_item(item: &'static Il2CppString, allow_rare: bool) -> &'static Il2CppString {
    let item_check = ItemData::get(&item.get_string().unwrap());
    // if Item is rare
    if item_check.is_some() {
        let flag = item_check.unwrap().get_flag().value;
        if flag & 1 == 1 { return item;  }
        let iid = item_check.unwrap().iid.get_string().unwrap();
        if ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == iid).is_some() { return item; }
    }
    else { return item; }
    let price = ItemData::get(&item.get_string().unwrap()).unwrap().price;
    let mut count = 0;
    loop {
        count += 1;
        let new_iid = random_item(0, allow_rare);
        let new_price = ItemData::get(&new_iid.get_string().unwrap()).unwrap().price;
        if new_price < price * CONFIG.lock().unwrap().replaced_item_price / 100 { continue; }
        if count >= 50 { return new_iid; }
        return new_iid;
    }
}

pub fn has_name(this: &ItemData, include_money: bool) -> bool {
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
                let item = ItemData::get(&iid.get_string().expect(format!("Bad Item ID in Reward Data\nItem {} in List {}", y, x).as_str()));
                if item.is_none() { continue; }
                let price = item.unwrap().price;
                let mut new_iid;
                let mut new_price;
                let mut count = 0;
                loop {
                    new_iid = random_item(1, rare_item);
                    new_price = ItemData::get(&new_iid.get_string().unwrap()).unwrap().price;
                    count += 1;
                    if new_price < 3*price || count >= 50 { break; }
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
            let price = ItemData::get(&iid.get_string().expect("Bad Item in Well Item List")).expect(format!("Invalid Item ID in Well Item {}", y).as_str()).price;
            let mut new_price; 
            let mut item_index;
            let mut new_iid; 
            let curent_reward = &well_items[y as usize];
            let mut count = 0;
            loop {
                new_iid = random_item(1, true);
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
                new_reward.ratio = curent_reward.ratio;
                new_reward.min = curent_reward.min;
                new_reward.max = curent_reward.max;
            }
            well_items.add(new_reward);
            in_set[item_index as usize] = true; 
        }
    }
    println!("Complete Randomization of Gift/Well Items");
    crate::shop::randomize_hub_random_items();
}
pub fn change_liberation_type() {
    let alear_random = !( GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap() == "PID_リュール" );
    let random_job = ( GameVariableManager::get_number("G_Random_Job") == 1 || GameVariableManager::get_number("G_Random_Job") == 3 ) ||  alear_random;
    if !random_job || !GameVariableManager::get_bool("G_Cleared_M002") { return; }
    if GameVariableManager::get_number("G_Liberation_Type") != 0 { return; }

    let hero_unit = unsafe { crate::person::unit_pool_get_hero(false, None) }.unwrap();
    let kinds = hero_unit.get_job().get_equippable_item_kinds();
    let mut liberation_type = 1; //Sword
    for i in 0..kinds.len() {
        if kinds[i] == 7 || kinds[i] >= 9 { continue; }
        if kinds[i] == 0 { continue; }
        liberation_type = kinds[i];
    }
    let liberation = ItemData::get_mut("IID_リベラシオン").unwrap();
    liberation.kind = liberation_type as u32;
    if liberation_type == 4 {
        liberation.range_o = 3;
        liberation.range_i = 2;
        liberation.set_cannon_effect("弓砲台".into());
        liberation.on_complete();
        liberation.get_equip_skills().add_skill(SkillData::get("SID_飛行特効").unwrap(),4, 0);
    }
    else if liberation_type == 5 || liberation_type == 6 {
        liberation.range_i = 1;
        liberation.range_o = 2;
        if liberation_type == 6 {
            liberation.set_cannon_effect("魔砲台炎".into());
            liberation.set_hit_effect( "エルファイアー".into());
            liberation.on_complete();
        }
        else { liberation.get_give_skills().add_sid("SID_毒",4, 0); }
    }
    else if liberation_type == 8 {
        liberation.get_equip_skills().add_sid("SID_気功",4, 0);
        liberation.get_equip_skills().add_sid("SID_２回行動",4,0);
    }
    else {
        liberation.range_i = 1;
        liberation.range_o = 1;
    }
    GameVariableManager::make_entry("G_Liberation_Type", liberation_type);
    GameVariableManager::set_number("G_Liberation_Type", liberation_type);
    println!("Liberation changed to weapon type {}", liberation_type);
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
        this.help_text = match CONFIG.lock().unwrap().random_job {
            1 => { "Playable units will be in random classes." },
            2 => { "Enemy/NPC units will be in random classes."},
            3 => { "All units will be in random classes."},
            _ => { "Units will be in their assigned class"},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match  CONFIG.lock().unwrap().random_job {
            1 => {  "Player" },
            2 => { "Enemy/NPC" },
            3 => { "All Units" },
            _ => { "None"},
        }.into();
    }
}
pub struct EnemyJobGauge;
impl ConfigBasicMenuItemGaugeMethods  for EnemyJobGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio =  if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_enemy_job_rate as f32 / 100.0 }
                            else { GameVariableManager::get_number("G_EnemyJobGauge") as f32 / 100.0 };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_enemy_job_rate as f32 / 100.0 }
                    else { GameVariableManager::get_number("G_EnemyJobGauge") as f32 / 100.0 };
        let result = ConfigBasicMenuItem::change_key_value_f(gauge, 0.0, 1.0, 0.25);
        if gauge != result {
            if GameUserData::get_sequence() == 0 {CONFIG.lock().unwrap().random_enemy_job_rate = ( result * 100.0 ) as i32; }
            else { GameVariableManager::set_number("G_EnemyJobGauge", ( result * 100.0 ) as i32 ); }
            this.gauge_ratio = result;
            //Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Percentage of enemy units will be in a random class.".into();
    }
}
pub struct ItemPriceGauge;
impl ConfigBasicMenuItemGaugeMethods  for ItemPriceGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio =  if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().replaced_item_price as f32 / 100.0 }
                            else { GameVariableManager::get_number("G_ItemGauge") as f32 / 100.0 };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().replaced_item_price as f32 / 100.0 }
                    else { GameVariableManager::get_number("G_ItemGauge") as f32 / 100.0 };
        let result = ConfigBasicMenuItem::change_key_value_f(gauge, 0.0, 1.0, 0.25);
        if gauge != result {
            if GameUserData::get_sequence() == 0 {CONFIG.lock().unwrap().replaced_item_price = ( result * 100.0 ) as i32; }
            else { GameVariableManager::set_number("G_ItemGauge", ( result * 100.0 ) as i32 ); }
            this.gauge_ratio = result;
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Minimum value of new item as a percentage of original item's value.".into();
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
        this.help_text = match CONFIG.lock().unwrap().random_item {
            1 => { "Items obtained from chests/villages will be random." },
            2 => { "Item drops from enemies will be random." },
            3 => { "Item obtained from events and enemy drops will be random." },
            _ => { "No changes made to item events or item drops." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_item {
            1 => { "Events"},
            2 => { "Drops"},
            3 => { "Events/Drops"}
            _ => { "None "}
        }.into();
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

