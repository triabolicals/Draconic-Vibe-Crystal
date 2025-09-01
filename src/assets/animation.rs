use super::{accessory::*, data::{job::Mount, *}};
use concat_string::concat_string;
use transform::has_enemy_tiki;
use engage::mapmind::MapMind;
use super::*;

pub const MONSTERS: [&str; 8] = ["JID_幻影飛竜", "JID_異形飛竜", "JID_幻影狼", "JID_異形狼",  "JID_E006ラスボス", "JID_幻影竜", "JID_異形竜", "JID_邪竜"];
pub const WEP_PRE: [&str ;10] = ["No", "Sw", "Lc", "Ax", "Bw", "Dg", "Mg", "Rd", "Ft", "No"];
pub const INF_ACT: [&str; 10] = ["Com0A", "Swd0A", "Lnc0A", "Axe0A", "Bow0A", "Dge0A", "Mag0A", "Rod0A", "Rod0A", "Rod0A"];
pub const EATK_ACT: [&str; 23] = ["Mar1A", "Sig1B", "Cel1A", "Mic1A", "Roy1A", "Lei1A", "Luc1A", "Lyn1A", "Ike1A", "Byl1A", "Cor1A", "Eir1A", "Thr1A", "Tik1A", "Hec1A", "Ver1A", "Sor1A", "Cmi1D", "Chr1A", "Ler1A", "Thr2A", "Ver2A", "Ler2A"];
pub static mut LINK_COUNT: i32 = 0;
pub const MAGIC: [&str; 31 ] = ["Fire", "Thunder", "Wind", "Surge", "WpFire", "LFire", "LThunder", "LWind", "LSurge", "Bolganone", "Thoron", "SThoron", "Excalibur", "Micaiah_Thani", "Micaiah_Shine", "Celica_Ragnarok", "Micaiah_Nosferatu", "Obscurite", "Nova", "Meteo", "WpWind", "WpByl", "WpNature", "WpEir", "WpRoy", "WpLight", "WpLer", "WpThunder", "FlameBall", "IceBreath", "IceBall"];
pub const ROD: [&str; 16] = ["Heal", "HiHeal", "FarHeal", "Recover", "WholeHeal", "Rest", "Warp", "ReWarp", "Rescue", "Draw", "TorchRod", "Freeze", "Silence",  "Block", "Collapse", "Dance"];

pub fn gen_str(gender: Gender) -> &'static str {
    if gender == Gender::Male { "M" } else { "F" }
}
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


pub fn set_class_animations(
    result: &mut AssetTableResult,
    job: &JobData,
    item: Option<&ItemData>,
    unit: &Unit,
    mode: i32,
    conditions: ConditionFlags)
{
    if conditions.contains(ConditionFlags::Dance) {
        set_dancing_animation(result, mode, conditions);
        return;
    }
    let kind = item.map_or_else(||0, |i| i.get_kind());

    let gen = if conditions.contains(ConditionFlags::Male) { Gender::Male } else { Gender::Female };
    let gen_str = if conditions.contains(ConditionFlags::Male) { "M" } else { "F" };
    let is_morph = unit.person.aid.is_some_and(|aid| aid.str_contains("AID_異形兵"));
    if unit.person.gender == 0 || unit.person.get_bmap_size() > 1 || unit.person.parent.index == 0 { return; }
    if (unit.status.value & 8388608 != 0 || conditions.contains(ConditionFlags::Engaged)) &&
        ( conditions.contains(ConditionFlags::Male) ||  conditions.contains(ConditionFlags::Female) )
    {
        remove_mounts_accs(result);
        if !is_tiki_engage(result) {
            if mode == 2 { result.body_anims.add(engaged_weapon_animation(unit, kind, gen)); }
            else { 
                result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_Enb0A",  gen_str)));
                result.body_anim = Some(concat_string!("UAS_Enb0A", gen_str).into()); 
            }
            if kind == 9 {
                if SEARCH_LIST.get().unwrap().job_can_use_canon(job) {
                    if mode == 2 {
                        result.body_anims.add(Il2CppString::new_static(concat_string!("Enh0A", gen_str, "-Mg2_c000_M"))); 
                    }
                    else {
                        let body = concat_string!("UAS_Enh2A", gen_str);
                        result.body_anims.add(Il2CppString::new_static(body.as_str()));
                        result.body_anim = Some(body.into());
                    }
                }
                else if SEARCH_LIST.get().unwrap().job_can_use_dragonstone(job) {
                    if mode == 2 {
                        result.right_hand = "null".into();
                        if conditions.contains(ConditionFlags::Male)  {
                            result.body_anims.add(Il2CppString::new_static("End0AM-No2_c049_N"));
                        }
                        else {
                            result.body_anims.add(Il2CppString::new_static("End0AF-No2_c099_N"));
                        }
                    }
                }
            }
        }
    }
    else {
        let job_hash = job.parent.hash;
        let search_lists = SEARCH_LIST.get().unwrap();
        /*
        if let Some(data) = search_lists.job.iter()
            .find(|a|
                a.job_hash == job_hash && mode == a.mode &&
                    conditions.contains(ConditionFlags::Generic) &&
                    job.parent.index > 25 &&
                    !a.unique
            )
        {
            if data.mount != Mount::None && !has_mount_weapon_anim(result, data.mount, gen, kind) {
                let new_anim = create_anim_from_mount(data.mount, gen, kind);
                result.body_anims.add(Il2CppString::new_static(new_anim));
            }
            return;
        }
        */

        if let Some(data) = search_lists.job.iter().find(|a| a.job_hash == job_hash && mode == a.mode) {
            result.body_anims.clear();
            // println!("Mount: {}", data.mount.get_ride_race());
            let male_pegasus = gen == Gender::Male && data.mount == Mount::Pegasus;
            let mount = if male_pegasus { Mount::Wyvern } else { data.mount };
            add_standard_for_mount_type(result, kind, gen, mode, mount, is_morph);
            if male_pegasus {
                if mode == 1 { result.ride_anim = Some("UAS_Wng2DR".into()); }
                else { result.body_anims.add(create_anim_from_mount(mount, gen, kind).into()); }
            }
            else { data.get_body_anims(result, kind, gen, is_morph); }
            if kind == 9 {
                let g = if conditions.contains(ConditionFlags::Female) { Gender::Female } else { Gender::Male };
                special_item_type_anim(result, g, mode, data.cannon);
            }
            if kind == 4 || kind == 6 { change_accessory(result.accessory_list, "null", "l_shld_loc");  }
            else if !conditions.intersects(ConditionFlags::NoShield) && data.cannon {
                add_accessory_to_list(result.accessory_list, "uAcc_shield_Mcn3AM", "l_shld_loc");
            }
            correct_mis_gender_animations(result, kind, gen, data.mount, mode, data.cannon, conditions);
        }
    }   
    if conditions.contains(ConditionFlags::Transforming) { edit_result_for_monster_trans(result, unit, item, mode);}
}
pub fn special_item_type_anim(result: &mut AssetTableResult, gender: Gender, mode: i32, is_canon: bool) {
    remove_mounts_accs(result);
    if !is_canon && mode == 2  {  //DragonStone
        result.right_hand = "null".into();
        if gender == Gender::Female { result.body_anims.add(Il2CppString::new_static("Sds0AF-No2_c099_N"));  }
        else { result.body_anims.add(Il2CppString::new_static("Sds0AM-No2_c049_N")); }
    }
    else {
        if mode == 2 {  // Bullet
            result.right_hand = "uWep_Mg28".into();
            result.body_anims.add(Il2CppString::new_static(concat_string!("Mcn3A", gen_str(gender), "-Mg2_c000_M")));
        }
        else {
            result.right_hand = "oWep_Mg28".into();
            result.body_anim = Some(concat_string!("UAS_Mcn3A", gen_str(gender)).into());
        }
    }
}
pub fn create_anim_from_mount(mount: Mount, gender: Gender, item_kind: i32) -> String {
    let gen = if gender == Gender::Male { "M" } else { "F" };
    let anim_type = data::job::create_anim_type(mount, gender);
    match mount {
        Mount::Griffin => {
            match item_kind {
                1|2|3|7 => { concat_string!("Wng1F", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
                4|5 => { concat_string!("Wng1F", gen, "-Ln1_c000_M") }
                6 => { concat_string!("Wng1F", gen, "-Sw1_c000_M") }
                8|9 => { concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
                _ => { concat_string!("Wng1F", gen, "-No1_c000_N") }
            }
        }
        Mount::Cav => { 
            match item_kind {
                1|2|3 => { concat_string!("Cav0B", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
                4 => { concat_string!("Bow2B", gen, "-", WEP_PRE[item_kind as usize], "1_c000_L") }
                6 => { concat_string!("Mag2B", gen, "-Mg1_c000_M") }
                7 => { concat_string!("Com0B", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
                5 => { concat_string!("Cav0B", gen, "-Ln1_c000_M") }
                8|9 => { concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
                _ => {  concat_string!("Com0B", gen, "-No1_c000_N") }
            }
        }
        Mount::Pegasus => { 
            match item_kind {
                1|2|3|7 => { concat_string!("Wng0EF-", WEP_PRE[item_kind as usize], "1_c000_N") }
                6 => { String::from("Slp0EF-Mg1_c351_M") }
                5 => {  String::from("Wng0EF-Ln1_c000_M") }
                4|8|9 => { concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_", if item_kind ==  4 { "L" } else { "N"}) }
                _ => {String::from("Wng0EF-No1_c000_N") }
            }
        }
        Mount::Wolf => { 
            match item_kind {
                1|2|3|5|7 => { concat_string!("Cav2C", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
                4 => { concat_string!("Cav2C", gen, "-", "Ln1_c000_M") }
                6 => { concat_string!("Cav2C", gen, "-Sw1_c000_M") }
                8|9 =>  { concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_", if item_kind ==  4 { "L" } else { "N"}) }
                _ => {  concat_string!("Com0B", gen, "-No1_c000_N") }
            }
        }
        Mount::Wyvern => { 
            match item_kind {
                1|2|3|7 => { concat_string!("Wng2D", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
                4|5 => { concat_string!("Wng2D", gen, "-Ln1_c000_M") }
                6 => {
                    if anim_type == "DF" { String::from("Cmi0DF-Mg1_c561_M") }
                    else {  String::from("Wng2DM-Sw1_c000_M") }
                }
                8|9 => { concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
                _ => { concat_string!("Wng2D", gen, "-No1_c000_N") }
            }
        }
        _ => { concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_N") }
    }
}


pub fn has_mount_weapon_anim(result: &mut AssetTableResult, mount: Mount, gender: Gender, kind: i32) -> bool {
    let act_type = data::job::create_anim_type(mount, gender);
    let anim_type = concat_string!(act_type, "-", WEP_PRE[kind as usize]);
    result.body_anims.iter().any(|x| x.to_string().contains(anim_type.as_str()))
}


pub fn adjust_engaging_animations(result: &mut AssetTableResult, unit: &Unit) {
    if unit.person.get_asset_force() == 0 {
        if MapMind::get_target_unit().is_some_and(|target| unit.person.parent.index == target.person.parent.index) {
            result.body_anims.clear();
            result.body_anims.add(Il2CppString::new_static( if unit_dress_gender(unit) == 1 { "Com0AM-No1_c000_N"} else {  "Com0AF-No1_c000_N" } ));
        }
        else if MapMind::get_target_unit().is_some_and(|target| unit.person.parent.index != target.person.parent.index) {
            let gen = unit_dress_gender(unit);
            result.body_anims.add(Il2CppString::new_static( if gen == 1 { "Tsf0AM-No1_c001_N"} else { "Tsf0AF-No1_c051_N" } ));
        }
        else {
            result.body_anims.clear();
            result.body_anims.add(Il2CppString::new_static( if unit_dress_gender(unit) == 1 { "Com0AM-No1_c000_N"} else {  "Com0AF-No1_c000_N" } ));
        }
    }
    else {  emblem::random_engage_voice(result);   }
}


pub fn edit_result_for_monster_trans(result: &mut AssetTableResult, unit: &Unit, equipped: Option<&ItemData>, mode: i32) {
    if is_tiki_engage(result) { return; }
    let kind = if equipped.is_none() { 0 } else { equipped.unwrap().kind } as usize;
    let gender = unit_dress_gender(unit);
    let state = unit.get_god_state();
    if state == 2 {
        if has_enemy_tiki(unit) { replace_body_anim_for_transformation(result, mode, kind, gender); }
    }
    else if state == 0 || equipped.is_some_and(|w| w.iid.str_contains("チキ")) || kind == 9 {
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

pub fn lueur_engage_atk(result: &mut AssetTableResult, unit: &Unit, flags: ConditionFlags) {
    let mut gen_str = if flags.contains(ConditionFlags::Male) { "M" } else { "F"};
    remove_mounts_accs(result);
    if is_tiki_engage(result) {
        SEARCH_LIST.get().unwrap().replace_with_god(result, 2, 13, false);
        gen_str = "F";
    }
    if let Some(god) = unit.god_link.or(unit.god_unit) {
        result.body_anims.clear();
        if god.child.is_none() { result.body_anims.add(Il2CppString::new_static(concat_string!("Ler1A",gen_str,"-Sw1_c000_N"))); }
        else if flags.contains(ConditionFlags::EngageAttackComboMain){ result.body_anims.add(Il2CppString::new_static(concat_string!("Ler2A", gen_str,"-Sw1_c000_N"))); }
        else { result.body_anims.add(Il2CppString::new_static(concat_string!("Ler2A", gen_str,"-Sw1_p000_N")));  }
    }
}

pub fn anim_exists(body: &str) -> bool {
    let search = body.split_at(9).0;
    AnimSetDB::get_list().unwrap().iter().find(|x| x.name.str_contains(search)).is_some()
}

pub fn engaged_weapon_animation(unit: &Unit, kind: i32, gender: Gender) -> &'static Il2CppString {
    let gen_str = if gender == Gender::Male { "M" } else { "F" };
    match unit.job.style {
        7 => { Il2CppString::new_static(concat_string!("Enc0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N")) }  // Qi
        6 => { Il2CppString::new_static(concat_string!("Enm0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N")) }  // Magic
        8 => { Il2CppString::new_static(concat_string!("End0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N")) }  // Dragon
        5 => { Il2CppString::new_static(concat_string!("Enw0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N")) }  // Flying
        4 => { Il2CppString::new_static(concat_string!("Enh0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N")) }  // Armor
        _ => { Il2CppString::new_static(concat_string!("Enb0A",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N")) }
    }
}
pub fn add_standard_for_mount_type(result: &mut AssetTableResult, kind: i32, gender: Gender, mode: i32, mount: Mount, is_morph: bool) {
    let gen_str = gen_str(gender);
    let mut suffix = if is_morph { "c707" } else { "c000" };
    let prefix = if mode == 1 { "oBody" } else { "uBody"};
    let ride_type = mount.get_ride_race();
    let mut ride_asset: Option<&'static str>;
    if mode == 1 {

        if result.ride_anim.is_none() && mount != Mount::None {
            let mount_gen_type = data::job::create_anim_type(mount, gender);
            if let Some(mount_act) = result.body_anims.iter().find(|x| x.to_string().contains(&mount_gen_type)) {
                let s = mount_act.to_string();
                result.ride_anim = Some(s.replace(&mount_gen_type, ride_type).into());
            }
        }
        result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_oBody_A", gen_str)));
        match mount {
            Mount::Cav | Mount::Wolf => { result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_oBody_B", gen_str))); }
            Mount::Griffin | Mount::Wyvern => { result.body_anims.add(Il2CppString::new_static(concat_string!("UAS_oBody_F", gen_str))); }
            Mount::Pegasus => { result.body_anims.add(Il2CppString::new_static("UAS_oBody_FF")); }
            _ => {}
        }
    }
    else {
        result.body_anims.add(Il2CppString::new_static(concat_string!("Com0A", gen_str, "-No1_c000_N")));
        result.body_anims.add(Il2CppString::new_static(concat_string!("Com0A", gen_str, "-", WEP_PRE[kind as usize], "1_c000_N")));
        match mount {
            Mount::Cav | Mount::Wolf => {
                if mount == Mount::Cav { result.ride_model = "uRig_HorsR".into(); }
                else { result.ride_model = "uRig_WolfR".into(); }
                match kind {
                    0|1|2|3|7 => { result.body_anims.add(Il2CppString::new_static(concat_string!("Com0B",  gen_str, "-", WEP_PRE[kind as usize], "1_c000_N"))); }
                    _ => {}
                }
            }
            Mount::Wyvern => {
                result.ride_model = "uRig_DragR".into();
                result.body_anims.add(Il2CppString::new_static(concat_string!("Wng2D",  gen_str, "-No1_c000_N")));
            }
            Mount::Pegasus => {
                result.ride_model = "uRig_PegaR".into();
                result.body_anims.add("Wng0EF-No1_c000_N".into());
            }
            Mount::Griffin => {
                result.ride_model = "uRig_GrifR".into();
                result.body_anims.add(Il2CppString::new_static(concat_string!("Wng1F", gen_str, "-No1_c000_N")));
            }
            _ => { result.ride_model = "".into(); }
        }
    }
    if (mode == 1 && (result.ride_model.is_null() || !result.ride_model.str_contains(ride_type))) ||
        (mode == 2 && (result.ride_dress_model.is_null() || !result.ride_dress_model.str_contains(ride_type)))
    {
        let head = if result.head_model.is_null() { result.head_model.to_string() } else { String::from("null") };
        match mount {
            Mount::Wyvern => {
                ride_asset = Some("Wng2DR");
                if head.contains("303") { suffix = "c303" }
                else if head.contains("350") {
                    ride_asset = Some("Lnd0DR");
                    suffix = "c350";
                }
                else if head.contains("553") {
                    ride_asset = Some("Msn0DR");
                    if head.contains("553b") { suffix = "c553b"; }
                    else { suffix = "c553"; }
                }
            }
            Mount::Pegasus => {
                ride_asset = Some("Wng0ER");
                if head.contains("351") {
                    ride_asset = Some("Slp0ER");
                    suffix = "c351";
                }
            }
            Mount::Wolf => {
                ride_asset = Some("Cav2CR");
                if head.contains("452") { suffix = "c452"; }
            }
            Mount::Cav => { ride_asset = Some("Cav1BR"); }
            _ => { ride_asset = None }
        }
    }
    else { ride_asset = None; }
    if let Some(ride) = ride_asset {
        let ride_model = format!("{}_{}_{}", prefix, ride, suffix);
        if mode == 1 { result.ride_model = ride_model.into(); }
        else { result.ride_dress_model = ride_model.into(); }
    }
}
fn correct_mis_gender_animations(result: &mut AssetTableResult, kind: i32, gender: Gender, mount: Mount, mode: i32, is_canon: bool, condition_flags: ConditionFlags) {
    let is_morph = condition_flags.contains(ConditionFlags::Corrupted);
    let mut new_gender = gender;
    if mode == 2 {
        if !result.dress_model.is_null() {
            let dress_model = result.dress_model.to_string();
            if gender == Gender::Male && dress_model.contains("F_c") {
                new_gender = Gender::Female;
                result.body_anims.clear();
                add_standard_for_mount_type(result, kind, Gender::Female, mode, mount, is_morph);
                result.body_anims.add(create_anim_from_mount(mount, Gender::Female, kind).into());
            }
            else if gender == Gender::Female && dress_model.contains("M_c") {
                new_gender = Gender::Male;
                result.body_anims.clear();
                add_standard_for_mount_type(result, kind, Gender::Male, mode, mount, is_morph);
                result.body_anims.add(create_anim_from_mount(mount, Gender::Male, kind).into());
            }
        }
        if !has_mount_weapon_anim(result, mount, new_gender, kind) {
            if kind == 8 {
                let gen_str = if new_gender == Gender::Male { "M" } else { "F" };
                result.body_anims.clear();
                result.scale_stuff[16] = 2.6;
                remove_mounts_accs(result);
                let new_anim = create_anim_from_mount(Mount::None, gender, kind);
                result.body_anims.add(concat_string!("Com0A", gen_str, "-No1_c000_N").into());
                result.body_anims.add(concat_string!("Com0A", gen_str, "-", WEP_PRE[kind as usize], "1_c000_N").into());
                result.body_anims.add(new_anim.into());
            }
            else if kind == 9 {
                special_item_type_anim(result, new_gender, mode, is_canon);
            }
        }
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
