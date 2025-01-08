use engage::menu::BasicMenuItemAttribute;

use super::*;
use super::item_rando::*;
use crate::{continuous::{get_story_chapters_completed, get_number_main_chapters_completed2}, randomizer::{assets::animation::MONSTERS, person::unit::{self, unit_update_auto_equip}}};

pub struct PlayerRandomWeapons;
impl ConfigBasicMenuItemSwitchMethods for PlayerRandomWeapons {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry_norewind("G_PRW", 0);
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let state = GameVariableManager::get_bool("G_PRW");
        let result = ConfigBasicMenuItem::change_key_value_b(state);
        if state != result {
            GameVariableManager::set_bool("G_PRW", result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if GameVariableManager::get_bool("G_PRW") { "Player units can be recruited with non-standard weapons."}
            else { "Player units are recruited with standard weapons." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if GameVariableManager::get_bool("G_PRW") { "Random"} else {"Standard"}.into();
    }
}
fn prw_build_attrs(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_number("G_Random_Item") != 0 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_prw() ->  &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<PlayerRandomWeapons>("Player Starting Weapon Settings");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = prw_build_attrs as _);
    switch
}

pub const STANDARD_WEPS: [&str; 35] = [ // S L A B K T F
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
    if e_skills.is_some() { return e_skills.unwrap().find_sid("SID_轟雷発動可能".into()).is_some();}
    return false;
}
pub fn get_replacement_type(item: &UnitItem) -> i32 {
    if let Some(e_skills) = item.get_equipped_skills() {
        if e_skills.find_sid("SID_スマッシュ".into()).is_some() { return 6; }  // Smash
        if e_skills.find_sid("SID_轟雷発動可能".into()).is_some() { return 7; } // Thunder
    }
    let iid = item.item.iid.to_string();
    match iid.as_str() {
        "IID_いかづちの剣"|"IID_ほのおの槍"|"IID_かぜの大斧"| "IID_光の弓" => { 2 },    //magic
        "IID_ほそみの剣"|"IID_ほそみの槍"| "IID_ショートアクス" => { 1 },   //silm
        "IID_キルソード"|"IID_キラーランス"|"IID_キラーアクス"|"IID_キラーボウ"|"IID_エルサージ" => { 3 }, //crit
        "IID_手槍"|"IID_スレンドスピア"|"IID_手斧"|"IID_トマホーク"|"IID_長弓" => { 4 }, //Range
        "IID_アーマーキラー"|"IID_ナイトキラー"|"IID_ポールアクス"|"IID_ドラゴンキラー" => { 5 }, //Effective
        _ => { 0 }, // Standard
    }
}
pub fn is_slim(item: &UnitItem) -> bool {
    let iid = item.item.iid.to_string();
    if iid == "IID_ほそみの剣" { return true; }
    if iid == "IID_ほそみの槍" { return true; }
    if iid == "IID_ショートアクス" { return true; }
    return false; 
}
pub fn is_magic_weapon(item: &UnitItem) -> bool {
    let iid = item.item.iid.to_string();
    if iid == "IID_いかづちの剣" { return true; }
    if iid == "IID_ほのおの槍" { return true; }
    if iid == "IID_かぜの大斧" { return true; }
    if iid == "IID_光の弓" { return true;}
    return false; 
}
pub fn is_crit_weapon(item: &UnitItem) -> bool {
    let iid = item.item.iid.to_string();
    if iid == "IID_キルソード" { return true; }
    if iid == "IID_キラーランス" { return true; }
    if iid == "IID_キラーアクス" { return true; }
    if iid == "IID_キラーボウ" { return true;}
    if iid == "IID_エルサージ" { return true;}
    return false; 
}
pub fn range_weapon(item: &UnitItem) -> bool {
    let iid = item.item.iid.to_string();
    if iid == "IID_手槍" { return true; }
    if iid == "IID_スレンドスピア" { return true; }
    if iid == "IID_手斧" { return true; }
    if iid == "IID_トマホーク" { return true;}
    if iid == "IID_長弓" { return true;}
    return false;
}
pub fn is_effective_weapon(item: &UnitItem) -> bool {
    let iid = item.item.iid.to_string();
    if iid == "IID_アーマーキラー" { return true; }
    if iid == "IID_ナイトキラー" { return true; }
    if iid == "IID_ポールアクス" { return true; }
    return false;
}

pub fn replace_weapon(item: &UnitItem, weapon_mask: i32, max_rank: i32, is_enemy: bool) {
    if item.item.get_flag().value & 128 != 0 { return; }
    if item.item.get_flag().value & 2 != 0 { return; }
    let mut level = item.item.get_weapon_level();
    if max_rank < level { level = max_rank; }
    let mut new_weapon_type: i32  = -1;
    let flag = item.flags;
    for x in 1..9 {
        if weapon_mask & ( 1 << x ) != 0 {
            new_weapon_type = x-1;
            break;
        }
    }
    if new_weapon_type < 0 { 
        let rng = Random::get_game().get_value(3);
        new_weapon_type = rng;
    }
    // Random Weapons for Enemy
    let ran_map = GameVariableManager::get_number("G_Continuous") == 3;
    if is_enemy && ( GameVariableManager::get_bool("G_Cleared_M011") || ran_map ) {
       // println!("Enemy Item Replacement for rank: {}, Weapon: {}", max_rank, new_weapon_type);
        if get_number_main_chapters_completed2() < 13 && ran_map {
            if let Some(generic_weapon) = item_rando::WEAPONDATA.lock().unwrap().get_generic_weapon(new_weapon_type, level) {
               // println!("Replacement Generic Item: {}", Mess::get(generic_weapon.name));
                item.ctor(generic_weapon);
                item.set_flags(flag);
                return;
            } 
        }
        if let Some(new_item) = item_rando::WEAPONDATA.lock().unwrap().get_new_weapon(item, new_weapon_type, true) {
            // println!("Replacement Item: {}", Mess::get(new_item.name));
            item.ctor(new_item);
            if new_item.flag.value & 2 == 0 { item.set_flags(flag); }
            return;
        }
        if let Some(generic_weapon)= item_rando::WEAPONDATA.lock().unwrap().get_generic_weapon(new_weapon_type, level) {
           // println!("Replacement Generic Item: {}", Mess::get(generic_weapon.name));
            item.ctor(generic_weapon);
            item.set_flags(flag);
            return;
        } 
    }
    if GameVariableManager::get_bool("G_PRW") {
        if let Some(new_item) = item_rando::WEAPONDATA.lock().unwrap().get_new_weapon(item, new_weapon_type, false) {
            item.ctor(new_item);
            return;
        }
    }
    // println!("Normal Item Replacement");
    if new_weapon_type == 7 { new_weapon_type = 6; }
    if new_weapon_type < 0 || new_weapon_type > 6 { return; }
    if weapon_mask & 512 != 0 { return;  }
    let mut weapons: [&str; 7];
    // Standard Set
    weapons = match level { 
        0|1 => { ["IID_鉄の剣", "IID_鉄の槍", "IID_鉄の斧", "IID_鉄の弓", "IID_鉄のナイフ", "IID_ファイアー", "IID_鉄身の法"] },
        2 => { ["IID_鋼の剣", "IID_鋼の槍", "IID_鋼の斧", "IID_鋼の弓", "IID_鋼のナイフ", "IID_エルファイアー", "IID_鋼身の法"] },
        3 => { ["IID_銀の剣", "IID_銀の槍", "IID_銀の斧", "IID_銀の弓", "IID_銀のナイフ", "IID_エルファイアー", "IID_銀身の法"] },
        4 => { ["IID_勇者の剣", "IID_勇者の槍", "IID_勇者の斧", "IID_勇者の弓", "IID_ペシュカド", "IID_ボルガノン", "IID_閃進の法"] },
        _ => { ["IID_クラドホルグ", "IID_ブリューナク", "IID_フラガラッハ", "IID_レンダウィル", "IID_シンクエディア", "IID_ノヴァ", "IID_覇神の法"] },
    };
    match get_replacement_type(item) {
        1 => { weapons = ["IID_ほそみの剣","IID_ほそみの槍", "IID_ショートアクス", "IID_ショートボウ", "IID_ショートナイフ" , "IID_サージ", "IID_初心の法"];  }, // slims
        2 => {  weapons[0] = "IID_いかづちの剣"; 
                weapons[1] = "IID_ほのおの槍"; 
                weapons[2] = "IID_かぜの大斧"; 
                weapons[3] = "IID_光の弓";
            },  //magic 
        3 => {
            weapons[0] = "IID_キルソード"; 
            weapons[1] = "IID_キラーランス"; 
            weapons[2] = "IID_キラーアクス"; 
            weapons[3] = "IID_キラーボウ"; 
            weapons[5] = "IID_エルサージ";
        },  // Crit
        4 => {
            match level {
                1 => { weapons[1] = "IID_手槍"; weapons[2] = "IID_手斧"; },
                2 => { weapons[3] = "IID_長弓"; weapons[1] = "IID_手槍"; weapons[2] = "IID_手斧"; },
                _ => { weapons[3] = "IID_長弓"; weapons[2] = "IID_トマホーク"; weapons[1] = "IID_スレンドスピア"; },
            };
        },
        5 => {  //
            if level == 2 { weapons[0] = "IID_アーマーキラー"; }
            else if level > 2 { weapons[0] = "IID_ドラゴンキラー"; }
            weapons[1] = "IID_ナイトキラー"; 
            weapons[2] = "IID_ポールアクス";
        },
        6 => {  // Smash
            match level {
                2 => { weapons[0] = "IID_鉄の大剣"; weapons[1] = "IID_鉄の大槍"; weapons[2] = "IID_鉄の大斧"; },
                3 => { weapons[0] = "IID_鋼の大剣";  weapons[1] = "IID_鋼の大槍"; weapons[2] = "IID_鋼の大斧"; },
                4 => { weapons[0] = "IID_銀の大剣";  weapons[1] = "IID_銀の大槍";  weapons[2] = "IID_銀の大斧"; },
                _ => {},
            }
        },
        7 => {  //Thunder
            weapons = match level {
                1 =>  { ["IID_鉄の剣", "IID_手槍", "IID_ショートアクス", "IID_ショートボウ", "IID_ショートナイフ", "IID_サンダー", "IID_初心の法"] },
                2 => { ["IID_いかづちの剣", "IID_ほのおの槍", "IID_手斧", "IID_長弓", "IID_カルド", "IID_エルサンダー", "IID_護身の法"] },
                3 => { ["IID_ドラゴンキラー", "IID_スレンドスピア", "IID_トマホーク", "IID_光の弓", "IID_スティレット", "IID_エルサンダー", "IID_護身の法"] },
                _ => { ["IID_ドラゴンキラー", "IID_スレンドスピア", "IID_トマホーク", "IID_光の弓", "IID_ペシュカド", "IID_トロン", "IID_閃進の法"] },
            };
        }
        _ => {},
    }
    //println!("Replace item {} , weapon mask {}, max level {}", item.item.name.to_string(), weapon_mask, max_rank);
    item.ctor_str(weapons[new_weapon_type as usize]);
    item.set_flags(flag);
}

//Has Healing staff
pub fn replace_staves(item_list: &UnitItemList){
    for x in 0..8 {
        let item = item_list.get_item(x);
        if item.is_some() {
            let staff = &item.unwrap();
            let kind = staff.item.get_kind(); 
            if kind == 7 { 
                let staff_name = staff.item.iid.to_string();
                if staff_name == "IID_ライブ" || staff_name == "IID_リブロー" || staff_name == "IID_リライブ" { 
                    staff.ctor_str("IID_傷薬");
                }
                else { staff.ctor_str("IID_特効薬"); }
            }
        }
    }
}
pub fn dispose_item_type(item_list: &UnitItemList, item_kind: i32){
    for x in 0..8 {
        if let Some(item) = item_list.get_item(x){
            let kind = item.item.get_kind(); 
            if kind ==  item_kind { item.dispose(); }
        }
    }
}

pub fn dispose_all_but_drops(item_list: &UnitItemList){
    for x in 0..8 {
        if !item_list.unit_items[x].is_drop() {
            if let Some(item) = item_list.get_item(x as i32 ) {
                if  !item.is_drop() { item.dispose(); }
            }
        }
    }
}
pub fn dispose_unusables(unit: &Unit) {
    for x in 0..8 {
        if unit.item_list.unit_items[x].item.parent.index < 3 { continue; }
        if let Some(item) = unit.item_list.get_item(x as i32 ) {
            if item.item.flag.value & 128 == 0 && item.flags & 2 == 0 && item.item.kind > 0 { // Not Engage Weapon or Not Drop
                if unsafe { !unit_can_equip_item(unit, x as i32 , true, true, None) } {
                    item.dispose();
                }
            }
        }
    }
    for x in 0..8 {
        let item = unit.item_list.unit_items[x];
        if item.item.parent.index < 3 { continue; }
        if item.is_empty() {
            for y in x+1..unit.item_list.unit_items.len() {
                if !unit.item_list.unit_items[y].is_empty() {
                    unit.item_list.move_item(y as i32, x as i32);
                    break;
                }
            }
        }
    }
}

pub fn remove_duplicates(item_list: &UnitItemList) {
    for x in 0..8 {
        if let Some(unit_item) = item_list.get_item(x as i32) { 
            if unit_item.is_drop() {  if unit_item.item.flag.value & 131 != 0 { unit_item.flags = 0; } }
            if unit_item.item.parent.index < 3 { continue; }
            let iid1 =  unit_item.item.iid.to_string();
            for y in x+1..8 {
                let item2 = item_list.get_item(y as i32 );
                if item2.is_some() {
                    let unit_item2 = item2.unwrap(); 
                    if unit_item2.is_drop() { continue; }
                    let iid2 =  unit_item2.item.iid.to_string();
                    if iid2 == iid1 { 
                        unit_item2.dispose(); 
                       //  println!("Dispose of {}", unit_item2.item.name.to_string());
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
        let iid1 =  item.item.iid.to_string();
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

pub fn adjust_melee_weapons(unit: &Unit) {
    if unit.person.get_asset_force() == 0 { return; }
    let job = unit.get_job();
    if unit.get_capability(1, true) < unit.get_capability(6, true) {
        let mag_weapons = ["IID_いかづちの剣", "IID_ほのおの槍", "IID_かぜの大斧", "IID_光の弓"];
        for x in 0..unit.item_list.unit_items.len() {
            let item = unit.item_list.get_item(x as i32);
            if item.is_none() { continue;}
            let weapon = item.unwrap();
            let kind = weapon.item.kind;
            if !can_change_weapon(weapon, true) { continue; }
            let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
            if weapon.item.get_weapon_level() <= weapon_level && ( ( unit.selected_weapon_mask.value & ( 1 << kind ) != 0 || unit.selected_weapon_mask.value == 0 ) ) {
               weapon.ctor_str(mag_weapons[kind as usize -1 ]);
            }
        }
        return;
    }
}
pub fn get_number_of_usable_weapons(unit: &Unit) -> i32 {
    let mut count = 0;
    for x in 0..8 {
        if unsafe { unit_can_equip_item(unit, x as i32 , true, true, None) }  { count += 1;}
    }
    count
}

pub fn can_change_weapon(weapon: &UnitItem, melee: bool) -> bool {
    if !weapon.item.is_weapon() { return false; }
    let flag = weapon.item.get_flag().value; 
    if flag & 128 != 0 || flag & 2 != 0 { return false; }
    let kind = weapon.item.kind;
    if melee { return kind > 0 && kind < 5;  }
    else { return kind > 0 && kind < 9; }
}

pub fn adjust_staffs(unit: &Unit) {
    let job = unit.get_job();
    let weapon_mask = job.get_weapon_mask();
    let jid = job.jid.to_string();
    remove_duplicates(unit.item_list);
    let is_vander = GameVariableManager::get_string("G_R_PID_ヴァンドレ").to_string() == unit.person.pid.to_string();
    let is_veyle = unit.person.pid.to_string() == "PID_ヴェイル";
    let rng = Random::get_system();
    let story_chapter = get_story_chapters_completed();
    let continous = GameVariableManager::get_number("G_Continuous") == 3;
    let is_player = unit.person.get_asset_force() == 0;
    if weapon_mask.value & 64 != 0 && (!is_vander && !is_veyle) {
        dispose_item_type(unit.item_list, 6);
        if GameVariableManager::get_bool("G_PRW") && is_player {
            loop {
                if let Some(new_item) = item_rando::WEAPONDATA.lock().unwrap().get_random_weapon(5) {
                    if new_item.get_weapon_level() <= job.weapon_levels[6]  {
                        unit.item_list.add_item_no_duplicate(new_item);
                        break;
                    }
                }
            }
        }
        else {
            let value = rng.get_value(10);
            let job_level = job.get_max_weapon_level(6);
            if job.is_low() && unit.level <= 20  { 
                if GameVariableManager::get_bool("G_Cleared_M009") || (continous && story_chapter >= 8 ){ 
                    if value < 6 { unit.item_list.add_iid_no_duplicate("IID_エルファイアー"); }
                    else { unit.item_list.add_iid_no_duplicate("IID_エルウィンド"); }
                }
                else { 
                    if value % 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_ファイアー");  }
                    else {unit.item_list.add_iid_no_duplicate("IID_ティラミス魔道書");  }
                }
                if GameVariableManager::get_bool("G_Cleared_M007") || (continous && story_chapter >= 6 ){
                    if value & 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_サージ");  }
                    else if GameVariableManager::get_bool("G_Cleared_M010") || (continous && story_chapter >= 10) { unit.item_list.add_iid_no_duplicate("IID_エルサージ"); }
                    else { unit.item_list.add_iid_no_duplicate("IID_サージ");  }
                }
            }
            else {
                if ( unit.level < 10 && job.is_high() ) || ( job.is_low() && unit.level < 30 ) || job_level < 4 {
                    if value < 6 { unit.item_list.add_iid_no_duplicate("IID_エルファイアー"); } //Elfire
                    else { unit.item_list.add_iid_no_duplicate("IID_エルウィンド"); }   //Elwind
                    if value % 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_エルサンダー"); }  //Elthunder 50%
                }
                else {
                    if job_level > 4 { // S-Rank
                        if value % 4 == 0 { unit.item_list.add_iid_no_duplicate("IID_ノヴァ");  }  // Nova
                        else if value % 4 == 1 { unit.item_list.add_iid_no_duplicate("IID_ボルガノン");   } // Bol
                        else if value % 4 == 2 { unit.item_list.add_iid_no_duplicate("IID_エクスカリバー");  } //Excal
                        else { unit.item_list.add_iid_no_duplicate("IID_トロン"); } // Thoron
                        if ( unit.person.get_asset_force() != 0 && value == 1) && ( GameVariableManager::get_bool("G_Cleared_M021") || GameVariableManager::get_bool("G_Cleared_S009")) {
                            unit.item_list.add_iid_no_duplicate("IID_メティオ_G004");  //Meteor
                        }
                        else { unit.item_list.add_iid_no_duplicate("IID_トロン"); }
                    }
                    else {
                        if value % 3 == 0 { unit.item_list.add_iid_no_duplicate("IID_ボルガノン");   } // Bol
                        if value % 3 == 1 { unit.item_list.add_iid_no_duplicate("IID_エクスカリバー");  } //Excal
                        if value % 3 == 2 { unit.item_list.add_iid_no_duplicate("IID_エルサージ");  } //Excal
                        if value < 5 { unit.item_list.add_iid_no_duplicate("IID_トロン"); } // Thoron
                    }
                }
            }
        }
    }
    let staff_level = if  unit::has_sid(unit, "SID_杖使い＋＋") { 4 } 
        else if unit::has_sid(unit, "SID_杖使い＋") { 3 }
        else if unit::has_sid(unit, "SID_杖使い") { 2 } 
        else { job.get_max_weapon_level(7) };

    if weapon_mask.value & ( 1 << 7 ) == 0 { replace_staves(unit.item_list); }
    else if weapon_mask.value & ( 1 << 7 ) != 0 && staff_level > 0 {
        dispose_item_type(unit.item_list, 7);
        if is_player { //Player Staff users
            if job.is_low() { //Fracture for Wing Tamer Hortensia
                if jid == "JID_スレイプニル下級" { unit.item_list.add_iid_no_duplicate("IID_コラプス"); }
                if unit.level < 10 { unit.item_list.add_iid_no_duplicate("IID_ライブ"); }
                else if unit.level < 15 {  unit.item_list.add_iid_no_duplicate("IID_リライブ") }
                else {  unit.item_list.add_iid_no_duplicate("IID_リブロー");  }
            }
            else {
                if jid == "JID_スレイプニル" { unit.item_list.add_iid_no_duplicate("IID_コラプス");  } 
                if jid == "JID_ハイプリースト" {    // Warp/Fortify for High Priest
                    let rng = Random::get_game();
                    let value = rng.get_value(10);
                    if value % 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_ワープ");  }
                    else {unit.item_list.add_iid_no_duplicate("IID_レスキュー");  }
                    unit.item_list.add_iid_no_duplicate("IID_リザーブ");
                }
                else { unit.item_list.add_iid_no_duplicate("IID_リブロー"); }  // physic for the rest of staffers instead
            }
        }
        else {
            for x in 1..4 {
                if x == 3 && rng.get_value(3) > 1 {  continue; }
                if let Some(staff_) = WEAPONDATA.lock().unwrap().get_staff(x, staff_level){
                    unit.item_list.add_item_no_duplicate(staff_);
                }
            }
        }
    }
    let pid = unit.person.pid.to_string();
    add_equip_condition(unit);
    dispose_unusables(unit);
    remove_duplicates(unit.item_list);
    let enemy = unit.person.get_asset_force() != 0;
    if jid == "JID_裏邪竜ノ娘" || jid == "JID_裏邪竜ノ子" { 
        unit.private_skill.add_sid("SID_オヴスキュリテ装備可能", 10, 0);    //Equip Obscurite
        if pid != "PID_エル" && pid != "PID_ラファール" {
            if let Some(stone1) =  WEAPONDATA.lock().unwrap().get_dragon_stone(enemy) {
                unit.item_list.add_item_no_duplicate(stone1);
                loop {
                    if let Some(stone2) = WEAPONDATA.lock().unwrap().get_dragon_stone(enemy) {
                        if stone2.parent.index != stone1.parent.index {
                            unit.item_list.add_item_no_duplicate(stone2);
                            break;
                        }
                    }
                }
            }
        }
    }
    if jid == "JID_神竜ノ子" || jid == "JID_神竜ノ王" {
        unit.private_skill.add_sid("SID_リベラシオン装備可能", 10, 0);
        unit.private_skill.add_sid("SID_ヴィレグランツ装備可能", 10, 0);
    }
    let completed_m017 = ( !continous && GameVariableManager::get_bool("G_Cleared_M017") ) || (continous && story_chapter >= 16);
    fix_weapons_by_rank(unit, completed_m017);
    if is_veyle {
        if is_player && !GameVariableManager::get_bool("G_PRW") {
            unit.item_list.add_iid_no_duplicate("IID_オヴスキュリテ"); 
        }
        unit.item_list.add_iid_no_duplicate("IID_ミセリコルデ");
        magic_dagger_weapon_change(unit.get_job());
    }
    if pid == "PID_エル" {
        unit.item_list.add_iid_no_duplicate("IID_邪竜石"); 
        unit.item_list.add_iid_no_duplicate("IID_邪竜石_騎馬特効"); 
    }
    if pid == "PID_ラファール" {    
        unit.item_list.add_iid_no_duplicate("IID_邪竜石"); 
        unit.item_list.add_iid_no_duplicate("IID_邪竜石_騎馬特効");
    }
    if jid == "JID_マージカノン" {  //Mage Canon
        let len = WEAPONDATA.lock().unwrap().bullet_list.len();
        if len > 1 {
            let index1 = WEAPONDATA.lock().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
            unit.item_list.add_item_no_duplicate(ItemData::try_index_get(index1).unwrap());
            loop {
                let index2 = WEAPONDATA.lock().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
                if index2 != index1 {
                    unit.item_list.add_item_no_duplicate(ItemData::try_index_get(index2).unwrap());
                    break;
                }
            }
        }
    }
    if jid == "JID_邪竜ノ娘" && !is_veyle {
        if !GameVariableManager::get_bool("G_PRW") && is_player {
            dispose_all_but_drops(unit.item_list);
            unit.item_list.add_iid_no_duplicate("IID_オヴスキュリテ"); 
        }
        unit.private_skill.add_sid("SID_オヴスキュリテ装備可能", 10, 0); //Equip Obscurite
        unit.private_skill.add_sid("SID_ミセリコルデ装備可能", 10, 0);  //Equip Misercode
    }
    if is_player { // Vul or Elixir
        if unit.get_capability(0, false) >= 45 { unit.item_list.add_iid_no_duplicate("IID_特効薬");   }
        else { unit.item_list.add_item_no_duplicate(ItemData::get("IID_傷薬").unwrap());  }
    }
    add_generic_weapons(unit);
    adjust_melee_weapons(unit);
    remove_duplicates(unit.item_list);
    add_equip_condition(unit);
    adjust_missing_weapons(unit);
}

pub fn adjust_missing_weapons(unit: &Unit) {
    if get_number_of_usable_weapons(unit) < 1 {
        dispose_unusables(unit);
        if unit.job.jid.to_string() == "JID_マージカノン" {  //Mage Canon
            let len = WEAPONDATA.lock().unwrap().bullet_list.len();
            let rng = Random::get_game();
            if len > 1 {
                let index1 = WEAPONDATA.lock().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
                unit.item_list.add_item_no_duplicate(ItemData::try_index_get(index1).unwrap());
                loop {
                    let index2 = WEAPONDATA.lock().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
                    if index2 != index1 {
                        unit.item_list.add_item_no_duplicate(ItemData::try_index_get(index2).unwrap());
                        break;
                    }
                }
            }
            return;
        }
       // println!("Has No Weapons: {} {}", Mess::get_name(unit.person.pid), Mess::get_name(unit.get_job().jid));
        add_generic_weapons(unit);
    }
}


pub fn fix_weapons_by_rank(unit: &Unit, upgrade_weapon: bool) {
   // println!("Fixing weapons by rank for {}", Mess::get(unit.person.get_name().unwrap()).to_string());
    let unit_level = unit.level as i32 + unit.internal_level as i32;
    let weapon_rank = if !upgrade_weapon { 0 }
    else if unit_level < 25 { 2 }
    else { 3 };

    let job = unit.get_job();
    let pid = unit.person.pid.to_string();
    for x in 0..unit.item_list.unit_items.len() {
        let weapon = unit.item_list.get_item(x as i32).unwrap();
        if !can_change_weapon(weapon, false) { continue; }
        let iid = weapon.item.iid.to_string();
        if iid == "IID_エンゲージ枠" || iid == "IID_不明" { continue; } // if engage weapon slot or none skip
        if iid == "IID_ミセリコルデ" && pid != "PID_ヴェイル" {
            weapon.dispose();
            continue;
        }
        let kind = weapon.item.kind;

        let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
       //  println!("Item {}: {} Kind: {}, Rank: {}", x, weapon.item.name.to_string(), kind, weapon_level);
        if weapon_level == 0 && !weapon.is_drop(){ weapon.dispose(); continue;  }
        if weapon_level < weapon.item.get_weapon_level() && weapon_level > 0 { // Weapon is higher than the Job's rank
            let weapon_index;
            if kind > 8 { continue; }
            if kind == 8 { weapon_index = 6;}
            else { weapon_index = (kind - 1 ) as i32;}
            let index = ( (7* (weapon_level - 1) ) + weapon_index ) as usize;
            if index < 35 { weapon.ctor_str(  STANDARD_WEPS[ index as usize ] ); }
        }
        if upgrade_weapon {
            if weapon_rank > 0 && ( weapon.item.get_weapon_level() < weapon_rank && weapon_level >= weapon_rank ) {    // Weapon is lower than level rank and Job can use it
                let weapon_index;
                if kind == 8 { weapon_index = 6;}
                else { weapon_index = (kind - 1 ) as i32;}
                let index = ( (7* (weapon_rank - 1) ) + weapon_index ) as usize;
                if index < 35 {  weapon.ctor_str(  STANDARD_WEPS[ index ] );}
            }
        }
    }
}
pub fn add_generic_weapons(unit: &Unit) {
    //if unit.person.get_asset_force() != 0 { // Making sure enemies have weapon
    let job = unit.get_job();
    let job_mask = job.get_weapon_mask().value;
   // println!("Adding Weapons for {} ({}), with Selected Mask {} / {}", Mess::get(unit.person.get_name().unwrap()).to_string(), Mess::get(unit.get_job().name), unit.selected_weapon_mask.value, job_mask);
    let jid = job.jid.to_string();
    if jid == "JID_ボウナイト" { unit.item_list.add_item_no_duplicate(ItemData::get("IID_銀の弓").unwrap());  }
    if jid == "JID_エンチャント" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_HPの薬").unwrap());  
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_力の薬").unwrap());  
    }
    let combine_mask = unit.selected_weapon_mask.value | job_mask;
    let unit_level = unit.level as i32 + unit.internal_level as i32;
    let player = unit.person.get_asset_force() == 0;
    let weapon_rank = if get_story_chapters_completed() < 7 { 1 }
        else if unit_level < 21 { 2 }
        else { 3 };
    let mut has_weapon: [bool; 10] = [false; 10];
    has_weapon[7] = true;
    for x in 0..unit.item_list.unit_items.len() {
        let item = unit.item_list.get_item(x as i32);
        if item.is_none() { continue;}
        let weapon = item.unwrap();
        if !weapon.item.is_weapon() { continue; }
        let kind = weapon.item.kind;
        if kind > 9 || kind == 0 { continue; }
        let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
        if weapon.item.get_weapon_level() <= weapon_level && combine_mask & ( 1 << kind ) != 0 { has_weapon[kind as usize] = true; }
        else if weapon.flags & 2 != 0 { weapon.dispose();  }
    }
    for i in 1..9 {
        if has_weapon[i as usize] { continue; }
        let rank = if job.get_max_weapon_level(i) < weapon_rank { job.get_max_weapon_level(i) } else { weapon_rank };
        if combine_mask & (1 << i ) != 0 && rank > 0 {
            if player  && GameVariableManager::get_bool("G_PRW") {
                loop {
                    if let Some(item) = WEAPONDATA.lock().unwrap().get_random_weapon(i-1) {
                        if item.get_weapon_level() <= rank {
                            unit.item_list.add_item_no_duplicate(item); 
                            break;
                        }
                    }
                }
            }
            if player || !GameVariableManager::get_bool("G_Cleared_M011") {
                if let Some(item) = WEAPONDATA.lock().unwrap().get_generic_weapon(i-1, rank) { unit.item_list.add_item_no_duplicate(item); }
            }
            else {
                loop {
                    if let Some(item) = WEAPONDATA.lock().unwrap().get_random_weapon(i-1) {
                        if item.get_weapon_level() <= rank {
                            unit.item_list.add_item_no_duplicate(item); 
                            break;
                        }
                    }
                }
            }
        }
    }
    if job.get_max_weapon_level(4) >= 2 && GameVariableManager::get_bool("G_Cleared_M022") { unit.item_list.add_item_no_duplicate(ItemData::get("IID_長弓").unwrap());  } 
 //   println!("Has {} weapons", get_number_of_usable_weapons(unit));
}

pub fn random_items_drops(unit: &Unit){
    let rng = Random::get_system();
    let mut none_count = 0;
    let mut rate =
        if GameUserData::get_difficulty(false) == 2 {1 / 2} else { 1 } * GameVariableManager::get_number("G_ItemDropGauge");
        
    for x in 0..8 {
        if let Some(u_item) = unit.item_list.get_item(x) {
            if u_item.item.flag.value & 131 != 0 {  u_item.flags = 0; }
        }
    }
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_none() { continue; }
        let u_item = &mut item.unwrap();
        if u_item.is_drop(){
            if u_item.item.flag.value & 131 != 0  { //Unequip 
                u_item.flags = 0;  
                continue;
            }
            else if !u_item.is_equip() { 
                let new_item = get_random_item(u_item.item.iid, false);
                u_item.ctor_str(&new_item.to_string());
                u_item.flags = 2;
                none_count += 1;
            }
        } 
        let iid1 =  u_item.item.iid.to_string();
        if iid1 == "IID_無し" && none_count == 0 { 
            if rng.get_value(100) < rate {
                u_item.ctor_str(&random_item(4, false).to_string());
                u_item.flags = 2;
                rate = rate / 2;
            }
            else { none_count += 1; }
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
    let mut misercode_type = 5; //Dagger
    veyle_job.get_equippable_item_kinds().iter().for_each(|&k| if ( k > 0 && k < 6 ) || k == 8 { misercode_type = k });
    GameVariableManager::make_entry("G_Misercode_Type", misercode_type);
    GameVariableManager::set_number("G_Misercode_Type", misercode_type);
    super::change_misercode_type();
}


// Meteor Adjustment
pub fn adjust_items() {
    let meteor = ItemData::get_mut("IID_メティオ").unwrap();
    meteor.endurance = 1;
    let index = meteor.parent.index;
    let item_list = ItemData::get_list_mut().unwrap();
    let can_trade = true; //CONFIG.lock().unwrap().enable_tradables_item;
    for x in 0..item_list.len() {
        let random_item = &mut item_list[x];
        let flag = random_item.flag.value;
        if can_trade && ( flag & 131 == 3 || flag & 131 == 2 ) { random_item.get_flag().value = flag - 2; }
        if random_item.get_flag().value & 16 != 0  {
            let iid = random_item.iid.to_string();
            if iid != "IID_メティオ_G004" {
                random_item.get_flag().value = 0; 
                random_item.price = 5000;
            }
        }
    }
    // Unit Pool 
    for x in 1..250 {
        let o_unit: Option<&mut Unit> = unsafe { unit_pool_get(x, None) };
        if o_unit.is_none() { continue; }
        let unit = o_unit.unwrap();
        let force = if unit.force.is_none() { -1 } else {
            unit.force.unwrap().force_type
        };
        if force == 1 { adjust_missing_weapons(unit); }
        let uses = if unit.force.is_none() { 255 }
            else if unit.force.unwrap().force_type == 1 { 255 }
            else { 1 };

        for y in 0..8 {
            if unit.item_list.unit_items[y].item.parent.index < 3 { continue; }
            if unit.item_list.unit_items[y].item.parent.index == index {
                unit.item_list.unit_items[y].set_endurance(uses);
            }
            if ( force == 0 || force == 3 ) && unit.item_list.unit_items[y].item.flag.value & 2 != 0 {  //Cant Trade
                unit.item_list.unit_items[y].dispose();
            }
        }
        for x in 0..8 {
            let item = unit.item_list.unit_items[x];
            if item.item.parent.index < 3 { continue; } 
            if item.is_empty() {
                for y in x+1..unit.item_list.unit_items.len() {
                    if !unit.item_list.unit_items[y].is_empty() {
                        unit.item_list.move_item(y as i32, x as i32);
                        break;
                    }
                }
            }
        }
        unsafe { unit_update_auto_equip(unit, None);}
    }
    let convoy_index = unsafe { get_empty_index(None) };
    for x in 0..convoy_index {
        let item = unsafe {  get_item_from_index(x, None ) };
        if item.unit_item.item.parent.index == index {
            item.unit_item.endurance = 1;
        }
        if item.unit_item.item.flag.value & 130 != 0 {  //Engage + //Cant Trade
            item.unit_item.dispose();
        }
    }
}

pub fn adjust_enemy_meteor(unit: &mut Unit) {
    if unit.person.get_asset_force() == 0 {  return;  }
    let index = ItemData::get("IID_メティオ").unwrap().parent.index;
    for y in 0..8 {
        if unit.item_list.unit_items[y].item.parent.index == index {
            unit.item_list.unit_items[y].ctor_str("IID_メティオ_G004");
        }
    }
}

pub fn add_equip_condition(unit: &Unit) {
    for x in 0..8 {
        if unit.item_list.unit_items[x].item.equip_condition.is_some() {
            let sid = unit.item_list.unit_items[x].item.equip_condition.unwrap().to_string();
            unit.private_skill.add_sid(&sid, 10, 0);
        }
    }
}

pub fn add_monster_weapons(unit: &Unit){
    // ["JID_幻影飛竜", "JID_異形飛竜", "JID_幻影狼", "JID_異形狼",  "JID_E006ラスボス", "JID_幻影竜", "JID_異形竜", "JID_邪竜"];
    let jid = unit.get_job().jid.to_string();
    unit.item_list.put_off_all_item();
    if let Some(pos) = MONSTERS.iter().position(|&x| jid == x) {
        match pos {
            0 => { // Phantom Wyvern
                unit.item_list.add_iid_no_duplicate("IID_氷のブレス");
                unit.item_list.add_iid_no_duplicate("IID_氷塊");
            }
            1 => {  // Corrupted Wyvern
                unit.item_list.add_iid_no_duplicate("IID_瘴気のブレス");
                unit.item_list.add_iid_no_duplicate("IID_瘴気の塊");
            }
            2|3 => {    // Wolves
                unit.item_list.add_iid_no_duplicate("IID_牙");
            }
            4|7 => {
                unit.item_list.add_iid_no_duplicate("IID_邪竜石_魔法攻撃");
                unit.item_list.add_iid_no_duplicate("IID_邪竜石");
            }   
            5|6 => {    //Wyrms
                unit.item_list.add_iid_no_duplicate("IID_火のブレス");
                unit.item_list.add_iid_no_duplicate("IID_炎塊");
            }
            _ => {},
        }
        add_equip_condition(unit);
    }
    if unit.person.get_asset_force() == 0 { unit.item_list.add_iid_no_duplicate("IID_特効薬"); }
    // "Monster Class Weapons Added");
}


#[unity::class("App", "Transporter.Data")]
pub struct TransporterData {
    pub unit_item: &'static mut UnitItem,
}

#[skyline::from_offset(0x022a13d0)]
fn get_item_from_index(index: i32, method_info: OptionalMethod) -> &'static mut TransporterData;

#[skyline::from_offset(0x022a1990)]
fn get_empty_index(method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x01c53f80)]
fn unit_pool_get(index:i32, method_info: OptionalMethod) -> Option<&'static mut Unit>;

#[skyline::from_offset(0x01a436b0)]
fn unit_can_equip_item(unit: &Unit, index: i32, rod: bool, exp: bool, method_info: OptionalMethod) -> bool;