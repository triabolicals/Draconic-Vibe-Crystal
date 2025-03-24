use super::{accessory::*, data::{job::Mount, *}};
use concat_string::concat_string;
use transform::has_enemy_tiki;
use super::*;

pub const MONSTERS: [&str; 8] = ["JID_幻影飛竜", "JID_異形飛竜", "JID_幻影狼", "JID_異形狼",  "JID_E006ラスボス", "JID_幻影竜", "JID_異形竜", "JID_邪竜"];
pub const WEP_PRE: [&str ;10] = ["No", "Sw", "Lc", "Ax", "Bw", "Dg", "Mg", "Rd", "Ft", "No"];
pub const INF_ACT: [&str; 10] = ["Com0A", "Swd0A", "Lnc0A", "Axe0A", "Bow0A", "Dge0A", "Mag0A", "Rod0A", "Rod0A", "Rod0A"];
pub const EATK_ACT: [&str; 23] = ["Mar1A", "Sig1B", "Cel1A", "Mic1A", "Roy1A", "Lei1A", "Luc1A", "Lyn1A", "Ike1A", "Byl1A", "Cor1A", "Eir1A", "Thr1A", "Tik1A", "Hec1A", "Ver1A", "Sor1A", "Cmi1D", "Chr1A", "Ler1A", "Thr2A", "Ver2A", "Ler2A"];
pub static mut LINK_COUNT: i32 = 0;
pub const MAGIC: [&str; 31 ] = ["Fire", "Thunder", "Wind", "Surge", "WpFire", "LFire", "LThunder", "LWind", "LSurge", "Bolganone", "Thoron", "SThoron", "Excalibur", "Micaiah_Thani", "Micaiah_Shine", "Celica_Ragnarok", "Micaiah_Nosferatu", "Obscurite", "Nova", "Meteo", "WpWind", "WpByl", "WpNature", "WpEir", "WpRoy", "WpLight", "WpLer", "WpThunder", "FlameBall", "IceBreath", "IceBall"];
pub const ROD: [&str; 16] = ["Heal", "HiHeal", "FarHeal", "Recover", "WholeHeal", "Rest", "Warp", "ReWarp", "Rescue", "Draw", "TorchRod", "Freeze", "Silence",  "Block", "Collapse", "Dance"];

pub fn fix_common_male_swords() {
    AnimSetDB::get_mut("Com0AM-Sw1_c000_N").map(|anim|{
        anim.atk1 = Some("Swd0AM-Sw1_c000=".into());
        anim.atk2 = Some("Swd0AM-Sw1_c000=".into());
        anim.atk3 = Some("Swd0AM-Sw1_c000=".into());
        anim.atk4 = Some("null".into());
        anim.atk5 = Some("null".into());
        anim.atkc = Some("Swd0AM-Sw1_c000=".into());
        anim.atkt = Some("null".into());
    });
    let seadall = AssetTableStaticFields::get_condition_index("MPID_Seadas");
    let dancer = AssetTableStaticFields::get_condition_index("JID_ダンサー");
    let sf = AssetTableStaticFields::get();
    let male = SEARCH_LIST.get().unwrap().male_index;
    for mode in 1..3 {
        sf.search_lists[mode].iter_mut().filter(|e|
            e.condition_indexes.list.iter().any(|ci| ci.iter().any(|&i| i == dancer) ) && 
            e.condition_indexes.list.iter().any(|ci| ci.iter().any(|&i| i == seadall) ) 
        )
        .for_each(|e|{
            e.condition_indexes.list.iter_mut().flat_map(|ci| ci.iter_mut())
            .find(|i| **i == seadall).map(|i| *i = male ).unwrap()
        });
    }
}


pub fn set_class_animations(result: &mut AssetTableResult, job: &JobData, item: Option<&ItemData>, unit: &Unit, mode: i32, conditions: ConditionFlags) {
    let kind = item.map_or_else(||0, |i| i.get_kind());
    let gen = unit.get_dress_gender();
    let gen_str = if gen == Gender::Male { "M" } else { "F" };
    let is_morph = unit.person.aid.is_some_and(|aid| aid.to_string() == "AID_異形兵");
    if unit.person.gender == 0 || unit.person.get_bmap_size() > 1 || unit.person.parent.index == 0{ return; }
    if unit.status.value & 8388608 != 0 {
        remove_mounts_accs(result);
        if !is_tiki_engage(result) {
            if mode == 2 {
                let style_name = job.style_name.to_string();
                match style_name.as_str() {
                    "気功スタイル" => { result.body_anims.add(Il2CppString::new_static(concat_string!("Enc0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); }  // Qi
                    "魔法スタイル" => { result.body_anims.add(Il2CppString::new_static(concat_string!("Enm0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); }  // Magic
                    "竜族スタイル" => { result.body_anims.add(Il2CppString::new_static(concat_string!("End0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); }  // Dragon
                    "飛行スタイル" => { result.body_anims.add(Il2CppString::new_static(concat_string!("Enw0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); }  // Flying
                    "重装スタイル" => { result.body_anims.add(Il2CppString::new_static(concat_string!("Enh0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); }  // Armor
                    _ => { result.body_anims.add(Il2CppString::new_static(concat_string!("Enb0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); }   // Backup
                }

            }
            else { 
                result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_Enb0A",  gen_str)));
                result.body_anim = Some(concat_string!("UAS_Enb0A", gen_str).into()); 
            }
            if kind == 9 && unit.job.parent.hash == 1398652429 {
                if mode == 2 { result.body_anims.add(Il2CppString::new_static(concat_string!("Enh0A", gen_str, "-Mg2_c000_M"))); }
                else {
                    let body = concat_string!("UAS_Enh2A", gen_str);
                    result.body_anims.add(Il2CppString::new_static(body.as_str()));
                    result.body_anim = Some(body.into());

                }
            }
        }
        return;
    }
    else {
        let job_hash = job.parent.hash; 
        let search_lists = SEARCH_LIST.get().unwrap();
        if conditions.contains(ConditionFlags::Generic) && 
            job.parent.index > 25 && search_lists.job.iter().find(|a| a.job_hash == job_hash && mode == a.mode).is_some_and(|a| !a.unique || a.mound == Mount::Pegasus) {
                // Generic in a Generic Class
                return;
            }
        if let Some(data) = search_lists.job.iter().find(|a| a.job_hash == job_hash && mode == a.mode) {
            println!("Found Class: {} for {} (Unique?: {})", Mess::get_name(job.jid), Mess::get_name(unit.person.pid), data.unique);
            result.body_anims.clear();
            if mode == 1 {
                match data.mound {
                    Mount::None => { result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_oBody_A", gen_str))); }
                    Mount::Cav | Mount::Wolf => { result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_oBody_B", gen_str))); }
                    Mount::Pegasus | Mount::Griffin | Mount::Wyvern => { result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_oBody_F", gen_str))); }
                }
            }
            else {
                result.body_anims.add(Il2CppString::new_static(concat_string!("Com0A", gen_str, "-No1_c000_N"))); 
                result.body_anims.add(Il2CppString::new_static(concat_string!("Com0A", gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); 
                match data.mound {
                    Mount::Cav | Mount::Wolf => {
                        match kind {
                            0|1|2|3|7 => { result.body_anims.add(Il2CppString::new_static(concat_string!("Com0B",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); }
                            _ => {}
                        }
                    }
                    Mount::Wyvern => { result.body_anims.add(Il2CppString::new_static(concat_string!("Wng2D",  gen_str, "-No1_c000_N"))); }
                    _ => {}
                }
            }
            data.get_body_anims(result, kind, gen, is_morph);
            if kind == 9 && mode == 2 {         
                if data.dragon_stone {  //DragonStone
                    result.right_hand = "null".into();
                    if conditions.contains(ConditionFlags::Female) { result.body_anims.add(Il2CppString::new_static("Sds0AF-No2_c099_N"));  }
                    else { result.body_anims.add(Il2CppString::new_static("Sds0AM-No2_c049_N")); }
                }
                else {  // Bullet
                    result.right_hand = "uWep_Mg28".into();
                    result.body_anims.add(Il2CppString::new_static(concat_string!("Mcn3A", gen_str, "-Mg2_c000_M"))); 
                }
                remove_mounts_accs(result);
            }
            if kind == 4 || kind == 6 { change_accessory(result.accessory_list, "null", "l_shld_loc");  }
            if dismount(data.mound, gen, kind) { remove_mounts_accs(result); }

        }
    }
    if conditions.contains(ConditionFlags::Transforming) { edit_result_for_monster_trans(result, unit, item, mode);}
}

pub fn dismount(mount: Mount, gender: Gender, kind: i32) -> bool {
    match mount {
        Mount::None => { false }
        Mount::Cav => {
            match kind {
                5|8|9 => { true }
                _ => { false }
            }
        }
        Mount::Pegasus => {
            match kind {
                4|5|8|9 => { true }
                _ => { false }
            }
        }
        Mount::Wolf => {
            match kind {
                4|6|9 => { true }
                _ => { false }
            }
        }
        Mount::Wyvern => {
            match kind {
                4|5|8|9 => { true }
                6 => { gender == Gender::Male }
                _ => { false }
            }
        }
        Mount::Griffin => {
            match kind {
                4|5|6|8|9 => { true }
                _ => { false }
            }
        }
    }
}

pub fn change_hair_change(unit: &Unit, result: &mut AssetTableResult) {
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

pub fn adjust_engaging_animations(result: &mut AssetTableResult, unit: &Unit) {
    if unit.person.get_asset_force() == 0 {
        result.body_anims.clear();
        if ASSET_STATUS.try_read().unwrap().engaging_count == 0 {
            result.body_anims.add(Il2CppString::new_static( if unit_dress_gender(unit) == 1 { "Tsf0AM-No1_c001_N "} else { "Tsf0AF-No1_c051_N" } ));
            let _ = ASSET_STATUS.try_write().map(|mut lock| lock.engaging_count = 1);
        }
        else {
            result.body_anims.add(Il2CppString::new_static( if unit_dress_gender(unit) == 1 { "Com0AM-No1_c000_N"} else {  "Com0AF-No1_c000_N" } ));
            let _ = ASSET_STATUS.try_write().map(|mut lock| lock.engaging_count = 0);
        }
    }
    else { 
        let _ = ASSET_STATUS.try_write().map(|mut lock| lock.engaging_count = 0);
        emblem::random_engage_voice(result); 
    }
}


pub fn edit_result_for_monster_trans(result: &mut AssetTableResult, unit: &Unit, equipped: Option<&ItemData>, mode: i32) {
    if is_tiki_engage(result) { return; }
    let kind = if equipped.is_none() { 0 } else { equipped.unwrap().kind } as usize;
    let gender = unit_dress_gender(unit);
    let state = unit.get_god_state();
    println!("Unit God State transformation: {}", state);
    if state == 2 {
        if has_enemy_tiki(unit) { replace_body_anim_for_transformation(result, mode, kind, gender); }
    }
    else if state == 0 || equipped.is_some_and(|w| w.iid.to_string().contains("チキ")) || kind == 9 {
        replace_body_anim_for_transformation(result, mode, kind, gender); 
    }
    else if state == 1 && mode == 2 && !is_tiki_engage(result) {
        if gender == 1 { result.body_anims.add( Il2CppString::new_static("End0AM-No2_c049_N")); }
        else { result.body_anims.add( Il2CppString::new_static("End0AF-No2_c099_N")); }
    }
}

pub fn set_dancing_animation(result: &mut AssetTableResult, mode: i32, flags: ConditionFlags) {
    result.left_hand = "uWep_Mg00".into();
    result.right_hand = "uWep_Mg00".into();
    remove_mounts_accs(result);
    result.magic = "RD_Dance".into();
    if flags.contains(ConditionFlags::TikiEngage) && mode == 2 { 
        result.body_anims.add(Il2CppString::new_static("Ent0AT-Ft3_c000_N"));  
    }
    else {
        if flags.contains(ConditionFlags::Male) {
            if mode == 1 {
                result.body_anim = Some("UAS_Dnc0AM".into());
                result.body_anims.add( Il2CppString::new_static("UAS_Dnc0AM") );
            }
            else { result.body_anims.add( Il2CppString::new_static("Dnc0AM-No1_c000_N") );  }
        }
        else if flags.contains(ConditionFlags::Female) {
            if mode == 1 {
                result.body_anim = Some("UAS_Rod1AF".into());
                result.body_anims.add(Il2CppString::new_static("UAS_Rod1AF"));
            }
            else {
                result.body_anims.add(Il2CppString::new_static("Rod1AF-No1_c000_N"));
                result.body_anims.add(Il2CppString::new_static("Rod1AF-Ft1_c000_N"));
            }
        }
    }
}

pub fn remove_mounts_accs(result: &mut AssetTableResult) {
    result.ride_model = "null".into();
    result.ride_dress_model = "null".into();
    result.ride_anim = None;
    change_accessory(result.accessory_list, "null", "c_hip_loc");   //Remove Feet Mount
    change_accessory(result.accessory_list, "null", "l_shld_loc");  //Remove Shield
}

pub fn vision_swd_animations(result: &mut AssetTableResult, gender: Gender, mode: i32) {
    let gen_str = if gender == Gender::Male { "M" } else { "F" };
    if mode == 2 {
        result.body_anims.clear();
        result.body_anims.add(Il2CppString::new_static(concat_string!("Com0A", gen_str, "-No1_c000_N")));
        result.body_anims.add(Il2CppString::new_static(concat_string!("Com0A", gen_str, "-Sw1_c000_N")));
        result.body_anims.add(Il2CppString::new_static(concat_string!("Enb0A", gen_str, "-No1_c000_N")));
        result.body_anims.add(Il2CppString::new_static(concat_string!("Enb0A", gen_str, "-Sw1_c000_N")));
    }
    else {
        result.body_anims.clear();
        result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_oBody_A", gen_str)));
        result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_Enb0A", gen_str)));
        result.body_anim = Some(concat_string!("UAS_Enb0A", gen_str).into());
    }
}

pub fn replace_body_anim_for_transformation(result: &mut AssetTableResult, mode: i32, kind: usize, gender: i32) {
    result.ride_model = "null".into();
    result.ride_dress_model = "null".into();
    result.ride_anim = None;
    result.body_anims.clear();
    let gen = if gender == 1 { "M" } else { "F" };
    change_accessory(result.accessory_list, "null", "c_hip_loc");   //Remove Feet Mount
    change_accessory(result.accessory_list, "null", "l_shld_loc");  //Remove Shield
    if mode == 1 { 
        result.body_anims.add( Il2CppString::new_static(concat_string!("UAS_oBody_A", gen)));
        result.body_anim = Some(concat_string!("UAS_oBody_A", gen).into());
        if kind < 10 {
            if kind == 9 { result.body_anims.add( Il2CppString::new_static(concat_string!(INF_ACT[kind], gen))); }
            else { result.body_anims.add( Il2CppString::new_static(concat_string!(INF_ACT[kind], gen))); }
        }
        result.scale_stuff[16] = 2.6;
        change_accessory(result.accessory_list, "null", "l_shld_loc");  // Remove Shield
    }
    else { 
        result.body_anims.add( Il2CppString::new_static(concat_string!("Com0A", gen, "-No1_c000_N")));
        if kind < 10 {
            if kind == 9 { result.body_anims.add( Il2CppString::new_static(concat_string!(INF_ACT[kind], gen, "-Ft1_c000_N"))); }
            else { result.body_anims.add( Il2CppString::new_static(concat_string!(INF_ACT[kind], gen, "-", WEP_PRE[kind], "1_c000_N"))); }
        }

        if gender == 1 { result.body_anims.add( Il2CppString::new_static("Sds0AM-No2_c049_N")); }
        else {  result.body_anims.add( Il2CppString::new_static("Sds0AF-No2_c099_N")); }
    }
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

// JID_ダンサー; 踊り; 
// MPID_Seadas; JID_ダンサー;