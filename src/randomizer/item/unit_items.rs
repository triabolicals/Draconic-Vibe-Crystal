use engage::force::ForceType;
use utils::get_rng;
use super::*;
use super::item_rando::*;
use crate::randomizer::person::unit;

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
    if e_skills.is_some() { return e_skills.unwrap().find_sid("SID_轟雷発動可能".into()).is_some();}
    return false;
}
pub fn get_replacement_type(item: &UnitItem) -> i32 {
    let e_skills = item.get_equipped_skills();
    if e_skills.is_some() {
        if e_skills.unwrap().find_sid("SID_スマッシュ".into()).is_some() { return 6; }  // Smash
        if e_skills.unwrap().find_sid("SID_轟雷発動可能".into()).is_some() { return 7; } // Thunder
    }
    let iid = item.item.iid.get_string().unwrap();
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
        if new_weapon_type < 0 || new_weapon_type > 2 { new_weapon_type = 0; }
    }
    // Random Weapons for Enemy
    if is_enemy {
        let new_item = item_rando::WEAPONDATA.lock().unwrap().get_new_weapon(item, new_weapon_type, true);
        if new_item.is_some() {
            let n_item = new_item.unwrap();
            item.ctor(new_item.unwrap());
            if n_item.flag.value & 2 == 0 { item.set_flags(flag); }
            return;
        }
        let generic_weapon = item_rando::WEAPONDATA.lock().unwrap().get_generic_weapon(new_weapon_type, level);
        if generic_weapon.is_some() {
            item.ctor(generic_weapon.unwrap());
            item.set_flags(flag);
            return;
        } 
    }
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
    //println!("Replace item {} , weapon mask {}, max level {}", item.item.name.get_string().unwrap(), weapon_mask, max_rank);
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
pub fn dispose_tomes(item_list: &UnitItemList){
    for x in 0..8 {
        let item = item_list.get_item(x);
        if item.is_some() {
            let staff = &item.unwrap();
            let kind = staff.item.get_kind(); 
            if kind == 6 { staff.dispose(); }
        }
    }
}
pub fn remove_duplicates(item_list: &UnitItemList) {
    for x in 0..8 {
        let item = item_list.get_item(x as i32);
        if item.is_some() {
            let unit_item = item.unwrap();
            if unit_item.is_drop() { 
                if unit_item.item.flag.value & 131 != 0 { unit_item.flags = 0; }
            }
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
    let jid = job.jid.get_string().unwrap();
    remove_duplicates(unit.item_list);
    let is_vander = GameVariableManager::get_string("G_R_PID_ヴァンドレ").get_string().unwrap() == unit.person.pid.get_string().unwrap();
    let is_veyle = unit.person.pid.get_string().unwrap() == "PID_ヴェイル";
    let rng = Random::get_system();
    if weapon_mask.value & 64 != 0 && (!is_vander && !is_veyle) {
        dispose_tomes(unit.item_list);
        let value = rng.get_value(10);
        let job_level = job.get_max_weapon_level(6);
        if job.is_low() && unit.level <= 20 { 
            if GameVariableManager::get_bool("G_Cleared_M009") { 
                if value < 6 { unit.item_list.add_iid_no_duplicate("IID_エルファイアー"); }
                else { unit.item_list.add_iid_no_duplicate("IID_エルウィンド"); }
            }
            else { 
                if value % 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_ファイアー");  }
                else {unit.item_list.add_iid_no_duplicate("IID_ティラミス魔道書");  }
            }
            if GameVariableManager::get_bool("G_Cleared_M007") {
                if value & 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_サージ");  }
                else if GameVariableManager::get_bool("G_Cleared_M010") { unit.item_list.add_iid_no_duplicate("IID_エルサージ"); }
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
    let staff_level = if  unit::has_sid(unit, "SID_杖使い＋＋") { 4 } 
    else if unit::has_sid(unit, "SID_杖使い＋") { 3 }
    else if unit::has_sid(unit, "SID_杖使い") { 2 } 
    else { job.get_max_weapon_level(7) };

    if weapon_mask.value & ( 1 << 7 ) == 0 { replace_staves(unit.item_list); }
    else if weapon_mask.value & ( 1 << 7 ) != 0 && staff_level > 0 {
        dispose_staves(unit.item_list);
        if unit.person.get_asset_force() == 0 { //Player Staff users
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
                let staff_ = WEAPONDATA.lock().unwrap().get_staff(x, staff_level);
                if staff_.is_some() { unit.item_list.add_item_no_duplicate(staff_.unwrap()); }
            }
            /*
            // Heal Staff
            match staff_level {
                // A and Above
                4|5|6 => {
                    unit.item_list.add_iid_no_duplicate("IID_リザーブ");
                    match value {
                        1 => { 
                            if GameVariableManager::get_bool("G_Cleared_M020") { unit.item_list.add_iid_no_duplicate("IID_ドロー"); }
                            else { unit.item_list.add_iid_no_duplicate("IID_フリーズ"); }
                        },
                        2 => {  unit.item_list.add_iid_no_duplicate("IID_ワープ"); },
                        3 => { unit.item_list.add_iid_no_duplicate("IID_レスキュー"); },
                        _ => {  unit.item_list.add_iid_no_duplicate("IID_フリーズ");   },
                    }
                },
                // B Rank 
                3 => {
                    if job.is_low() {
                        if value % 3 == 0 { unit.item_list.add_iid_no_duplicate("IID_カップケーキ杖"); }
                        else if value % 3 == 1 { unit.item_list.add_iid_no_duplicate("IID_リブロー_G004"); }
                        else { unit.item_list.add_iid_no_duplicate("IID_リライブ"); }
                    }
                    else {
                        unit.item_list.add_iid_no_duplicate("IID_リカバー"); // Recover
                    }
                    match value {
                        1 => {  unit.item_list.add_iid_no_duplicate("IID_コラプス"); }, // Fracture
                        2 => { 
                            if GameVariableManager::get_bool("G_Cleared_M017") || GameVariableManager::get_bool("G_Cleared_S006") {
                                unit.item_list.add_iid_no_duplicate("IID_ワープ"); //Warp
                            }
                            else { unit.item_list.add_iid_no_duplicate("IID_サイレス");  }
                        },   
                        3 => { unit.item_list.add_iid_no_duplicate("IID_レスキュー"); },  // Rescue
                        _ => { unit.item_list.add_iid_no_duplicate("IID_フリーズ"); },   // Freeze
                    }
                },
                2 => {
                    unit.item_list.add_iid_no_duplicate("IID_リブロー_G004"); 
                    if value % 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_フリーズ"); }
                    else { unit.item_list.add_iid_no_duplicate("IID_サイレス");}
                },
                1 => { unit.item_list.add_iid_no_duplicate("IID_リライブ"); },
                _ => {}
            }
            */
        }
    }
    let pid = unit.person.pid.get_string().unwrap();
    remove_duplicates(unit.item_list);
    if jid == "JID_裏邪竜ノ娘" || jid == "JID_裏邪竜ノ子" { 
        unit.private_skill.add_sid("SID_オヴスキュリテ装備可能", 10, 0);    //Equip Obscurite
        if pid != "PID_エル" && pid != "PID_ラファール" {
            let dragonstone = WEAPONDATA.lock().unwrap().get_dragon_stone();
            if let Some(stone) = dragonstone {
                unit.item_list.add_item_no_duplicate(stone);
            }
        }
    }
    if jid == "JID_神竜ノ子" || jid == "JID_神竜ノ王" {
        unit.private_skill.add_sid("SID_リベラシオン装備可能", 10, 0);
        unit.private_skill.add_sid("SID_ヴィレグランツ装備可能", 10, 0);
    }
    let completed_m017 = GameVariableManager::get_bool("G_Cleared_M017");
    fix_weapons_by_rank(unit, completed_m017);
    if is_veyle {
        unit.item_list.add_iid_no_duplicate("IID_オヴスキュリテ"); 
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
        unit.put_off_all_item();
        unit.item_list.add_iid_no_duplicate("IID_オヴスキュリテ"); 
        unit.private_skill.add_sid("SID_オヴスキュリテ装備可能", 10, 0); //Equip Obscurite
        unit.private_skill.add_sid("SID_ミセリコルデ装備可能", 10, 0);  //Equip Misercode
        if unit.person.get_asset_force() == 0 { // Vul or Elixir
            if unit.get_capability(0, false) >= 45 { unit.item_list.add_iid_no_duplicate("IID_特効薬");   }
            else { unit.item_list.add_item_no_duplicate(ItemData::get("IID_傷薬").unwrap());  }
        }
    }
    add_generic_weapons(unit);
    adjust_melee_weapons(unit);
    remove_duplicates(unit.item_list);
    add_equip_condition(unit);
}



pub fn fix_weapons_by_rank(unit: &Unit, upgrade_weapon: bool) {
    println!("Fixing weapons by rank for {}", Mess::get(unit.person.get_name().unwrap()).get_string().unwrap());
    let unit_level = unit.level as i32 + unit.internal_level as i32;
    let weapon_rank = if !upgrade_weapon { 0 }
    else if unit_level < 25 { 2 }
    else { 3 };

    let job = unit.get_job();
    let pid = unit.person.pid.get_string().unwrap();
    for x in 0..unit.item_list.unit_items.len() {
        let item = unit.item_list.get_item(x as i32);
        if item.is_none() { continue;}
        let weapon = item.unwrap();
        if !can_change_weapon(weapon, false) { continue; }
        let iid = weapon.item.iid.get_string().unwrap();
        if iid == "IID_エンゲージ枠" || iid == "IID_不明" { continue; } // if engage weapon slot or none skip
        if iid == "IID_ミセリコルデ" && pid != "PID_ヴェイル" {
            weapon.dispose();
            continue;
        }
        let kind = weapon.item.kind;
        println!("Item {}: {} Kind: {}", x, weapon.item.name.get_string().unwrap(), kind);
        let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
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
    //if unit.person.get_asset_force() != 0 { // Making sure enemies have weapons
    println!("Adding Weapons for {}", Mess::get(unit.person.get_name().unwrap()).get_string().unwrap());
    let job = unit.get_job();
    let jid = job.jid.get_string().unwrap();
    if jid == "JID_ボウナイト" { unit.item_list.add_item_no_duplicate(ItemData::get("IID_銀の弓").unwrap());  }
    if jid == "JID_エンチャント" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_HPの薬").unwrap());  
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_力の薬").unwrap());  
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
        if kind > 9 || kind == 0 { continue; }
        let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
        if weapon.item.get_weapon_level() <= weapon_level && ( ( unit.selected_weapon_mask.value & ( 1 << kind ) != 0 || unit.selected_weapon_mask.value == 0 ) ) {
            has_weapon[kind as usize] = true;
        }
        else {
            if weapon.flags & 2 == 0 { weapon.dispose();  }
        }
    }
    for i in 1..9 {
        if i == 7 { continue; } // Rod
        if has_weapon[i as usize] { continue; }
        if i == 8 {
            if job.get_max_weapon_level(i) >= weapon_rank && ( unit.selected_weapon_mask.value & ( 1 << i ) != 0 || unit.selected_weapon_mask.value == 0 ) {
                let index = 7*(weapon_rank - 1) + i - 2;
                if index < 35 { unit.item_list.add_item_no_duplicate(ItemData::get(STANDARD_WEPS[index as usize]).unwrap()); }
            }
        }
        else {
            if job.get_max_weapon_level(i) >= weapon_rank && ( unit.selected_weapon_mask.value & ( 1 << i ) != 0 || unit.selected_weapon_mask.value == 0 ) {
                let index = 7*(weapon_rank - 1) + i - 1;
                if index < 35 { unit.item_list.add_item_no_duplicate(ItemData::get(STANDARD_WEPS[index as usize]).unwrap());  }
            }
        } 
    } 
    if job.get_max_weapon_level(4) >= 2 && GameVariableManager::get_bool("G_Cleared_M017") { unit.item_list.add_item_no_duplicate(ItemData::get("IID_長弓").unwrap());  } 
}

pub fn random_items_drops(unit: &Unit){
    let rng = Random::get_system();
    let mut none_count = 0;
    let mut rate = GameVariableManager::get_number("G_ItemDropGauge");
    if GameUserData::get_difficulty(false) == 2 {
        rate = rate / 2;
    }
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_none() { continue; }
        let u_item = &mut item.unwrap();
        if u_item.is_drop(){
            if u_item.item.flag.value & 3 != 0 || u_item.item.flag.value & 128 != 0 { 
                u_item.flags = 0;  
                continue;
            }
            else if !u_item.is_equip() { 
                let new_item = get_random_item(u_item.item.iid, false);
                u_item.ctor_str(&new_item.get_string().unwrap());
                u_item.flags = 2;
                none_count += 1;
            }
        } 
        let iid1 =  u_item.item.iid.get_string().unwrap();
        if iid1 == "IID_無し" && none_count == 0 { 
            if rng.get_value(100) < rate {
                u_item.ctor_str(&random_item(4, false).get_string().unwrap());
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

// Meteor Adjustment
pub fn adjust_items() {
    let mut meteor = ItemData::get_mut("IID_メティオ").unwrap();
    meteor.endurance = 1;
    let index = meteor.parent.index;
    let item_list = ItemData::get_list_mut().unwrap();
    for x in 0..item_list.len() {
        let random_item = &mut item_list[x];
        if random_item.get_flag().value == 16 {
            let iid = random_item.iid.get_string().unwrap();
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
        let uses = if unit.force.is_none() { 255 }
            else if unit.force.unwrap().force_type == 1 { 255 }
            else { 1 };

        for y in 0..8 {
            if unit.item_list.unit_items[y].item.parent.index == index {
                unit.item_list.unit_items[y].set_endurance(uses);
            }
        }
    }
    let convoy_index = unsafe { get_empty_index(None) };
    for x in 0..convoy_index {
        let item = unsafe {  get_item_from_index(x, None ) };
        if item.unit_item.item.parent.index == index {
            item.unit_item.endurance = 1;
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

fn add_equip_condition(unit: &Unit) {
    for x in 0..8 {
        if unit.item_list.unit_items[x].item.equip_condition.is_some() {
            let sid = unit.item_list.unit_items[x].item.equip_condition.unwrap().get_string().unwrap();
            unit.private_skill.add_sid(&sid, 10, 0);
        }
    }
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
