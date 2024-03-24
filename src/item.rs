use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    random::*,
    gamedata::{*, unit::*, item::*},
};
use super::CONFIG;
use crate::person;

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
    if new_weapon_type < 0 || new_weapon_type > 6 { return; }
    item.ctor_str(weapons[new_weapon_type as usize]);
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
            let iid1 =  unit_item.item.iid.get_string().unwrap();
            if iid1 == "IID_エンゲージ枠" || iid1 == "IID_不明" { continue; } 
            if iid1 == "IID_無し" { continue; }
            for y in x+1..item_list.get_count()+3 {
                let item2 = item_list.get_item(y);
                if item2.is_some() {
                    let unit_item2 = item2.unwrap(); 
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
        if job.is_low() && unit.level < 10 { unit.item_list.add_item_no_duplicate(ItemData::get("IID_ファイアー").unwrap());  }
        if unit.level >= 10 && job.is_low() { unit.item_list.add_item_no_duplicate(ItemData::get("IID_サンダー").unwrap()); }
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

pub struct RandomJobMod;
impl ConfigBasicMenuItemSwitchMethods for RandomJobMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_job);
        if CONFIG.lock().unwrap().random_job != result {
            CONFIG.lock().unwrap().random_job  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_job {  this.help_text = "Playable units will be in random classes.".into(); }
        else { this.help_text = "Units will be recruited in their default class.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_job  { this.command_text = "Random".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}

#[no_mangle]
extern "C" fn job_rnd() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomJobMod>("Random Classes") } 

 pub fn install_rnd_jobs() { cobapi::install_global_game_setting(job_rnd); }