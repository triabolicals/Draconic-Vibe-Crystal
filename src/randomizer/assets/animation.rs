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
const EATK_ACT: [&str; 22] = ["Mar1A", "Sig1B", "Cel1A", "Mic1A", "Roy1A", "Lei1A", "Luc1A", "Lyn1A", "Ike1A", "Byl1A", "Cor1A", "Eir1A", "Thr1A", "Tik1A", "Hec1A", "Ver1A", "Sor1A", "Cmi1D", "Chr1A", "Ler1A", "Thr2A", "Ver2A"];

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
            0|4|6|14 => {   
                result.ride_model = concat_string!("oBody_", job_act, "R_", job_suffix).into(); 
                result.ride_anim = Some(concat_string!("UAS_", ACT_PRE[act_type as usize], "R").into()); 
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
            0|11|14 => { result.ride_dress_model = ride_dress_model.clone().into(); }
            _ => {
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into();
            },
         }
    }



}
pub fn find_animation_set(anim_set: String) -> String {
    if anim_set.contains("Avn1BM-Sw") { return "Avn1BM-Sw1_c100_N".to_string(); }
    if anim_set.contains("Msn1DF-Mg") { return "Msn1DF-Mg1_c553_N".to_string(); }
    let list = AnimSetDB::get_list().unwrap();
    if let Some(act) = list.iter().find(|f| str_contains(f.name, anim_set.as_str())) {
        return act.name.to_string();
    }
    if anim_set.contains("M-No") { return "Com0AM-No1_c000_N".to_string(); }
    else if anim_set.contains("F-No") {  return "Com0AF-No1_c000_N".to_string(); }
    anim_set

}

pub fn is_mounted(act_type: i32) -> bool {
    match act_type {
        0|3|4|5|6|11|12|14 => { true }
        _ => { false}
    }
}

pub fn get_animation_type(job: &JobData) -> i32 {
    let jid = job.jid; 

    if CAV_JIDS.iter().any(|jid| str_contains(job.jid, jid)) && job.is_high() { 
        if str_contains(jid, "JID_ロイヤルナイト") { 14 }   // Royal Knight
        else { 0 }
    }  //Cav
    else if job.jid.contains("エンチャント") { 16 }
    else if job.jid.contains("マージカノン") { 7 } //Mage Cannon
    else if job.jid.contains("JID_メリュジーヌ") { 12 } //Melsuine
    else if ARMOR_JIDS.iter().any(|jid| str_contains(job.jid, jid)) { 1 } //Armor 1
    else if str_contains(jid, "JID_ジェネラル") { 2 } //Armor 2
    else if WYVERN.iter().any(|jid| str_contains(job.jid, jid)) { 3 } // WYVERN
    else if FLIER.iter().any(|jid| str_contains(job.jid, jid)) { 4 } // Flier Pegasus
    else if str_contains(job.jid, "JID_グリフォンナイト") { 5 } // Griffin
    else if str_contains(job.jid, "JID_ウルフナイト") { 6 } //Wolf
    else if str_contains(job.jid, "JID_ダンサー") { 13 }
    else if job.move_type == 1 && job.is_low() { 9 } // Infantry
    else if job.move_type == 1 && job.is_high() { 
        if str_contains(jid, "JID_ブレイブヒーロー") { 15 }
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
        (16, 5|8) => { 33 },
        (14, 2|6) => { 31 },    //Royal Knight Lnc
        (15, 1|2|3) => { 32 },  //Hero Swrd
        (13, _) => { if is_female { 25 } else { 24 } }   //Dancer
        (12, 0) => { if is_female { 23 } else { 3 } }   // Melsuine
        (7, 9) => { 7 },    // Mage Cannon
        (_, 0) => { 
            if act_type == 12 && !is_female { 3 }
            else if act_type == 4 && !is_female { 5 }
            else { act_type }
         }, // XXXG-No1
        (4, 1|2|3) => { //Pegasus
            if is_female { 4 }
            else { 5 }  // Male to Griffin
        }
        (4, 6) => { 
            if is_female { 15  }
            else { 17 }
        }, 
        (0|1|2|3|5|6|7, 1|2|3) => { act_type }, // Cav/Armor/Flyers with Sword/Lance/Axe
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
        (11, 1|2|3) => { 22 }, //Unpromoted Cav
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
pub fn change_unique_class(unit: &Unit, result: &mut AssetTableResult, mode: i32, gender: i32, equipped: Option<&ItemData>, is_emblem: bool) -> bool {
    let uniques = UNIQUE_JOB_DATA.lock().unwrap();
    let job = unit.get_job();
    let kind = if let Some(item) = equipped { item.kind } else { 0 } as i32;
    let gender_str = if gender == 1 { "M" } else { "F" };
    if let Some(ujob) = uniques.iter().find(|unique| str_contains(job.jid, &unique.jid) && gender == unique.gender) {
        //return true; 
        if mode == 1 {
            result.body_model = concat_string!("oBody_", ujob.act_prefix, gender_str, "_", ujob.act_suffix).into();
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
            let new_prefix = if ujob.jid.contains("下級") { ujob.act_prefix.clone().replace("0", "1") }
                else { ujob.act_prefix.clone() };
            if ujob.weapon_mask & (1 << kind) != 0 || kind == 0  {
                let act = concat_string!("UAS_", new_prefix, gender_str);
                result.body_anim = Some(act.clone().into());
                result.body_anims.add(Il2CppString::new_static(act.clone())); 
                if is_emblem { find_and_replace_emblem_animation(result, act)} 
                else { result.body_anims.add(Il2CppString::new_static(act.clone()));  }
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
            if ujob.rig != "none" { result.body_model = ujob.rig.clone().into(); }
            if str_contains(result.dress_model, "Swd") {
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
                    if kind < 4 {
                        result.body_anims.add(Il2CppString::new_static( concat_string!("Com0B", gender_str, "-", WEP_PRE[kind as usize], "_c000") ));
                    }
                }
                _ => {},
            }
            // println!("Mode {} for Unique Job: {} for {}", mode, Mess::get_name(job.jid).to_string(), Mess::get_name(unit.person.pid).to_string());
            if ujob.weapon_mask & (1 << kind) != 0 || kind == 0 {
                let act = if ujob.act_prefix.contains("Msn0") { concat_string!(ujob.act_prefix.replace("0", "1"), gender_str, "-", WEP_PRE[kind as usize]) }
                    else { concat_string!(ujob.act_prefix, gender_str, "-", WEP_PRE[kind as usize]) };
                // Avn1BM-Sw1
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
                if kind == 6 && ( ujob.jid.contains("メリュジーヌ") || ujob.jid.contains("邪竜ノ娘") ) {
                    result.right_hand = "null".into();
                    result.left_hand = "null".into();
                }
                else {equip_weapon(result, equipped); }
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
    else if let Some(ujob) = uniques.iter().find(|unique| str_contains(job.jid, &unique.jid) && gender != unique.gender ) { // In Class Wrong Gender
        let new_act = get_animation_index(job, equipped, false, gender == 2) as usize;
        // println!("New act: {}", new_act);
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
// For non-emblems
fn correct_animations(unit: &Unit, result: &mut AssetTableResult, mode: i32, equipped: Option<&ItemData>) {
    let gender = if unit.edit.is_enabled() {  unit.edit.gender }  else { unsafe { get_dress_gender(unit.person, None)} };
    if gender == 0 || gender > 2 { return; }
    if mode != 1 && mode != 2 { return; }
    let gender_str = if gender == 1 { "M" } else { "F" };
    let job = &unit.job;
    let jid = job.jid.to_string();
    if MONSTERS.iter().any(|&mjob| jid == mjob) {  //Ignore Monster
        return;
    }
    if change_unique_class(unit, result, mode, gender, equipped, false) {
        result.body_anims.iter().for_each(|str|println!("Body Anims {}", str.to_string()) );
        if let Some(body) = result.body_anim { println!("Body Act: {}", body.to_string()); }
        if let Some(ride) = result.ride_anim { println!("Ride Act: {}", ride.to_string());  }
        return;
    }
    if gender == 1 && job.move_type == 1 {  //Male Infantry replace last animation if it's Com0AM to Swd0AM
        if let Some(last) = result.body_anims.iter_mut().last() {
            if last.contains("Com0AM-Sw1") {  *last = Il2CppString::new_static("Swd0AM-Sw1_c000_N"); }
        }
    }
    let kind = if equipped.is_some() { equipped.unwrap().kind } else { 0 } as i32;
    let mut body="NA".to_string();
    let mut replace = "NA".to_string();
    let anim_set = get_animation_type(job);
    let act_type = get_animation_index(job, equipped, false, gender == 2);
    if let Some(name) = unit.person.get_name() {
        let name_list = NAME_DATA.lock().unwrap();
        let person_name = name.to_string();

        if let Some(anim_replace) = name_list.act_replace.iter().find(|x| x.0 == person_name ) {
            //println!("Found for animation: {} for Act_type: {}: {}", Mess::get_name(unit.person.pid), act_type, Mess::get(job.name));
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
        if mode == 2 && kind != 6 { equip_weapon(result, equipped); }
    }
}
fn change_hair_change(unit: &Unit, result: &mut AssetTableResult) {
    let value = unit.grow_ssed;
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

fn edit_result_for_monster_trans(result: &mut AssetTableResult, unit: &mut Unit, equipped: Option<&ItemData>, mode: i32) {
    let kind = if equipped.is_none() { 0 } else { equipped.unwrap().kind } as usize;
    let state = unsafe { unit_god_get_state(unit, None) };
    if unsafe { !is_null_empty(result.dress_model, None) } { //Tiki Engage
        if result.dress_model.contains("Tik1AT") { return;}
    }
    if unsafe { !is_null_empty(result.body_model, None) } {
        if result.body_model.contains("Tik1AT") { return; }
    }
    if state != 0 && kind != 9 { return; }
    result.ride_model = "null".into();
    result.ride_dress_model = "null".into();
    result.ride_anim = None;
    result.body_anims.clear();
    let gender = unit_dress_gender(unit);
    let gen = if gender == 1 { "M" } else { "F" };
    if mode == 1 { 
        result.body_anims.add( Il2CppString::new_static(concat_string!("UAS_oBody_A", gen)));
        result.body_anim = Some(concat_string!("UAS_oBody_A", gen).into());
        result.scale_stuff[16] = 2.6;
     }
    else { 
        result.body_anims.add( Il2CppString::new_static(concat_string!("Com0A", gen, "-No1_c000_N")));
        if kind < 10 {
            result.body_anims.add( Il2CppString::new_static(concat_string!(INF_ACT[kind], gen, "-", WEP_PRE[kind], "1_c000_N")));
        }
        change_accessory(result.accessory_list, "null", "c_hip_loc");   //Remove Feet Mount
        change_accessory(result.accessory_list, "null", "l_shld_loc");  //Remove Shield
        if gender == 1 { result.body_anims.add( Il2CppString::new_static("Sds0AM-No2_c049_N")); }
        else {  result.body_anims.add( Il2CppString::new_static("Sds0AF-No2_c099_N")); }
    }
}


#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook(this: &mut AssetTableResult, mode: i32, unit: &mut Unit, equipped: Option<&ItemData>, conditions: &mut Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let mut result;
    let sequence = engage::gameuserdata::GameUserData::get_sequence();
    let mut job_hash = 0;
    if unit.person.gender > 0 && ( super::transform::is_emblem_class(unit) || super::transform::is_monster_class(unit) ) {
        job_hash = unit.get_job().parent.index;
        if let Some(jid) = unit.person.jid { unit.job = JobData::get_mut(jid).unwrap(); }
        else {unit.job = JobData::get_mut("JID_ソードマスター").unwrap(); }
    }

    // PlayerOutfit Mode
    if ( sequence == 3 || sequence == 2 ) && unit.person.get_asset_force() == 0 {
        if !GameVariableManager::get_bool("G_PlayerOutfit") {
            let mut accessories = [-1; 16];
            for x in 0..unit.accessory_list.unit_accessory_array.len() {
                accessories[x] = unit.accessory_list.unit_accessory_array[x].index;
                unit.accessory_list.unit_accessory_array[x].index = 0;
            }
            result = call_original!(this, mode, unit, equipped, conditions, method_info);
            for x in 0..unit.accessory_list.unit_accessory_array.len() {
                unit.accessory_list.unit_accessory_array[x].index = accessories[x];
            }
        }
        else { result = call_original!(this, mode, unit, equipped, conditions, method_info); }
    }
    else { result = call_original!(this, mode, unit, equipped, conditions, method_info); }

    if job_hash > 0 { 
        unit.job = JobData::try_index_get_mut(job_hash).unwrap(); 
        edit_result_for_monster_trans(this, unit, equipped, mode);
    }

    if unit.person.get_flag().value & 2048 != 0 && mode == 2 {
        add_generic_unit_acc(unit, result.accessory_list);
        let generic_mode =  GameVariableManager::get_number("G_GenericMode");
        if generic_mode != 0 {
            if generic_mode & 1 == 1 {  unsafe { HEAD_DATA.replace_by_rng(unit, result); }  }
            if generic_mode & 2 == 2 { change_hair_change(unit, result); }
        }
        set_accessories_for_unit(unit, result);
    }
    else if GameVariableManager::get_bool("G_Random_Names") && !unit.person.pid.contains(PIDS[0]) {
        let name = unit.person.get_name().unwrap();
        if let Some(emblem_index) = RINGS.iter().position(|x| name.contains(x) ){
            let index = if emblem_index > 18 { emblem_index + 1 }
                else { emblem_index };
            if unsafe { EMBLEM_NAMES[index] } != -1 {
                let emblem_person = PersonData::get(PIDS[ unsafe { EMBLEM_NAMES[index] } as usize ]);
                result = super::transform::asset_table_result_setup_person_hook(result, mode, emblem_person, emblem_person.unwrap().get_job(), equipped, conditions, method_info);
                clear_accessory_from_list(result.accessory_list, "Eff_EmblemAura");
            }
        }
    }
    if CONFIG.lock().unwrap().misc_option_1 >= 4.75 && (unit.person.gender == 1 || unit.person.gender == 2 ){   // Bust Rando using Fortune Seed
        if unit.fortune_seed == 0 {
            let seed = unit.drop_seed as u32;
            let rng = Random::instantiate().unwrap();
            rng.ctor(seed);
            unit.fortune_seed = rng.value(); 
        }
        let rng = Random::instantiate().unwrap();
        rng.ctor(unit.fortune_seed as u32);
        result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.03;
    }
    // Dance Command
    
    if conditions.iter().any(|con| con.to_string() == "踊り") {
        // println!("Dance Check");
        let gender = if result.dress_model.contains("Tik1AT") { 
            set_accessories_for_unit(unit, result, );
            3 
        } else { unit_dress_gender(unit) };
        //Ent0AT-Ft3_c000_N
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
        return result;
    }
    let state = unsafe { unit_god_get_state(unit, None) };
    // Doubles
    if unsafe { !is_null_empty(result.dress_model, None) } && state != 2 { //Tiki Engage
        if result.dress_model.contains("uBody_Tik1AT") { return result;  }
    }

    if unit.person.pid.contains("_残像") {
        result.ride_model = "null".into();
        result.ride_dress_model = "null".into();
        result.ride_anim = None;
        let gender = if let Some(owner) = unsafe { get_vision_owner(unit, None) } { unit_dress_gender(owner) }
            else { unit_dress_gender(unit) };

        let gender_str = if gender == 2 { "F" } else { "M" };
        if gender != 0 && gender < 3 {
            if mode == 1 {
                result.body_anims.iter_mut().for_each(|body| *body = Il2CppString::new_static( concat_string!("UAS_Enb0A", gender_str )) );
                result.body_anim = Some( Il2CppString::new_static( concat_string!("UAS_Enb0A", gender_str )));
                if result.body_anims.len() > 2 { result.body_anims[0] = Il2CppString::new_static( concat_string!("UAS_oBody_A", gender_str )); }
            }
            else {
                result.body_anims.iter_mut().for_each(|body| {
                    if body.contains("-Sw") {
                        let str = unsafe { sub_string(body, 5, None) }.to_string();
                        *body = Il2CppString::new_static( concat_string!("Enb0A", str ));
                    }
                });
            }
        }
        return result;
    }

    if state == 0 { //Not Engage
        if let Some(name) = unit.person.get_name() {  
            if job_hash == 0 {
                if name.contains("Sfoglia") && mode == 1 { result.body_model = "oBody_Wng2DF_c000".into(); }
                else if name.contains("MPID_Eve") && mode == 1 { result.body_model = "oBody_Rod2AF_c000".into(); } 
                if unit.person.pid.contains("M022_紋章士") && GameVariableManager::get_number("G_Emblem_Mode") != 0 {
                    if let Some(emblem_index) = EMBLEM_ASSET.iter().position(|x| unit.person.pid.contains(x)) {
                        let gid = GameVariableManager::get_string(format!("G_R_GID_{}", EMBLEM_ASSET[emblem_index]).as_str());
                        if let Some(emblem_index2) = EMBLEM_ASSET.iter().position(|x| gid.contains(x)){
                            if emblem_index2 >= 12 {    // DLC Dark Emblems
                                let gid2 = format!("GID_E006_敵{}", EMBLEM_ASSET[emblem_index2]);
                                return asset_table_result_god_setup(this, mode+10, GodData::get(gid2), true, conditions, method_info);
                            }
                            else {  // Base game Dark Emblems
                                return asset_table_result_god_setup(this, mode+10, GodData::get(gid), true, conditions, method_info);
                            }
                        }
                    }
                }
                if !result.body_anims.iter().any(|body| body.contains("Ent0AT")) {
                    let name_str = name.to_string();
                    if let Some(pos) = RINGS.iter().position(|ring| name_str == concat_string!("MPID_", ring) || name_str == concat_string!("MGID_", ring)){
                        correct_emblem_animations(unit, result, mode, equipped);
                    }
                    else { correct_animations(unit, result, mode, equipped);  }
                    set_accessories_for_unit(unit, result);
                }
            }
            //Re-classing or //Starting Engaging
            if conditions.iter().any(|con| str_contains(con, "クラスチェンジ中") || str_contains(con, "エンゲージ開始")) {
                result.ride_model = "null".into();
                result.ride_dress_model = "null".into();
                result.ride_anim = None;
            }
            else {
                let kind = if equipped.is_some() { equipped.unwrap().kind } else { 0 };
                let job = unit.get_job();
                if kind == 6 && ( job.jid.contains("メリュジーヌ") || job.jid.contains("邪竜ノ娘") ) {
                    result.right_hand = "null".into();
                    result.left_hand = "null".into();
                }
                else { equip_weapon(result, equipped); }
            }
            return result;
        }
    }
    if GameVariableManager::get_number("G_Random_God_Mode") & 2 == 0 || !crate::utils::can_rand() { return result; }
    if state == 2  { // Engage Attack Correction
        let engage_attack = unsafe { get_engage_attack(unit, None) };
        if engage_attack.is_none() { return result; } 
        let list = unsafe { get_body_anims(result, None) };
        let sid = engage_attack.unwrap().sid;
        let mut emblem_index = 50;
        let mut animation_index = 50;
        let mut old_engage = 50;
        // Finding the Engage Animation Index in Body Animations
        for x in 0..list.len() {
            println!("Engage Act {}: {}", x, list[x]);
            if let Some(animation) = EATK_ACT.iter().position(|&y| list[x].contains(y)) {
                animation_index = x;
                old_engage = animation;
                break;
            }
        }
        // Get New Engage Attack
        emblem_index = if let Some(pos) = EMBLEM_ASSET.iter().position(|god| sid.contains(god)) { pos }
            else if sid.contains("三級長エンゲージ技＋") { 20 }
            else if sid.contains("三級長エンゲージ") { 12 }
            else { 50 };

        //println!("Found Engage Attack {} and Old Emblem: {}", emblem_index, old_engage);
        if animation_index == 50 && list.len() > 0 { animation_index = list.len() -1; }
        if emblem_index == 50 { return result; }
        if emblem_index == old_engage { return result; }
        let gender = unit_dress_gender(unit);
        let mut gender_str = if gender == 2 { "F" } else { "M" };

        if mode == 1 {
            if old_engage == 13 {
                result.body_model = "oBody_Tik0AF_c560".into();
                result.hair_model = "oHair_h560".into();
            }
            let act = if emblem_index == 12 || emblem_index == 20 {  concat_string!("UAS_Mar1A", gender_str) } 
                else if emblem_index == 13 { concat_string!("UAS_Mic1A", gender_str) }
                else { concat_string!("UAS_", ENGAGE_PREFIX[old_engage], ",1A", gender_str) };
            
            if list.len() <= animation_index { list.add(Il2CppString::new_static(act.clone()));  }
            else { list[animation_index] = Il2CppString::new_static(act.clone()); }
            result.body_anim = Some(act.clone().into());
            return result;
        }
        let mpid = unit.person.get_name().unwrap().to_string();
        result.ride_model = "null".into();
        result.ride_dress_model = "null".into();


        let mut accessory_list = unsafe { asset_table_result_accessory_list(result, None) };
        // Remove Accessories
        match old_engage {
            5 => {  //Leif  Remove Weapons
                result.left_hand = "null".into();
                result.right_hand = "null".into();
            }
            7|10 => {  // Lyn Remove Bow, Corrin remove Dragon Fang
                result.right_hand = "null".into();
            },
            12 => { // Edelgard Remove Weapons
                result.left_hand = "null".into();
                result.right_hand = "null".into();
                clear_accessory_from_list(accessory_list, "uWep_Ax20");
                clear_accessory_from_list(accessory_list, "uWep_Lc21");
                clear_accessory_from_list(accessory_list, "uWep_Bw16s-Bw");
            }
            13 => { //Change Dragon Tiki to Human Tiki for all animations not Divine Blessing
                if emblem_index != 13 {
                    result.dress_model = "uBody_Tik0AF_c560".into();
                    result.body_model = "uRig_GodF1".into();
                    result.head_model = "uHead_c560".into();
                    result.hair_model = "uHair_null".into();
                    add_accessory_to_list(accessory_list, "uAcc_spine2_Hair560", "c_spine1_jnt");
                    add_accessory_to_list(accessory_list, "uAcc_head_Tiara560", "c_head_loc");
                    gender_str = "F";
                }
            }
            15 => { clear_accessory_from_list(accessory_list, "uAcc_Event_SummonStone"); }
            21 => { clear_accessory_from_list(accessory_list, "uAcc_Event_SummonStoneB"); }
            _ => {}
        }
        let new_engage_animation: String;
        match emblem_index {    //Marth, Roy, Leif, Lucina, Ike, Byleth, Dragon Blast
            0|4|5|6|8|9|19 => { 
                new_engage_animation = concat_string!( ENGAGE_PREFIX[ emblem_index as usize], "1A", gender_str,"-Sw1_c000_N"); 
                if emblem_index == 5 {
                    result.right_hand = "uWep_Sw42R".into();
                    result.left_hand = "uWep_Sw42L".into();
                    result.trail = "cEff_EngageA_Swd_00".into();
                }
                else { equip_weapon(result, equipped); }
            }
            2|15|16|21 => { 
                new_engage_animation = concat_string!(ENGAGE_PREFIX[ emblem_index as usize], "1A",gender_str,"-Mg1_c000_N");  //Celica / Veronica / Soren 
                if emblem_index == 2 { result.right_hand = "uWep_Mg00".into(); }  
                if emblem_index == 15 {  add_accessory_to_list(accessory_list, "uAcc_Event_SummonStone", "reserve4_loc");  }
                if emblem_index == 21 {  add_accessory_to_list(accessory_list, "uAcc_Event_SummonStoneB", "reserve4_loc");  }
            }
            1 => {  //Sigurd
                result.ride_model = "uRig_HorsR".into();
                result.ride_dress_model = "uBody_Sig0BR_c531".into();
                new_engage_animation = concat_string!("Sig1B", gender_str, "-Sw1_c000_N");
                equip_weapon(result, equipped); 
            }
            3|13 => {  //Micaiah and Tiki
                if ( mpid == "MPID_Lueur" && gender == 1 ) || mpid == "MPID_Boucheron" || mpid == "MPID_Bonet" || mpid == "MPID_Vandre" || mpid == "MPID_Mauve" {
                    new_engage_animation = "Mic1AM-Mg1_c001_N".to_string();
                }
                else if mpid == "MPID_Jean" || mpid == "MPID_Staluke" || mpid == "MPID_Clan" { new_engage_animation = "Mic1AM-Mg1_c501_N".to_string(); }
                else if mpid == "MPID_Saphir" { new_engage_animation = "Mic1AF-Mg1_c254_N".to_string(); }
                else { new_engage_animation = concat_string!("Mic1A",gender_str, "-Mg1_c000_N" ); }
            }
            7 => { 
                new_engage_animation = concat_string!("Lyn1A", gender_str, "-Bw1_c000_L");
                result.right_hand = "uWep_Ft00".into();
            }   // Lyn 
            10 => { 
                new_engage_animation = concat_string!("Cor1A",  gender_str, "-Ft1_c000_N");
                result.left_hand = "uWep_Ft02".into();
            }  // Corrin
            11 => {
                if mpid == "MPID_Jean" { new_engage_animation = "Eir1AM-Sw1_c103_N".to_string();  }
                else if mpid == "MPID_Anna" { new_engage_animation = "Eir1AF-Sw1_c552_N".to_string(); }
                else { new_engage_animation = concat_string!("Eir1A",gender_str, "-Sw1_c000_N"); }
                equip_weapon(result, equipped); 
            }   // Eirika
            12 => { 
                new_engage_animation = concat_string!("Thr1A",  gender_str, "-Ax1_c000_N");
                result.left_hand = "uWep_Ax30L".into();
                result.right_hand = "uWep_Ax30R".into();
                result.trail = "cEff_EngageA_Swd_00".into();
                add_accessory_to_list(accessory_list, "reserve1_loc", "uWep_Ax20");
                add_accessory_to_list(accessory_list, "reserve2_loc", "uWep_Lc21");
                add_accessory_to_list(accessory_list, "reserve3_loc", "uWep_Bw16s-Bw");
            } //Houses Unite   //Tiki
            14 => { new_engage_animation = concat_string!("Hec1A", gender_str, "-Ax1_c000_N"); equip_weapon(result, equipped); } // Hector
            17 => { //Camilla 
                result.ride_model = "uRig_DragR".into();
                result.ride_dress_model = "uBody_Cmi0DR_c561".into(); 
                new_engage_animation = concat_string!("Cmi1D", gender_str,"-Ax1_c000_N");
                equip_weapon(result, equipped);
            }
            18 => {     //Chrom
                if mpid == "MPID_Jean" {  new_engage_animation = "Chr1AM-Sw1_c103_N".to_string(); }
                else if mpid == "MPID_Mauve" || mpid == "MPID_Boucheron" { new_engage_animation = "Chr1AM-Sw1_c502_N".to_string(); }
                else if mpid == "MPID_Anna" { new_engage_animation = "Chr1AF-Sw1_c552_N".to_string();; }
                else if ( mpid == "MPID_Lueur" && gender == 2 )  || mpid == "MPID_Chloe" || mpid == "MPID_Jade" || mpid == "MPID_Ivy" || mpid == "MPID_Merin" || mpid == "MPID_Saphir" {
                    new_engage_animation = "Chr1AF-Sw1_c254_N".to_string();
                }
                else if mpid == "MPID_El" || mpid == "MPID_Selestia" { new_engage_animation = "Chr1AF-Sw1_c254_N".to_string(); }
                else { new_engage_animation = concat_string!("Chr1A", gender_str, "-Sw1_c000_N"); }
                equip_weapon(result, equipped);
            }
            20 => { new_engage_animation = concat_string!("Thr2A", gender_str, "-Ax1_c000_N"); }  // Houses Unite+
            _ => { return result; }
        }
    // overwrite old engage attack animation
        for x in animation_index..list.len() { list[x as usize] = Il2CppString::new_static(new_engage_animation.clone()); }
        if list.len() <= animation_index {
            if let Some(last) = list.iter_mut().last() {
                *last = Il2CppString::new_static(new_engage_animation);
            }
        }
    }
    result
}
//Adding Accessories to AssetTable Result's AssetTableAccessoryList 

pub fn equip_weapon(result: &mut AssetTableResult, item: Option<&ItemData>) {
    if let Some(w_item) = item {
        let index = w_item.parent.index; 
        let weapons = WEAPON_ASSET.lock().unwrap();

        if let Some(weapon) = weapons.iter().find(|&x| x.iid_index == index) {
            if weapon.right_hand != "none" {  result.right_hand = weapon.right_hand.clone().into(); }
            if weapon.left_hand != "none" { result.left_hand = weapon.left_hand.clone().into(); }
        }
    }
}


#[skyline::from_offset(0x01bb2a80)]
fn result_commit(this: &AssetTableResult, mode: i32, person: &PersonData, job: &JobData, equipped: Option<&ItemData>, method_info: OptionalMethod);

#[skyline::from_offset(0x01c764b0)]
fn get_vision_owner(this: &Unit, method_info: OptionalMethod) -> Option<&'static Unit>;

#[skyline::from_offset(0x01bb7a60)]
fn get_result_from_item(mode: i32, equipped: Option<&ItemData>, method_info: OptionalMethod) -> &'static AssetTableResult;

