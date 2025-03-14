use crate::utils::{is_null_empty, str_contains, sub_string};
use super::{accessory::*, data::*, emblem::*};
use concat_string::concat_string;
use super::*;

const CAV_JIDS: [&str; 15] = ["JID_ボウナイト", "JID_紋章士_シグルド", "JID_マージナイト", "JID_グレートナイト", "JID_ロイヤルナイト", "JID_ソードナイト", "JID_ランスナイト", "JID_アクスナイト", "JID_パラディン", "JID_クピードー下級", "JID_クピードー", "JID_アヴニール下級", "JID_アヴニール", "JID_アヴニール_E", "JID_クピードー_E"];
const ARMOR_JIDS: [&str; 4] = ["JID_ソードアーマー", "JID_ランスアーマー", "JID_アクスアーマー", "JID_マージカノン"];
const WYVERN: [&str; 6] = ["JID_紋章士_カミラ", "JID_ドラゴンナイト", "JID_リンドブルム下級", "JID_リンドブルム", "JID_メリュジーヌ_味方", "JID_リンドブルム_E"];
const FLIER: [&str; 6] = ["JID_ソードペガサス", "JID_ランスペガサス", "JID_アクスペガサス", "JID_スレイプニル下級", "JID_スレイプニル", "JID_スレイプニル_E"];
pub const MONSTERS: [&str; 8] = ["JID_幻影飛竜", "JID_異形飛竜", "JID_幻影狼", "JID_異形狼",  "JID_E006ラスボス", "JID_幻影竜", "JID_異形竜", "JID_邪竜"];
pub const WEP_PRE: [&str ;10] = ["No", "Sw", "Lc", "Ax", "Bw", "Dg", "Mg", "Rd", "Ft", "No"];
pub const ACT_PRE: [&str; 34] = ["Cav1B", "Amr0A", "Amr1A", "Wng2D", "Wng0E", "Wng1F", "Cav2C", "Mcn3A", "Bow0A", "Bow1A", 
"Bow2B", "Mag0A", "Mag1A", "Mag2B", "Lnd1D", "Slp1E", "Cmi0D", "Enb0A", "Dge0A", "Swd0A", "Swd1A", "Com0A", "Cav0B", "Msn0D", "Dnc0A", "Rod0A", "Rod1A", "Lnc0A", "Lnc1A", "Axe0A", "Axe1A", "Lnc2B", "Swd2A", "Ect3A"];
pub const INF_ACT: [&str; 10] = ["Com0A", "Swd0A", "Lnc0A", "Axe0A", "Bow0A", "Dge0A", "Mag0A", "Rod0A", "Rod0A", "Rod0A"];
pub const EATK_ACT: [&str; 22] = ["Mar1A", "Sig1B", "Cel1A", "Mic1A", "Roy1A", "Lei1A", "Luc1A", "Lyn1A", "Ike1A", "Byl1A", "Cor1A", "Eir1A", "Thr1A", "Tik1A", "Hec1A", "Ver1A", "Sor1A", "Cmi1D", "Chr1A", "Ler1A", "Thr2A", "Ver2A"];
pub static mut LINK_COUNT: i32 = 0;
pub const MAGIC: [&str; 31 ] = ["Fire", "Thunder", "Wind", "Surge", "WpFire", "LFire", "LThunder", "LWind", "LSurge", "Bolganone", "Thoron", "SThoron", "Excalibur", "Micaiah_Thani", "Micaiah_Shine", "Celica_Ragnarok", "Micaiah_Nosferatu", "Obscurite", "Nova", "Meteo", "WpWind", "WpByl", "WpNature", "WpEir", "WpRoy", "WpLight", "WpLer", "WpThunder", "FlameBall", "IceBreath", "IceBall"];
pub const ROD: [&str; 16] = ["Heal", "HiHeal", "FarHeal", "Recover", "WholeHeal", "Rest", "Warp", "ReWarp", "Rescue", "Draw", "TorchRod", "Freeze", "Silence",  "Block", "Collapse", "Dance"];

pub fn find_and_replace_body_animation(result: &mut AssetTableResult, body_act: String, to_replace: &str) {
    if let Some(animation) = result.body_anims.iter_mut().find(|act| str_contains(act, to_replace)) {
        //println!("Replaced {} with {}", to_replace, body_act);
        *animation = Il2CppString::new_static(body_act.clone());
    }
    if let Some(last) = result.body_anims.iter_mut().last() {  *last = Il2CppString::new_static(body_act.clone()); }
    //result.body_anims.iter_mut().for_each(|animation| println!("Body Replace Act: {}", animation.to_string()));
    //if count == 0 { result.body_anims.add(Il2CppString::new_static(body_act.clone())); }
}
fn assign_mounts(result: &mut AssetTableResult, act_type: i32, job_act: &str, job_suffix: &str, mode: i32) {
    if mode == 1 {
        match act_type {   //Mounts
            0|4|6 => {   
                result.ride_model = concat_string!("oBody_", job_act, "R_", job_suffix).into(); 
                result.ride_anim = Some(concat_string!("UAS_", ACT_PRE[act_type as usize], "R").into()); 
            }
            14 => {
                result.ride_model = "oBody_Lnc2BR_c000".into();
                result.ride_anim = Some("UAS_Lnc2BR".into());
            }
            5 => {
                result.ride_anim = Some("UAS_Wng1FR".into());
                result.ride_model = "oBody_Wng1FR_c000".into(); 
                result.scale_stuff[18] = 0.50;
            }
            3|12 => {
                result.ride_model = concat_string!("oBody_", job_act, "R_", job_suffix).into(); 
                result.ride_anim = Some(concat_string!("UAS_", ACT_PRE[act_type as usize], "R").into()); 
                result.scale_stuff[18] = 0.50;
            }
            _ => {
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into();
                result.ride_anim = None;
            },
        }
    }
    else {
        let ride_dress_model: String = concat_string!("uBody_", job_act, "R_", job_suffix).into();
        match act_type {   //Mounts
            3|12 => { 
                result.ride_model = "uRig_DragR".into();
                result.ride_dress_model = ride_dress_model.clone().into();
            }
            4 => {
                result.ride_model = "uRig_PegaR".into();
                result.ride_dress_model = ride_dress_model.clone().into();
            }
            5 => {
                result.ride_model = "uRig_GrifR".into();
                result.ride_dress_model = "uBody_Wng1FR_c000".into();
            }
            6 => {
                result.ride_model = "uRig_WolfR".into();
                result.ride_dress_model = ride_dress_model.clone().into();
            }
            0|11|14 => { 
                result.ride_dress_model = ride_dress_model.clone().into(); 
            }
            _ => {
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into();
            },
         }
    }



}
pub fn find_animation_set(anim_set: String) -> String {
    if anim_set.contains("Avn1BM-Sw") { 
        return "Avn1BM-Sw1_c100_N".to_string();
    }
    if anim_set.contains("Msn1DF-Mg") { return "Msn1DF-Mg1_c553_N".to_string(); }
    let list = AnimSetDB::get_list().unwrap();
    if let Some(act) = list.iter().find(|f| str_contains(f.name, anim_set.as_str())) { return act.name.to_string(); }
    if anim_set.contains("M-No") { return "Com0AM-No1_c000_N".to_string(); }
    else if anim_set.contains("F-No") { return "Com0AF-No1_c000_N".to_string(); }
    anim_set
}

pub fn is_mounted(act_type: i32) -> bool {
    match act_type { 0|3|4|5|6|11|12|14 => { true }
        _ => { false}
    }
}

pub fn get_animation_type(job: &JobData) -> i32 {
    let jid = job.jid.to_string();
    if CAV_JIDS.iter().any(|jid| str_contains(job.jid, jid)) && job.is_high() { 
        if jid.contains("JID_ロイヤルナイト") { 14 }   // Royal Knight
        else { 0 }
    }  //Cav
    else if job.jid.contains("エンチャント") { 16 }
    else if job.jid.contains("マージカノン") { 7 } //Mage Cannon
    else if job.jid.contains("JID_メリュジーヌ") { 12 } //Melsuine
    else if ARMOR_JIDS.iter().any(|&ajid| jid == ajid) { 1 } //Armor 1
    else if jid.contains("JID_ジェネラル") { 2 } //Armor 2
    else if WYVERN.iter().any(|&ajid| jid == ajid) { 3 } // WYVERN
    else if FLIER.iter().any(|&ajid| jid == ajid) { 4 } // Flier Pegasus
    else if jid.contains("JID_グリフォンナイト") { 5 } // Griffin
    else if jid.contains("JID_ウルフナイト") { 6 } //Wolf
    else if jid.contains("JID_ダンサー") { 13 }
    else if job.move_type == 1 && job.is_low() { 9 } // Infantry
    else if job.move_type == 1 && job.is_high() { 
        if jid == "JID_ブレイブヒーロー" { 15 }
        else { 10 }
    }
    else if CAV_JIDS.iter().any(|jid| str_contains(job.jid, jid)) && job.is_low() { 11 }
    else { 8 } // Emblem
}

pub fn get_animation_index(job: &JobData, equipped: Option<&ItemData>, is_emblem: bool, is_female: bool) -> i32 {
    let mut act_type = get_animation_type(job);
    if is_emblem && ( act_type == 9 || act_type == 10 ) {
        if act_type == 12 && !is_female { act_type = 8;  }
        else if act_type != 12 { act_type = 8;  }
    }
    let kind = if let Some(item) = equipped { item.kind } else { 0 } as i32;
    match (act_type, kind) {
        (14, 0|2|6) => { 31 },   
        (16, 5|8) => { 33 },
        (15, 1|2|3) => { 32 },  //Hero Swrd
        (13, _) => { if is_female { 25 } else { 24 } }   //Dancer
        (12, 0) => { if is_female { 23 } else { 3 } }   // Melsuine
        (11, 0|1|2|3) => { 22 }, //Unpromoted Cav
        (7, 9) => { 7 },    // Mage Cannon
        (4, 1|2|3) => { //Pegasus
            if is_female { 4 }
            else { 5 }  // Male to Griffin
        }
        (4, 6) => { 
            if is_female { 15  }
            else { 17 }
        }, 
        (0|1|2|3|5|6|7, 0|1|2|3) => { act_type }, // Cav/Armor/Flyers with Sword/Lance/Axe
        (0|11, 4) => { 10 }, // Bow Cav
        (0|11, 6) => { 13 }, // Mage Cav
        (1|7|9, 4) => { 8 },  //Armor Bow1
        (2|10, 4) => { 9 },    //Armor2 Bow2
        (1|7|9, 6) => { 11 }, //Armor / Infantry Magic
        (2|10, 6) => { 12 },   //Armor / Promoted Infantry Magic
        (3, 6) => { 
            if is_female {
                if is_emblem { 16}
                else { 14 }
            }
            else { 17 }
        },   //Camilla / Lindwurm Magic
  //Slephir Magic
        (6, 5) => { act_type }, //Wolf Dagger
        (1|2|9|10, 5) => { 18 }, // Infantry Dagger
        (9, 1) => { 19 },   //Infantry Sword
        (10, 1) => { 20 }, // Promoted Infantry Sword
        (9, 2) => { 27 },
        (10, 2) => { 28 },
        (9, 3) => { 29 },
        (10, 3) => { 30 },
        (9, 8) => { 25 },
        (10, 8) => { 26 },  
        (12, 1) => { if is_female { 23  } else { 3 } },
        (12, 6) => { if is_female { 23 } else { 17 }},
        (12 , 2|3) => { 3 },
        (_, 0) => { 
            if act_type == 12 && !is_female { 3 }
            else if act_type == 4 && !is_female { 5 }
            else { act_type }
         }, // XXXG-No1
        _ => { 17 }, // generic emblem 
    }
}
pub fn check_change_animation_type(job: &JobData, item: Option<&ItemData>, is_female: bool) -> bool {
    let act_type = get_animation_type(job);
    let kind = if let Some(i) = item { i.kind } else { 0 };
    match (act_type, kind) {
        (0|11, 5|8|9) => { true }, // Cav with Dagger/Fist
        (3|4, 4|5|8|9) => { true }, //Wyvern/Flier with Bow/Dagger/Fist
        (3|4, 6) => { !is_female },
        (4, 1|2|3) => { !is_female },   //Male Pegs change
        (5, 4|5|6|8|9) => { true }, //Griffin Bow/Dagger/Tome/Fist 
        (6, 4|6|8|9) => { true },   // Wolf Knight: Bow/Dagger/Tome/Fist
        (12, 4|5|7|8) => { true }, 
        (12, _) => { !is_female }, //Melsuine
        _ => { false },
    }
}

pub fn unique_class_dress(job: &JobData, result: &mut AssetTableResult, gender: i32, mode: i32, unit: &Unit, is_engage: bool, remove_mounts: bool) {
    let gender_str = if gender == 1 { "M" } else { "F" };
    if is_engage {
        if unit.person.parent.hash == 258677212 {   //Lumera Chapter 2
            if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                let gid = GameVariableManager::get_string("G_R_GID_シグルド").to_string();
                if let Some(pos) = EMBLEM_ASSET.iter().position(|asset| gid.contains(asset)) {
                    let body = match pos {
                        12|20|21 => { ENGAGE_PREFIX[12]},
                        23 => { ENGAGE_PREFIX[18] },
                        22 => { ENGAGE_PREFIX[11] },
                        _ => { ENGAGE_PREFIX[pos] }
                    };
                    if pos == 13 {
                        if mode == 2 { 
                            result.dress_model = "uBody_Tik0AF_c560".into(); 
                            result.body_anims.add( Il2CppString::new_static("Sds0AF-No2_c099_N")); 
                        }
                        else { result.body_model = "oBody_Tik0AF_c560".into(); }
                    }
                    else {
                        if mode == 2 { result.dress_model = concat_string!("uBody_", body, "1A", gender_str, "_c000").into(); }
                        else { result.body_model = concat_string!("oBody_", body, "1A", gender_str, "_c000").into();}
                    }
                    change_hair_change(unit, result);
                }
            }
            return;
        }
        else { set_accessories_generic(result, unit.person.aid, gender); }  //Corrupted
    }
    let uniques = UNIQUE_JOB_DATA.get().unwrap();
    if gender == 0 || gender > 2 { return; }
    if mode == 2 && unit.person.get_asset_force() != 0 && GameVariableManager::get_bool(DVCVariables::ENEMY_OUTFIT_KEY) {
        set_accessories_for_unit(unit, result);
        return;
    }
    let outfit = get_unit_outfit_mode(unit);
    let battle_outfits = unit.person.get_asset_force() == 0;
    let has_body = 
        if outfit == 1 { unit.accessory_list.unit_accessory_array[0].index > 0 }
        else if outfit == 2 && unit.accessory_list.unit_accessory_array.len() >= 5 {  unit.accessory_list.unit_accessory_array[5].index > 0 }
        else { false };

    if is_engage && unit.get_god_unit().is_some() {
        if let Some(gunit) = unit.get_god_unit() {
            let gid = gunit.data.gid.to_string();
            if let Some(pos) = EMBLEM_ASSET.iter().position(|asset| gid.contains(asset)) {
                let body = match pos {
                    12|20|21 => { ENGAGE_PREFIX[12]},
                    13 => { 
                        if gid.contains("相手") { super::emblem::tiki_engage(result, unit, mode); }
                        return; 
                    }   //Tiki
                    23 => { ENGAGE_PREFIX[18] },
                    22 => { ENGAGE_PREFIX[11] },
                    _ => { ENGAGE_PREFIX[pos] }
                };
                if outfit == 1 && mode == 2 && battle_outfits { set_accessories_for_unit(unit, result); }
                else if outfit == 0 && mode == 2 { result.dress_model = concat_string!("uBody_", body, "1A", gender_str, "_c000").into(); }
                else if outfit == 0 && mode == 1 { result.body_model = concat_string!("oBody_", body, "1A", gender_str, "_c000").into();}
            }
        }
    }
    else {

        if mode == 2 && (outfit == 0 || !battle_outfits || (battle_outfits && !has_body))   {
            if result.dress_model.to_string().contains("Swd0A") {
                let hash = job.parent.hash;
                if let Some(custom) = CUSTOM_CLASS_ASSETS.get().unwrap().iter().find(|assets| assets.job_hash == job.parent.hash) {
                    custom.replace(result, gender, mode, -1, false);
                }
                if let Some(ujob) = uniques.iter().find(|unique| hash == unique.job_hash && gender == unique.gender) {
                    result.dress_model = concat_string!("uBody_", ujob.act_prefix, gender_str, "_", ujob.act_suffix).into(); 
                }
            }
        }
    }
    if remove_mounts {
        result.ride_model = "null".into();
        result.ride_dress_model = "null".into();
        result.ride_anim = None;
    }
    if mode == 2 && outfit > 0 { if battle_outfits && outfit == 1 { set_accessories_for_unit(unit, result);  }  }
    if mode == 2 {
        if CONFIG.lock().unwrap().misc_option_1 >= 4.75 && (unit.person.gender == 1 || unit.person.gender == 2 ){   // Bust Rando using Grow Seed
            let rng = Random::instantiate().unwrap();
            rng.ctor(unit.grow_seed as u32);
            result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.025;
        }
    }

}

pub fn change_unique_class(job: &JobData, result: &mut AssetTableResult, mode: i32, gender: i32, equipped: Option<&ItemData>, is_emblem: bool) -> bool {
    // Custom Classes
    let kind = if let Some(item) = equipped { item.kind } else { 0 } as i32;
    if let Some(custom) = CUSTOM_CLASS_ASSETS.get().unwrap().iter().find(|assets| assets.job_hash == job.parent.hash) {
        custom.replace(result, gender, mode, kind, false);
        return true;
    }
    let uniques = UNIQUE_JOB_DATA.get().unwrap();
    let jid = job.jid.to_string();
    let hash = job.parent.hash;
    let gender_str = if gender == 1 { "M" } else { "F" };
    if let Some(ujob) = uniques.iter().find(|unique| unique.job_hash == hash && gender == unique.gender) {
        if mode == 1 {
            if result.body_model.to_string().contains("oBody_Swd0A"){
                result.body_model = concat_string!("oBody_", ujob.act_prefix, gender_str, "_", ujob.act_suffix).into();
            }
            let ride_act = match ujob.act_type {   //Mounts
                3|4|12 => {  
                    result.ride_model = concat_string!("oBody_", ujob.act_prefix.clone().replace("1", "0"), "R_", ujob.act_suffix).into();
                    result.scale_stuff[18] = 0.50;
                    true
                }
                0 => {
                    result.ride_model = concat_string!("oBody_", ujob.act_prefix.clone().replace("1", "0"), "R_", ujob.act_suffix).into();
                    true
                }
                _ => { false },
            };
            let new_prefix = if jid.contains("下級") { ujob.act_prefix.clone().replace("0", "1") }
                else { ujob.act_prefix.clone() };
            if ujob.weapon_mask & (1 << kind) != 0 || kind == 0  {
                let act = concat_string!("UAS_", new_prefix, gender_str);
                result.body_anim = Some(act.clone().into());
                result.body_anims.add(Il2CppString::new_static(act.clone())); 
                if is_emblem { find_and_replace_emblem_animation(result, act)} 
                else { result.body_anims.add(Il2CppString::new_static(act));  }
                if ride_act { result.ride_anim = Some(concat_string!("UAS_", new_prefix, "R").into()); }
            }
            else {
                let new_act = get_animation_index(job, equipped, false, gender == 2 ) as usize;
                if new_act >= 17 && new_act != 22 {  // Use Engaging Animations
                    result.ride_dress_model = "null".into();
                    result.ride_model = "null".into();
                    result.ride_anim = None;
                }
                let act = concat_string!("UAS_", ACT_PRE[new_act], gender_str);
                result.body_anim = Some(act.clone().into());
                if is_emblem { find_and_replace_emblem_animation(result, act); } 
                else { result.body_anims.add(Il2CppString::new_static(act.clone()));  }
                if ride_act {
                    if kind < 4 {   // Hortensia and Ivy's class not having mode 1 for meele weapons
                        if ujob.act_type == 3 { result.ride_anim = Some(concat_string!("UAS_Wng2DR").into()); } 
                        if ujob.act_type == 4 { result.ride_anim = Some(concat_string!("UAS_Wng0ER").into()); } 
                        else { result.ride_anim = Some(concat_string!("UAS_", new_prefix, "R").into()); }
                    }
                    else { result.ride_anim = Some(concat_string!("UAS_", new_prefix, "R").into()); }
                }
            }

        }
        else {  //Mode2
            // println!("Head: {}", result.head_model);
            if gender == 2 && jid == "JID_邪竜ノ子" && !result.hair_model.to_string().contains("null") {
                clear_accessory_from_list(result.accessory_list, "uAcc_spine2_Hair052");
            }
            if result.hair_model.to_string().contains("801") || result.hair_model.to_string().contains("851") {
                HEAD_DATA.get().unwrap().unique_head_for_generic(result, job.parent.hash, ujob.gender == 2);
            } 
            if ujob.rig != "none" { result.body_model = ujob.rig.clone().into(); }
            if result.dress_model.to_string().contains("Swd") {
                result.dress_model = concat_string!("uBody_", ujob.act_prefix, gender_str, "_", ujob.act_suffix).into(); 
            }
               //Outfits
            match ujob.act_type {   //Mounts
                3|12 => { 
                    result.ride_model = "uRig_DragR".into();
                    result.ride_dress_model = concat_string!("uBody_", ujob.act_prefix.replace("1", "0"), "R_", ujob.act_suffix).into();
                }
                4 => {
                    result.ride_model = "uRig_PegaR".into();
                    result.ride_dress_model = concat_string!("uBody_", ujob.act_prefix.replace("1", "0"), "R_", ujob.act_suffix).into();
                }
                0 => { 
                    result.ride_dress_model = concat_string!("uBody_", ujob.act_prefix.replace("1", "0"), "R_", ujob.act_suffix).into(); 
                    if kind < 4 { result.body_anims.add(Il2CppString::new_static( concat_string!("Com0B", gender_str, "-", WEP_PRE[kind as usize], "_c000_N") )); }
                }
                8 => {
                    result.body_anims.add(Il2CppString::new_static( concat_string!("Com0A", gender_str, "-", WEP_PRE[kind as usize], "_c000_N") ))
                }
                _ => {},
            }
            // println!("Mode {} for Unique Job: {} for {}", mode, Mess::get_name(job.jid).to_string(), Mess::get_name(unit.person.pid).to_string());
            if ujob.weapon_mask & (1 << kind) != 0 || kind == 0 {
                let act = if ujob.act_prefix.contains("Msn0") { concat_string!(ujob.act_prefix.replace("0", "1"), gender_str, "-", WEP_PRE[kind as usize]) }
                    else { concat_string!(ujob.act_prefix, gender_str, "-", WEP_PRE[kind as usize]) };
                let new_act = find_animation_set(act);
                result.body_anim = Some(new_act.clone().into());
                if kind == 9 && ujob.act_type == 8 {
                    let new_act = if gender == 1 { "Sds0AM-No2_c049_N"} //DragonStone
                        else { "Sds0AF-No2_c099_N" };
                    result.body_anims.add(Il2CppString::new_static(new_act));
                }
                else if is_emblem { find_and_replace_emblem_animation(result, new_act); }
                else {
                    if let Some(last_act) = result.body_anims.iter_mut().last() { *last_act = Il2CppString::new_static(new_act); }
                }
                // Weapons
                if kind == 6 && ( jid.contains("メリュジーヌ") || jid.contains("邪竜ノ娘") ) {
                    result.right_hand = "null".into();
                    result.left_hand = "null".into();
                }
                else { super::edit_asset_weapon(result, false, equipped); }
            }
            else if ujob.alt_weapon_mask != 0 && ujob.alt_weapon_mask & (1 << kind) != 0 {
                let act = concat_string!(ujob.alt_act, gender_str, "-", WEP_PRE[kind as usize], "_c000_N");
                if is_emblem { find_and_replace_emblem_animation(result, act); }
                else { result.body_anims.add(Il2CppString::new_static(find_animation_set(act))); }
                super::edit_asset_weapon(result, false, equipped);
            }
            else {
                let new_act = if is_emblem { 17 } else { get_animation_index(job, equipped, false, gender == 2) } as usize;
                if  new_act >= 17 && new_act != 22 {  // Use Engaging Animations
                    result.ride_dress_model = "null".into();
                    result.ride_model = "null".into();
                }
                let act = concat_string!(ACT_PRE[new_act], gender_str, "-", WEP_PRE[kind as usize]);
                let new_act = find_animation_set(act.clone());
                result.body_anim = Some(new_act.clone().into());
                if is_emblem { find_and_replace_emblem_animation(result, act); }
                else {
                    if let Some(last_act) = result.body_anims.iter_mut().last() { *last_act = Il2CppString::new_static(new_act); }
                }
            }
        }
        return true;
    }
    else if is_emblem { return false; }
    else if let Some(ujob) = uniques.iter().find(|unique| hash == unique.job_hash && gender != unique.gender ) { // In Class Wrong Gender
        let new_act = get_animation_index(job, equipped, false, gender == 2) as usize;
        assign_mounts(result, new_act as i32, &ujob.act_prefix, &ujob.act_suffix, mode);
        if mode == 1 {
            let act = concat_string!("UAS_", ACT_PRE[new_act], gender_str);
            result.body_anim = Some(act.clone().into());
            result.body_anims.add(Il2CppString::new_static(act.clone())); 
        }
        else if mode == 2 {
            let act = concat_string!(ACT_PRE[new_act], gender_str, "-", WEP_PRE[kind as usize]);
            let new_act = find_animation_set(act);
            result.body_anim = Some(new_act.clone().into());
            if let Some(last_act) = result.body_anims.iter_mut().last() { *last_act = Il2CppString::new_static(new_act); }
        }
        return true;
    }
    return false;
}

pub fn incorrect_mount_animation_fix(unit: &Unit, result: &mut AssetTableResult, mode: i32, gender: i32, equipped: Option<&ItemData>) -> bool {
    let move_type = unit.job.move_type;
    if move_type == 1 || move_type >= 4 { return false; }
    let kind = if equipped.is_some() { equipped.unwrap().kind } else { 0 } as i32;
    if kind == 7 && mode == 1 { return false; }
    let gender_str = if gender == 1 { "M" } else { "F" };
    let act_type = get_animation_type(unit.job);
    if mode == 2 {
        if unsafe { is_null_empty(result.ride_dress_model, None) } { return false; }
        if gender == 1 && ( result.ride_dress_model.to_string().contains("Slp0E") || act_type == 4 ) {
            if kind < 4 {
                result.ride_dress_model = "uBody_Wng1FR_c000".into();
                result.ride_model = "uRig_GrifR".into();
                result.body_anims.iter_mut()
                    .filter(|x| x.to_string().contains("Wng0E"))
                    .for_each(|body| *body = Il2CppString::new_static(body.to_string().replace("Wng0EF", "Wng1FM")));

                result.body_anims.add(Il2CppString::new_static(concat_string!("Wng1FM-", WEP_PRE[kind as usize], "1_c000_N")));
                result.dress_model = "uBody_Wng1FM_c000".into();
            }
            else {
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into();
            }
            return true;
        }
        match kind {
            4 => {
                if move_type == 2 {
                    result.ride_model = "uRig_HorsR".into();
                    if result.ride_dress_model.to_string().contains("Cav2C") {
                        result.ride_dress_model = "uBody_Bow2BR_c000".into();
                        result.body_anims.add(Il2CppString::new_static(concat_string!("Com0B", gender_str, "-No1_c000_N")));
                    }
                    let act = concat_string!("Bow2B", gender_str, "-Bw1_c000_L");
                    result.body_anims.iter_mut().filter(|x| x.to_string().contains("Com0A") && x.to_string().contains("Bw1")).for_each(|body| *body = Il2CppString::new_static(act.clone()));
                    if !result.body_anims.iter().any(|x| x.to_string().contains("Cpd")) {  result.body_anims.add(Il2CppString::new_static(act)); }
                }
                else {
                    result.ride_dress_model = "null".into();
                    result.ride_model = "null".into();
                    if unit.job.is_high() { result.body_anims.add(Il2CppString::new_static(concat_string!("Bow1A", gender_str, "-Bw1_c000_L"))); }
                    else { result.body_anims.add(Il2CppString::new_static(concat_string!("Bow0A", gender_str, "-Bw1_c000_L"))); }
                }
            }
            5 => {  // Daggers
                if move_type == 2 {
                    result.ride_model = "uRig_WolfR".into();
                    result.ride_dress_model = "uBody_Cav2CR_c000".into();
                    let act = concat_string!("Cav2C", gender_str, "-Dg1_c000_N");
                    result.body_anims.iter_mut().filter(|x| x.to_string().contains("Dg1")).for_each(|body| *body = Il2CppString::new_static(act.clone()));
                    result.body_anims.add(Il2CppString::new_static(act));
                }
                else {
                    result.ride_dress_model = "null".into();
                    result.ride_model = "null".into();
                    result.body_anims.add(Il2CppString::new_static(concat_string!("Dge0A", gender_str, "-Dg1_c000_L")));
                }
            }
            6 => {
                if move_type == 2 {
                    if result.ride_dress_model.to_string().contains("Cav2C") {
                        result.ride_model = "uRig_HorsR".into();
                        result.ride_dress_model = "uBody_Mag2BR_c000".into();
                        result.body_anims.add(Il2CppString::new_static(concat_string!("Com0B", gender_str, "-No1_c000_N")));
                    }
                    let act = concat_string!("Mag2B", gender_str, "-Mg1_c000_M");
                    result.body_anims.iter_mut().filter(|x| x.to_string().contains("Mg1")).for_each(|body| *body = Il2CppString::new_static(act.clone()));
                }
                else if move_type == 3 && gender == 1 {
                    result.ride_dress_model = "null".into();
                    result.ride_model = "null".into();
                    if unit.job.is_high() { result.body_anims.add(Il2CppString::new_static("Mag1AM-Mg1_c000_M")); }
                    else { result.body_anims.add(Il2CppString::new_static("Mag0AM-Mg1_c000_M")); }
                }
            }
            8 => {
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into();
                if unit.job.is_high() { result.body_anims.add(Il2CppString::new_static(concat_string!("Rod1A", gender_str, "-Ft1_c000_N"))); }
                else { result.body_anims.add(Il2CppString::new_static(concat_string!("Rod0A", gender_str, "-Ft1_c000_N"))); }
            }
            _ => { return false; }
        }
    }
    else {
        if unsafe { is_null_empty(result.ride_model, None) } { return false; }
        if gender == 1 && ( result.ride_model.to_string().contains("Slp0E") || act_type == 4 ) {
            result.ride_anim = Some("UAS_Wng1FR".into());
            result.body_model = "oBody_Wng1FM_c000".into();
            result.body_anim = Some("UAS_Wng1FM".into());
            result.ride_model = "oBody_Wng1FR_c000".into();
            result.body_anims.add(Il2CppString::new_static("UAS_Wng1FM"));
            return true;
        }
        if move_type == 2 {
            let act = concat_string!("UAS_oBody_B", gender_str);
            result.body_anims.add(Il2CppString::new_static(act));
        }
        else if move_type == 3 {
            let act = concat_string!("UAS_oBody_F", gender_str);
            result.body_anims.add(Il2CppString::new_static(act));
        }
        let inf_act = concat_string!("UAS_oBody_A", gender_str);
        match kind {
            4 => {
                if move_type == 2 {
                    let act = concat_string!("UAS_Bow2B", gender_str);
                    if result.ride_model.to_string().contains("Cav2C") {
                        result.ride_model = "oBody_Bow2BR_c000".into();
                        result.ride_anim = Some("UAS_Bow2BR".into());
                        result.body_anim = Some(act.clone().into());
                        result.body_anims.add(Il2CppString::new_static(act.clone()));
                    }
                    if !result.ride_model.to_string().contains("Cpd") && !result.ride_model.contains("Bow2B") {
                        result.ride_anim = Some("UAS_Bow2BR".into());
                        result.body_anim = Some(act.clone().into());
                        result.body_anims.add(Il2CppString::new_static(act));
                    }
                }
                else {
                    result.ride_anim = None;
                    result.ride_model = "null".into();
                    result.body_anim = Some(inf_act.into());
                }
                return true;
            }
            5 => {
                if move_type == 2 {
                    let act = concat_string!("UAS_Cav2C", gender_str);
                    if !result.ride_model.to_string().contains("Cav2C") {
                        result.ride_model = "oBody_Cav2CR_c000".into();
                        result.ride_anim = Some("UAS_Cav2CR".into());
                        result.body_anim = Some(act.clone().into());
                        result.body_anims.add(Il2CppString::new_static(act));
                    }
                }
                else {
                    result.ride_anim = None;
                    result.ride_model = "null".into();
                    result.body_anim = Some(inf_act.into());
                }
                return true;
            }
            6 => {
                if move_type == 2 {
                    let act = concat_string!("UAS_Mag2B", gender_str);
                    if result.ride_model.to_string().contains("Cav2C") {
                        result.ride_model = "oBody_Mag2BR_c000".into();
                        result.ride_anim = Some("UAS_Mag2BR".into());
                        result.body_anim = Some(act.clone().into());
                        result.body_anims.add(Il2CppString::new_static(act));
                    }
                    else {
                        result.ride_anim = Some("UAS_Mag2BR".into());
                        result.body_anim = Some(act.into());
                    }
                    return true;
                }
                else if gender == 1 && move_type == 3 {
                    result.ride_anim = None;
                    result.ride_model = "null".into();
                    result.body_anim = Some(inf_act.into());
                    return true;
                }
            }
            _ => {}
        }
    }
    return false;
}

// For non-emblems
pub fn correct_animations(unit: &Unit, result: &mut AssetTableResult, mode: i32, equipped: Option<&ItemData>) {
    let gender = if unit.edit.is_enabled() {  unit.edit.gender }  else { unsafe { get_dress_gender(unit.person, None)} };
    if gender == 0 { return; }
    if mode != 1 && mode != 2 { return; }
    let gender_str = if gender == 1 { "M" } else { "F" };
    let job = &unit.job;
    let jid = job.jid.to_string();
    if MONSTERS.iter().any(|&mjob| jid == mjob) {  //Ignore Monster
        return;
    }
    // println!("Correcting Animations for {} in {}", Mess::get_name(unit.person.pid), Mess::get(unit.job.name));
    if change_unique_class(unit.job, result, mode, gender, equipped, false) {
       // result.body_anims.iter().for_each(|str|println!("Unique Body Anims {} {}", Mess::get(unit.job.name), str.to_string()) );
        // if let Some(body) = result.body_anim { println!("Unique Body Act {}: {}", Mess::get(unit.job.name),  body.to_string()); }
        // if let Some(ride) = result.ride_anim { println!("Unique Ride Act {}: {}", Mess::get(unit.job.name), ride.to_string());  }
               return;
    }
    let kind = if equipped.is_some() { equipped.unwrap().kind } else { 0 } as i32;
    if gender == 1 && job.move_type == 1 && mode == 2 && kind == 1 {  //Male Infantry replace last animation if it's Com0AM to Swd0AM
        if result.body_anims.iter_mut().filter(|act| act.to_string().contains("AM-Sw1") && !act.to_string().contains("Com0AM-Sw1") ).count() == 0 {
            result.body_anims.add(Il2CppString::new_static("Swd0AM-Sw1_c000_N"));
        }
    }
    // if kind == 7 { return; }
    let mut body;
    let replace;
    let anim_set = get_animation_type(job);
    let act_type = get_animation_index(job, equipped, false, gender == 2);
    if let Some(name) = unit.person.get_name() {
        let name_list = NAME_DATA.get().unwrap();
        let person_name = name.to_string();
        if let Some(anim_replace) = name_list.act_replace.iter().find(|x| x.0 == person_name ) {
            replace = anim_replace.1.clone();
            body = if mode == 1 { concat_string!("UAS_",ACT_PRE[act_type as usize], gender_str) }
                else {
                    if act_type == 7 { "Mcn3AM-Mg2_c000_M".to_string() }
                    else  {  concat_string!(ACT_PRE[act_type as usize],gender_str,"-", WEP_PRE[kind as usize],"1_c000_N") }
                }; 
            result.body_anim = Some(body.clone().into());
            find_and_replace_body_animation(result, body, replace.as_str());
        }
        //Mounts
        if is_mounted(anim_set) {
            //println!("Mounted animation: {} for Act_type: {}: {}", Mess::get_name(unit.person.pid), act_type, Mess::get(job.name));
            if mode == 1 {
                result.ride_anim = Some( concat_string!("UAS_", ACT_PRE[act_type as usize], "R").into());
                result.body_anim = Some( concat_string!("UAS_",ACT_PRE[act_type as usize], gender_str).into());
            }
        }
        if check_change_animation_type(job, equipped, gender == 2) {
            if !is_mounted(anim_set) {
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into();
                result.body_anim = None;
            }
            body = if mode == 1 { concat_string!("UAS_", ACT_PRE[act_type as usize], gender_str) }
            else { concat_string!(ACT_PRE[act_type as usize], gender_str, "-", WEP_PRE[kind as usize], "1_c000_N") };
            result.body_anim = Some(body.clone().into());
            if let Some(last_act) = result.body_anims.iter_mut().last() { *last_act = Il2CppString::new_static(body); }
        }
        if mode == 2 && kind != 6 { super::edit_asset_weapon(result, false, equipped); }
    }
    /*
    if unit.job.move_type > 1 {
        println!("Class: {}", Mess::get(unit.job.name));
        result.body_anims.iter().for_each(|str|println!("Mount Body Anims {}", str.to_string()) );
        if let Some(body) = result.body_anim { println!("Mount Body Act: {}", body.to_string()); }
        if let Some(ride) = result.ride_anim { println!("Mount Ride Act: {}", ride.to_string());  }
    }
    */
    if incorrect_mount_animation_fix(unit, result, mode, gender, equipped) { 
      //  result.body_anims.iter().for_each(|str|println!("Mount Body Anims {}", str.to_string()) );
      //  if let Some(body) = result.body_anim { println!("Mount Body Act: {}", body.to_string()); }
      //  if let Some(ride) = result.ride_anim { println!("Mount Ride Act: {}", ride.to_string());  }

        return;
   };
}

pub fn change_hair_change(unit: &Unit, result: &mut AssetTableResult) {
   // if unit.person.get_sp() > 100 { return; }
    let value = unit.grow_seed;
    let index: [usize; 6] = [0, 1, 4, 5, 6, 7];
    let rng = Random::instantiate().unwrap();
    rng.ctor(value as u32);
    for x in index {
        let value2 = rng.value();
        result.unity_colors[x].r = ( value2 & 255 ) as f32 / 255.0;
        result.unity_colors[x].g = (( value2 >> 4 ) & 255 ) as f32 / 255.0;
        result.unity_colors[x].b = (( value2 >> 8 ) & 255 ) as f32 / 255.0;
    }
}

pub fn edit_result_for_monster_trans(result: &mut AssetTableResult, unit: &mut Unit, equipped: Option<&ItemData>, mode: i32) {
    let kind = if equipped.is_none() { 0 } else { equipped.unwrap().kind } as usize;
    let gender = unit_dress_gender(unit);
    let gen = if gender == 1 { "M" } else { "F" };
    let state = unit.get_god_state();
    if state == 2 {
        return;
    }
    if unsafe { !is_null_empty(result.body_model, None) } {
        if result.body_model.to_string().contains("Tik1AT") { return; }
    }
    if (state != 0 && !equipped.is_some_and(|w| w.iid.to_string().contains("チキ"))) && kind != 9 { return; }
    result.ride_model = "null".into();
    result.ride_dress_model = "null".into();
    result.ride_anim = None;
    result.body_anims.clear();

    // println!("Monster Status {}: {}", Mess::get_name(unit.person.pid), unit.status.value);
    if mode == 1 { 
        result.body_anims.add( Il2CppString::new_static(concat_string!("UAS_oBody_A", gen)));
        result.body_anim = Some(concat_string!("UAS_oBody_A", gen).into());
        result.scale_stuff[16] = 2.6;
        change_accessory(result.accessory_list, "null", "l_shld_loc");  // Remove Shield
     }
    else { 
        result.body_anims.add( Il2CppString::new_static(concat_string!("Com0A", gen, "-No1_c000_N")));
        if kind < 10 {
            if kind == 9 { result.body_anims.add( Il2CppString::new_static(concat_string!(INF_ACT[kind], gen, "-Ft1_c000_N"))); }
            else { result.body_anims.add( Il2CppString::new_static(concat_string!(INF_ACT[kind], gen, "-", WEP_PRE[kind], "1_c000_N"))); }
        }
        change_accessory(result.accessory_list, "null", "c_hip_loc");   //Remove Feet Mount
        change_accessory(result.accessory_list, "null", "l_shld_loc");  //Remove Shield
        if gender == 1 { result.body_anims.add( Il2CppString::new_static("Sds0AM-No2_c049_N")); }
        else {  result.body_anims.add( Il2CppString::new_static("Sds0AF-No2_c099_N")); }
    }
    //for x in 0..result.body_anims.len() { println!("Monster Act {}: {}", x, result.body_anims[x]); }
}

pub fn illusion_double_assets(mut result: &mut AssetTableResult, unit: &Unit, mode: i32, equipped: Option<&ItemData>, conditions: &Array<&Il2CppString>) { // Doubles
    result.ride_model = "null".into();
    result.ride_dress_model = "null".into();
    result.ride_anim = None;
    let size;
    let is_tiki;
    let gender = 
        if let Some(owner) = unsafe { get_vision_owner(unit, None) } {
            size = owner.person.get_bmap_size() as i32;
            if owner.person.get_flag().value & 2048 != 0 {
                let generic_mode =  GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY);
                if generic_mode & 1 == 1 && mode == 2{  HEAD_DATA.get().unwrap().replace_by_rng(owner, result); } 
                if generic_mode & 2 == 2 { change_hair_change(owner, result); }
                set_accessories_for_unit(unit, result);
            }
            let is_engage = owner.status.value & 8388608 != 0;
            if is_engage && owner.get_god_unit().is_some_and(|gunit| gunit.data.mid.to_string().contains("Tiki")) || owner.person.name.unwrap().to_string().contains("Tiki") {
                is_tiki = true;
            } 
            else {  is_tiki = false;  }
            unique_class_dress(owner.job, result, unit_dress_gender(owner), mode, owner, is_engage, true);
            unit_dress_gender(owner) 
        }
        else { 
            size = 2;
            is_tiki = false;
            unit_dress_gender(unit) 
        };
    if is_tiki {
        result =  unsafe { super::transform::asset_table_result_setup_person(result, mode, PersonData::get("PID_G001_チキ"), JobData::get("JID_紋章士_チキ"), equipped, conditions, None) };
        if mode == 1 { result.body_anim = Some( Il2CppString::new_static( "UAS_Enb0AF" ));  }
        else {
            result.body_anims.add( Il2CppString::new_static("Com0AF-Sw1_c000_N"));
            result.body_anims.add( Il2CppString::new_static("Com0AF-No1_c000_N"));
            result.body_anims.add( Il2CppString::new_static( "Enb0AF-Sw1_c000_N") );
        }
        return;
    }
    let gender_str = if gender == 2 { "F" } else { "M" };
    if gender != 0 && size == 1 {
        if mode == 1 {
            result.body_anims.iter_mut().for_each(|body| *body = Il2CppString::new_static( concat_string!("UAS_Enb0A", gender_str )) );
            result.body_anim = Some( Il2CppString::new_static( concat_string!("UAS_Enb0A", gender_str )));
            if result.body_anims.len() > 2 { result.body_anims[0] = Il2CppString::new_static( concat_string!("UAS_oBody_A", gender_str )); }
        }
        else {
            result.body_anims.iter_mut().for_each(|body| {
                if body.to_string().contains("-Sw") {
                    let str = unsafe { sub_string(body, 5, None) }.to_string();
                     *body = Il2CppString::new_static( concat_string!("Enb0A", str ));
                }
            });
        }
    }
    else {
        result = unsafe { super::transform::asset_table_result_setup_person(result, mode, PersonData::get("PID_S004_リン"), JobData::get("JID_紋章士_リン"), equipped, conditions, None) };
    }
    if mode == 2 { super::edit_asset_weapon(result, false, equipped); }

}
pub fn dancing_animation(result: &mut AssetTableResult, unit: &Unit, mode: i32) {
    let gender = if result.dress_model.to_string().contains("Tik1AT") { 
        set_accessories_for_unit(unit, result, );
        3 
    } else { unit_dress_gender(unit) };
    result.left_hand = "uWep_Mg00".into();
    result.right_hand = "uWep_Mg00".into();
    result.magic = "RD_Dance".into();
    match (gender, mode) {
        (1, 1) => {
            result.body_anim = Some("UAS_Dnc0AM".into());
            result.body_anims.add( Il2CppString::new_static("UAS_Dnc0AM") );
        }
        (1, 2) => {
            result.dress_model = "uBody_Dnc0AM_c403".into();
            result.body_anim = Some("Dnc0AM-No1_c000_N".into());
            result.body_anims.add( Il2CppString::new_static("Dnc0AM-No1_c000_N") );
        }
        (2, 1) => {
            result.body_anim = Some("UAS_Rod1AF".into());
            result.body_anims.add(Il2CppString::new_static("UAS_Rod1AF"));
        }
        (2, 2) => {
            result.body_anim = Some("Rod1AM-Ft1_c000_N".into());
            result.body_anims.add(Il2CppString::new_static("Rod1AM-Ft1_c000_N"));
        }
        (3, 2) => {
            result.body_anim = Some("Ent0AT-Ft3_c000_N".into());
            result.body_anims.add(Il2CppString::new_static("Ent0AT-Ft3_c000_N"));
        }
        _ => {},
    }
    if gender != 3 { unique_class_dress(unit.job, result, gender, mode, unit, unit.status.value & 8388608 != 0, true); }
}

pub fn remove_mounts_accs(result: &mut AssetTableResult) {
    result.ride_model = "null".into();
    result.ride_dress_model = "null".into();
    result.ride_anim = None;
    change_accessory(result.accessory_list, "null", "c_hip_loc");   //Remove Feet Mount
    change_accessory(result.accessory_list, "null", "l_shld_loc");  //Remove Shield
}

#[skyline::from_offset(0x01bafdd0)]
pub fn add_condition(this: &AssetTableConditionFlags, key: &Il2CppString, method_info: OptionalMethod);

#[skyline::from_offset(0x01bb2750)]
pub fn clear_result(this: &AssetTableResult,method_info: OptionalMethod);

#[skyline::from_offset(0x01bb2a80)]
fn result_commit(this: &AssetTableResult, mode: i32, person: &PersonData, job: &JobData, equipped: Option<&ItemData>, method_info: OptionalMethod);

#[skyline::from_offset(0x01c764b0)]
fn get_vision_owner(this: &Unit, method_info: OptionalMethod) -> Option<&'static Unit>;

#[skyline::from_offset(0x01bb7a60)]
fn get_result_from_item(mode: i32, equipped: Option<&ItemData>, method_info: OptionalMethod) -> &'static AssetTableResult;

