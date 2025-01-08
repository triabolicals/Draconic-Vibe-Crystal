use super::{*, animation::*, transform::asset_table_result_setup_person_hook};
use crate::randomizer::assets::accessory::add_accessory_to_list;
use crate::randomizer::assets::accessory::clear_accessory_from_list;
use accessory::change_accessory;
use skyline::patching::Patch;
static mut TWIN_STRIKE_EMBLEM: i32 = 11;
static mut THREE_HOUSES: i32 = 12;
static mut ROBIN: i32 = 22;

const MALE_EMBLEMS: [usize; 9] = [0, 1, 4, 5, 8, 9, 14, 16, 18];
const THREE_HOUSE_ACTS: [&str; 4] = ["Thr2AF-Ax1_c563_N", "Thr2AM-Lc1_c514_N", "Thr2AM-Bw1_c515_N", "Thr2AM-Sw1_c535_N"];
pub fn find_and_replace_emblem_animation(result: &mut AssetTableResult, body_act: String) {
    ENGAGE_PREFIX.iter().for_each(|w|{
        result.body_anims.iter_mut().filter(|act| str_contains(act, w)).for_each(|animation|{
            if str_contains(animation, "Com0A") && str_contains(animation, "-No1") {}
            else { *animation = Il2CppString::new_static(body_act.clone()); }
        })
    });
    if let Some(last) = result.body_anims.iter_mut().last() {  *last = Il2CppString::new_static(body_act.clone()); }
    result.body_anims.iter_mut().for_each(|animation| println!("Emblem Act: {}", animation.to_string()));
}

fn houses_unite_plus_act(result: &mut AssetTableResult, kind: i32) {
    if kind >= 4 || kind < 0 { return; }
    result.body_anims.iter_mut().for_each(|str| *str = THREE_HOUSE_ACTS[kind as usize].into()); 
    result.right_hand = "null".into();
    result.left_hand = "null".into();
    match kind {
        0 => {result.right_hand = "uWep_Ax20".into();}
        1 => {result.right_hand = "uWep_Lc21".into();}
        2 => {
            result.right_hand = "uWep_Bw14-Ar".into();
            result.left_hand = "uWep_Bw14-Bw".into();
        }
        _ => {}
    }
}

#[skyline::hook(offset=0x01bb2d80)]
pub fn asset_table_result_god_setup(this: &mut AssetTableResult, mode: i32, god_data: Option<&GodData>, is_darkness: bool, conditions: &mut Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    if god_data.is_none() {  return call_original!(this, mode, god_data, is_darkness, conditions, method_info);  }
    let god = god_data.unwrap();
    if conditions.iter().any(|str| str.to_string() == "協力エンゲージ技") && mode == 2 && !GameVariableManager::get_bool("G_Random_Names") {    // Houses Unite+ Three Houses Replacements (Byleth for Robin)
        let three_houses = unsafe { THREE_HOUSES } as usize;
        let mut result = call_original!(this, mode, god_data, is_darkness, conditions, method_info);
        let gid = god.gid.to_string();
        let kind = if gid.contains("エーデルガルト") { 0 } else if gid.contains("ディミトリ") { 1 } else if gid.contains("クロード") { 2 } else { 3 };
        let animation = result.body_anims.iter().position(|act| act.to_string().contains("Thr2A"));
        if animation.is_none() || three_houses > 21 { return result; }
        if let Some(con) = conditions.iter_mut().find(|con| con.to_string() == "協力エンゲージ技")  {
            *con = "".into();
        }
        match three_houses {
            9|12|20 => { return result; }
            21|15 => {
                if kind == 0 {
                    result.dress_model = "uBody_Ver0AF_c562".into();
                    result.head_model = "uHead_c562".into();
                    result.hair_model = "uHair_null".into();
                    change_accessory(result.accessory_list, "uAcc_spine2_Hair562", "c_spine1_jnt");
                    change_accessory(result.accessory_list, "uAcc_head_Tiara562", "c_head_loc");
                }
                else {
                    let rng = Random::get_system();
                    let male = MALE_EMBLEMS[ rng.get_value(9) as usize ];
                    result = call_original!(this, mode,  GodData::get(EMBLEM_GIDS[male]), is_darkness, conditions, method_info);
                    houses_unite_plus_act(result, kind);
                }
            }
            6|18 => {
                if kind == 0 {
                    result.dress_model = "uBody_Luc0AF_c584".into();
                    result.head_model = "uHead_c584".into();
                    result.hair_model = "uHair_null".into();
                    change_accessory(result.accessory_list, "uAcc_spine2_Hair584", "c_spine1_jnt");
                }
                else if kind != 3 {
                    result.dress_model = "uBody_Chr0AM_c512".into();
                    result.head_model = "uHead_c512".into();
                    result.hair_model = "uHair_h513".into();
                }
                else {
                    result.dress_model = "uBody_Rbi0AM_c513".into();
                    result.head_model = "uHead_c513".into();
                    result.hair_model = "uHair_h513".into();
                }
            }
            11 => {
                if kind == 0 {
                    result.dress_model = "uBody_Eir0AF_c582".into();
                    result.head_model = "uHead_c582".into();
                    result.hair_model = "uHair_null".into();
                    change_accessory(result.accessory_list, "uAcc_spine2_Hair582", "c_spine1_jnt");
                }
                else {
                    result.dress_model = "uBody_Eph0AM_c536".into();
                    result.head_model = "uHead_c536".into();
                    result.hair_model = "uHair_h536".into();
                }
            }
            19 => {
                if kind == 0 {  // Female Alear
                    result.dress_model = "uBody_Drg0AF_c053".into();
                    result.head_model = "uHead_c053".into();
                    result.hair_model = "uHair_null".into();
                    change_accessory(result.accessory_list, "uAcc_spine2_Hair0053", "c_spine1_jnt");
                }
                else if kind != 3 { // Alear
                    result.dress_model = "uBody_Drg0AM_c003".into();
                    result.head_model = "uHead_c003".into();
                    result.hair_model = "uHair_h003".into();
                }
            }
            _ => { 
                let new_god2 = GodData::get(EMBLEM_GIDS[three_houses]).unwrap();
                if new_god2.female == 1 && kind == 0 { 
                    result = call_original!(this, mode, Some(new_god2), is_darkness, conditions, method_info);
                    houses_unite_plus_act(result, 0);
                }
                else if new_god2.female == 0 && kind != 0 { 
                    result = call_original!(this, mode, Some(new_god2), is_darkness, conditions, method_info);
                    houses_unite_plus_act(result, kind);
                }
            }
        }
        remove_mounts_accs(result);
        return result;
    }
    if GameVariableManager::get_bool("G_Random_Names") {
        let mode2 = if mode > 10 { mode - 10 } else { mode };

        if let Some(emblem) = EMBLEM_ASSET.iter().position(|&x|god.gid.contains(x)) {
            if unsafe { EMBLEM_NAMES[emblem] } != -1 {
                let index: usize = unsafe { EMBLEM_NAMES[emblem] as usize };
                let person = PersonData::get(PIDS[index]) ;
                let hash = person.unwrap().parent.index + ( GameVariableManager::get_number("G_Random_Seed") as u32 + 10 ) as i32 >> 2 ;
                let job = person.unwrap().get_job().unwrap();
                let item = crate::randomizer::job::get_weapon_for_asset_table(job);
            //Engaging
                if conditions.iter_mut().any(|str| str.contains("エンゲージ開始")) {
                    let mut body_vec: Vec<String> = Vec::new();
                    let reuslt1 = call_original!(this, mode2, god_data, is_darkness, conditions, method_info);
                    reuslt1.body_anims.iter().for_each(|str|{ body_vec.push(str.to_string()); println!("Engaging Act: {}", str) });
                    let result = asset_table_result_setup_person_hook(this, mode2, person, person.unwrap().get_job(), item, conditions, None);
                    for x in 0..body_vec.len() {
                        if x >= result.body_anims.len() { result.body_anims.add(body_vec[x].clone().into()); }
                        else { result.body_anims[x] = body_vec[x].clone().into(); }
                    }
                    if CONFIG.lock().unwrap().misc_option_1 >= 4.75 {
                        let rng = Random::instantiate().unwrap();
                        rng.ctor(hash as u32);
                        result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.03;
                    }
                    return result;
                }
                let result = asset_table_result_setup_person_hook(this, mode2, person, person.unwrap().get_job(), item, conditions, None);

                if conditions.iter().any(|str| str.contains("協力エンゲージ技")) && mode2 == 2 {  //House Unite+
                    //result.body_anims.iter().for_each(|str| // println!("Act: {}", str));
                    let gid = god.gid.to_string();
                    if gid.contains("エフラム") {
                        result.body_anims.iter_mut().for_each(|str| *str = "Eir1AM-Lc1_c536_N".into());
                        result.right_hand = "uWep_Lc19".into(); 
                    }
                    else {
                        let kind = if gid.contains("エーデルガルト") { 0 } else if gid.contains("ディミトリ") { 1 } else if gid.contains("クロード") { 2 } else { 3 };
                        houses_unite_plus_act(result, kind);
                    }
                }
                else {
                    let gender = if person.unwrap().gender == 1 {
                        if person.unwrap().get_flag().value & 32 != 0 { 2 }
                        else { 1 }
                    }
                    else { 2 };
                    super::animation::change_unique_class(job, result, mode2, gender, item, true);
                }
                emblem_bust_randomization(result, hash);
                return result;
            }
        }
    }
    let hash = god_data.unwrap().parent.hash + ( GameVariableManager::get_number("G_Random_Seed") as u32 + 10 ) as i32 >> 2 ;

    if mode > 10 { 
        let result = call_original!(this, mode-10, god_data, is_darkness, conditions, method_info); 
        emblem_bust_randomization(result, hash);
        println!("Create Chapter 22 Emblem!");
        return result;
    }

    let gid = god_data.unwrap().gid.to_string(); 
    if let Some(is_enemy_emblem) =  crate::randomizer::emblem::enemy::ENEMY_EMBLEMS.iter().find(|&x| x.0 == gid) {
        let emblem_index = is_enemy_emblem.1;
        //et new_emblem = crate::randomizer::emblem::EMBLEM_ORDER.lock().unwrap()[emblem_index as usize] as usize;
        if let Some(replace_god) = crate::randomizer::emblem::get_god_from_index(emblem_index, true) {
            let new_emblem = unsafe { crate::randomizer::emblem::EMBLEM_LIST.iter().position(|&hash| hash == replace_god.parent.hash).unwrap() };
            let emblem = 
                if new_emblem < 12 || new_emblem >= 19 { replace_god }
                else { GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[new_emblem])).unwrap() };
            let is_m002 = gid == "GID_M002_シグルド";
            let result = call_original!(this, mode, Some(emblem), !is_m002, conditions, method_info);
            emblem_bust_randomization(result, hash);
            return result;
        }
        else {
            let result = call_original!(this, mode, god_data, is_darkness, conditions, method_info); 
            emblem_bust_randomization(result, hash);
            return result;
        }
    }
    else { 
        let result = call_original!(this, mode, god_data, is_darkness, conditions, method_info); 
        emblem_bust_randomization(result, hash);
        return result;
    }
}

fn emblem_bust_randomization(result: &mut AssetTableResult, hash: i32){
    if CONFIG.lock().unwrap().misc_option_1 >= 4.75 {
        let rng = Random::instantiate().unwrap();
        rng.ctor(hash as u32);
        result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.025;
    }
}

#[skyline::hook(offset=0x01bb7ca0)]
pub fn asset_table_result_get_preset_name(name: &Il2CppString, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let mut result = call_original!(name, method_info);
    let asset_table = AssetTable::get_list().unwrap();
    let pre_name = name.to_string();

    if pre_name == "エンゲ技/【エイリーク】ツインストリーム/エフラム" { // Replacing Ephraim in Twin Strike
        for x in 1..1000 {
            if let Some(con) = &asset_table[x].conditions {
                let new_conditions = Array::<&Il2CppString>::new_specific( con.get_class(), 1).unwrap();
                let rng = Random::get_system();
                let eirika = unsafe { TWIN_STRIKE_EMBLEM } as usize;
                if eirika > 21 { break; }
                if GameVariableManager::get_bool("G_Random_Names") {
                    new_conditions[0] = "協力エンゲージ技".into();
                    return asset_table_result_god_setup(result, 2, GodData::get("GID_エフラム"), false, new_conditions, None);
                }
                match eirika {
                    15|21 => {
                        new_conditions[0] = "".into();
                        let male = MALE_EMBLEMS[ rng.get_value(9) as usize ];
                        result = asset_table_result_god_setup(result, 2, GodData::get(EMBLEM_GIDS[male]), false, new_conditions, None); 
                    }
                    18 => { // Robin
                        result.dress_model = "uBody_Rbi0AM_c513".into();
                        result.head_model = "uHead_c513".into();
                        result.hair_model = "uHair_h133".into();
                        return result;
                    }
                    19 => { // Alear
                        result.dress_model = "uBody_Drg0AM_c003".into();
                        result.head_model = "uHead_c003".into();
                        result.hair_model = "uHair_h003".into();
                        return result;
                    }
                    11 => { return result; }
                    12|20 => {
                        new_conditions[0] = "".into();
                        let dimi_claude = if rng.get_value(2) == 1 { GodData::get("GID_ディミトリ")  }
                            else { GodData::get("GID_クロード")  };
                        result = asset_table_result_god_setup(result, 2, dimi_claude, false, new_conditions, None);
                    }
                    _ => {
                        new_conditions[0] = "".into();
                        if let Some(god) = GodData::get(EMBLEM_GIDS[eirika]) {
                            if god.female == 0 { result = asset_table_result_god_setup(result, 2, Some(god), false, new_conditions, None); }
                            else { return result; }
                        }
                    }
                }
                // Animation Replacement 
                remove_mounts_accs(result);
                result.body_anims.iter_mut().for_each(|str| *str = "Eir1AM-Lc1_c536_N".into());
                result.right_hand = "uWep_Lc19".into(); 
                result.left_hand = "null".into(); 
                break;
            }
        }
    }
    return result;
}

pub fn correct_emblem_animations(unit: &Unit, result: &mut AssetTableResult, mode: i32, equipped: Option<&ItemData>) {
    let job = &unit.job;
    //if str_contains(job.jid, "JID_紋章士") { return;}
    let gender = unit_dress_gender(unit);
    if gender == 0 || gender > 2 { return; }
    if mode != 1 && mode != 2 { return; }
    let gender_str = if gender == 1 { "M" } else { "F" };
    let mut body = "NA".to_string();
    let mut act_type = get_animation_type(job);

    if gender == 1 && ( act_type == 9 || act_type == 10 ) {  //Male Infantry replace last animation if it's Com0AM to Swd0AM
        if let Some(last) = result.body_anims.iter_mut().last() {
            if last.to_string() == "Com0AM-Sw1_c000_N" {
                *last = Il2CppString::new_static("Swd0AM-Sw1_c000_N");
            }
        }
    }
    //Assign mounts
    if act_type == 12 && gender == 1 { act_type = 8; }
    println!("Emblem: {} has act type {}", Mess::get_name(unit.person.pid).to_string(), act_type);
    let kind = if equipped.is_some() { equipped.unwrap().kind } else { 0 };
    if !job.jid.contains("紋章士_") {
        match (act_type, mode) {
            (0|11|14, 1) => { 
                result.ride_model = "oBody_Sig0BR_c531".into();
                result.ride_anim = Some("UAS_Sig0BR".into());
            }
            (0|11|14, 2) => {
                result.ride_dress_model = "uBody_Sig0BR_c531".into();
                if kind < 4  || kind == 7 { // Sword / Axe / Lance / Rod
                    result.body_anims.add( Il2CppString::new_static(concat_string!("Com0B", gender_str, "-", WEP_PRE[kind as usize], "1_c000_N")));
                }
                result.ride_model = "uRig_HorsR".into();
            }
            (3, 1) => { //Wvyern
                result.ride_anim = Some("UAS_Wng2DR".into()); 
                result.scale_stuff[18] = 0.50;
                result.ride_model = "oBody_Cmi0DR_c561".into();
            }
            (3, 2) => {
                result.ride_dress_model= "uBody_Cmi0DR_c561".into();
                result.ride_model = "uRig_DragR".into();
            }
            (4, 1) => {
                result.ride_anim = Some("UAS_Wng0ER".into());
                result.scale_stuff[18] = 0.50;
                result.ride_model = "oBody_Wng0ER_c000".into();  }
            (4, 2) => { 
                result.ride_dress_model = "uBody_Wng0ER_c000".into();
                result.ride_model = "uRig_PegaR".into();
            }
            (5, 1) => { 
                result.ride_anim = Some("UAS_Wng1FR".into());
                result.ride_model = "oBody_Wng1FR_c000".into(); }
            (5, 2) => {                         
                result.ride_model = "uRig_GrifR".into();
                result.ride_dress_model = "uBody_Wng1FR_c000".into();
            }
            (6, 1) => { 
                result.ride_anim = Some("UAS_Cav2CR".into());
                result.ride_model = "oBody_Cav2CR_c000".into(); 
            }
            (6, 2) => {
                result.ride_model = "uRig_WolfR".into();
                result.ride_dress_model = "uBody_Cav2CR_c000".into();
            }
            (12, 1) => {
                result.ride_model = "oBody_Msn0DR_c553".into();
                result.ride_anim = Some("UAS_Msn0DR".into());
                result.scale_stuff[18] = 0.50;
            }
            (12, 2) => {
                result.ride_model = "uRig_DragR".into();
                result.ride_dress_model = "uBody_Msn0DR_c553".into();
            }
            _ => {  //Mage Cannoneer / Infantry
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into(); 
                result.ride_anim = None;
            }
        }
    }
    let emblem_generic = if mode == 2 { concat_string!("Enb0A", gender_str, "-",  WEP_PRE[kind as usize], "1_c000_N") } else { format!("UAS_Enb0A{}", gender_str) };
    if change_unique_class(unit.job, result, mode, gender, equipped, true) { return;  } // Check if Unique Class
    let new_act = get_animation_index(job, equipped, true, gender == 2) as usize;   // New Animation Index
    if check_change_animation_type(job, equipped, gender == 2 ) { // Removes mounts if impossible mount + animation weapon combo 
        result.ride_dress_model = "null".into();
        result.ride_model = "null".into();
        result.ride_anim = None;
        body = emblem_generic;
    }
    else if act_type == 7 && kind == 9 {    // Mage Cannoneer Bullets
        body = if mode == 2 { format!("Mcn3A{}-Mg2_c000_M", gender_str) }
        else { format!("UAS_Mcn3A{}", gender_str )};
    }
    else if act_type == 12 {
        if kind != 6 && kind != 1 {
            body = if mode == 2 { concat_string!("Wng2DF-", WEP_PRE[kind as usize], "1_c000_N") } else { "UAS_Msn0DF".to_string() };    // Wyvern 
        }
        else {
            body = if mode == 2 { concat_string!("Msn1DF-", WEP_PRE[kind as usize], "1_c553_N") } else { "UAS_Msn0DF".to_string() };    // Melsuine Sword / Tome
        }
    }
    else if new_act == 17 || job.jid.contains("紋章士_") {  // Emblem Animations + Default Emblem
        if let Some(pos) = ENGAGE_PREFIX.iter().position(|w| result.body_anims.iter().any(|act| act.contains(w))) {
            if pos != 1 && pos != 17 && job.jid.contains("紋章士_") {
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into();
                result.ride_anim = None;
                
            }
            match pos { 
                1 => { // Remove Mount Sigurd   on not horse / not sword/lance
                    if kind == 1 { return; }
                    if !is_mounted(act_type ) || (act_type != 0 && act_type != 11 && act_type != 14 ) {
                        result.ride_dress_model = "null".into();
                        result.ride_model = "null".into();
                        result.ride_anim = None;
                        body = emblem_generic;
                    }
                }
                17 => { // Remove Mount Camilla for not tome / axe and not wyvern 
                    if kind == 3 || kind == 6 { return;   }
                    else if !is_mounted(act_type ) || (act_type != 3 && act_type != 12) {
                        result.ride_dress_model = "null".into();
                        result.ride_model = "null".into();
                        result.ride_anim = None;
                        body = emblem_generic;
                        if mode == 1 { result.scale_stuff[16] = 2.6; }
                    }
                }
                0|4|5|6|8|9|10|11|18|19|26 => { if kind != 1 { body = emblem_generic; } }, // Not Swords
                12|13|25 => { body = emblem_generic; },  //  Always generic  Tiki / Camilla
                14|20 => { if kind != 3 {  body = emblem_generic; } }    //Hector / Edelgard for not Axes
                21|23 => { if kind != 2 {  body = emblem_generic; } } // Dimitri / Ephiram for not lance
                7|22 => { if kind != 4 { body = emblem_generic; } }  //Lyn / Claude for not Bows
                2|3|15|16 => { if kind != 6 { body = emblem_generic; } } //Celica/Micaih/Veronica/Soren for not Tome
                _ => { body = emblem_generic; }
            }
        }
    }
    else {  // The Rest
        body = if mode == 2 { find_animation_set(format!("{}{}-{}", ACT_PRE[new_act], gender_str, WEP_PRE[kind as usize])) }
            else { format!("UAS_{}{}", ACT_PRE[new_act], gender_str) };
    }
   // println!("Replacing Animation with Body: {} for {}", body, Mess::get_name(unit.person.pid));
    if body != "NA" {
        result.body_anim = Some(body.clone().into());
        find_and_replace_emblem_animation(result, body.clone());
    }
}

pub fn adjust_engage_attack_animation(result: &mut AssetTableResult, unit: &Unit, equipped: Option<&ItemData>, mode: i32) {
    unsafe { 
        TWIN_STRIKE_EMBLEM= 11;
        THREE_HOUSES = 12;
        ROBIN = 22;
    }
    if let Some(engage_attack) =  unsafe { get_engage_attack(unit, None) } {
        let mut animation_index = 0;
        let mut old_engage = 50;
        // Finding the Engage Animation Index in Body Animations

        result.body_anims.iter()
            .for_each(|act|{
                if let Some(animation) = EATK_ACT.iter().position(|&y| act.contains(y)) { old_engage = animation; }
                else if act.to_string().contains("Ler2A") {
                    old_engage = 49;
                }
                else if old_engage == 50 { animation_index += 1; }
                println!("Engage Act {}: {}", animation_index, act);
            }
        );
        let gender = unit_dress_gender(unit);
        let mut gender_str = if gender == 2 { "F" } else { "M" };

        if old_engage == 49 {
            if let Some(link) = unsafe { unit_get_engage_link_unit(unit, None )} {
                println!("Linked Unit: {}", Mess::get_name(link.person.pid));
                return;
            }
            if let Some(god) = unit.god_unit {
                if god.child.is_none() {
                    result.body_anims.iter_mut()
                    .filter(|act| act.contains("Ler2A"))
                    .for_each(|act| *act = concat_string!("Ler1A", gender_str,"-Sw1_c000_N").into());
                }
            }
            return;
        }
        else if old_engage == 50 {
            if let Some(god) = unit.god_unit {
                let rr = god.data.gid.to_string();
                if let Some(pos) = EMBLEM_ASSET.iter().position(|r| rr.contains(r)) {
                    old_engage = match pos {
                        12|20|21 => { 12 },
                        22 => { 18 },
                        23 => { 11 },
                        _ => { pos },
                    };
                }
            }
        }
        // Get New Engage Attack
        let emblem_index = if let Some(pos) = EMBLEM_ASSET.iter().position(|god| engage_attack.sid.contains(god)) { pos }
            else if engage_attack.sid.contains("三級長エンゲージ技＋") { 20 }
            else if engage_attack.sid.contains("三級長エンゲージ") { 12 }
            else { 50 };

        println!("Found Engage Attack {} and Old Emblem: {}", emblem_index, old_engage);
        if emblem_index == 50 { return; }

        if emblem_index == old_engage { 
            if emblem_index == 1 {
                result.ride_model = "uRig_HorsR".into();
                result.ride_dress_model = "uBody_Sig0BR_c531".into();
            }
            if emblem_index == 17 {
                result.ride_model = "uRig_DragR".into();
                result.ride_dress_model = "uBody_Cmi0DR_c561".into(); 
            }
            else {
                result.ride_model = "null".into();
                result.ride_dress_model = "null".into();
            }
            return; 
        }
        if mode == 1 {
            if old_engage == 13 {
                result.body_model = "oBody_Tik0AF_c560".into();
                result.hair_model = "oHair_h560".into();
            }
            let act = if emblem_index == 12 || emblem_index == 20 {  concat_string!("UAS_Mar1A", gender_str) } 
                else if emblem_index == 13 { concat_string!("UAS_Mic1A", gender_str) }
                else { concat_string!("UAS_", ENGAGE_PREFIX[emblem_index], ",1A", gender_str) };

            if result.body_anims.len() <= animation_index { result.body_anims.add(Il2CppString::new_static(act.clone()));  }
            else { result.body_anims[animation_index] = Il2CppString::new_static(act.clone()); }
            result.body_anim = Some(act.clone().into());
            return;
        }
        let mpid = unit.person.get_name().unwrap().to_string();
        result.ride_model = "null".into();
        result.ride_dress_model = "null".into();

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
                clear_accessory_from_list(result.accessory_list, "uWep_Ax20");
                clear_accessory_from_list(result.accessory_list, "uWep_Lc21");
                clear_accessory_from_list(result.accessory_list, "uWep_Bw16s-Bw");
            }
            13 => { //Change Dragon Tiki to Human Tiki for all animations not Divine Blessing
                if emblem_index != 13 {
                    result.dress_model = "uBody_Tik0AF_c560".into();
                    result.body_model = "uRig_GodF1".into();
                    result.head_model = "uHead_c560".into();
                    result.hair_model = "uHair_null".into();
                    add_accessory_to_list(result.accessory_list, "uAcc_spine2_Hair560", "c_spine1_jnt");
                    add_accessory_to_list(result.accessory_list, "uAcc_head_Tiara560", "c_head_loc");
                    gender_str = "F";
                }
            }
            15 => { clear_accessory_from_list(result.accessory_list, "uAcc_Event_SummonStone"); }
            21 => { clear_accessory_from_list(result.accessory_list, "uAcc_Event_SummonStoneB"); }
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
                if emblem_index == 15 {  add_accessory_to_list(result.accessory_list, "uAcc_Event_SummonStone", "reserve4_loc");  }
                if emblem_index == 21 {  add_accessory_to_list(result.accessory_list, "uAcc_Event_SummonStoneB", "reserve4_loc");  }
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
                unsafe { TWIN_STRIKE_EMBLEM = old_engage  as i32 };
            }   // Eirika
            12 => { 
                new_engage_animation = concat_string!("Thr1A",  gender_str, "-Ax1_c000_N");
                result.left_hand = "uWep_Ax30L".into();
                result.right_hand = "uWep_Ax30R".into();
                result.trail = "cEff_EngageA_Swd_00".into();
                add_accessory_to_list(result.accessory_list, "reserve1_loc", "uWep_Ax20");
                add_accessory_to_list(result.accessory_list, "reserve2_loc", "uWep_Lc21");
                add_accessory_to_list(result.accessory_list, "reserve3_loc", "uWep_Bw16s-Bw");
            } //Houses Unite  
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
                else if mpid == "MPID_Anna" { new_engage_animation = "Chr1AF-Sw1_c552_N".to_string(); }
                else if ( mpid == "MPID_Lueur" && gender == 2 )  || mpid == "MPID_Chloe" || mpid == "MPID_Jade" || mpid == "MPID_Ivy" || mpid == "MPID_Merin" || mpid == "MPID_Saphir" {
                    new_engage_animation = "Chr1AF-Sw1_c254_N".to_string();
                }
                else if mpid == "MPID_El" || mpid == "MPID_Selestia" { new_engage_animation = "Chr1AF-Sw1_c254_N".to_string(); }
                else { new_engage_animation = concat_string!("Chr1A", gender_str, "-Sw1_c000_N"); }
                equip_weapon(result, equipped);
                unsafe { ROBIN = old_engage as i32};
            }
            20 => { new_engage_animation = concat_string!("Thr2A", gender_str, "-Ax1_c000_N");
                unsafe { THREE_HOUSES = old_engage as i32 };
                }  // Houses Unite+
            _ => { return; },
        }
    // overwrite old engage attack animation
        result.body_anims.add(Il2CppString::new_static(new_engage_animation.clone()));
        if old_engage == 21 { old_engage = 15; }
        else if old_engage == 50 { return;  }

        result.body_anims.iter_mut()
            .filter(|act| act.to_string().contains(ENGAGE_PREFIX[old_engage]))
            .for_each(|act| *act = new_engage_animation.clone().into());
        
        result.body_anims.iter().for_each(|act| println!("Adjusted Act: {}", act));
    }
}

#[skyline::hook(offset=0x01bb4180)]
pub fn asset_table_robin_hook(this: &mut AssetTableResult, mode: i32, person: &PersonData, conditions: &mut Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let mut result = call_original!(this, mode, person, conditions, method_info);
    if mode == 2 && person.pid.to_string() == "PID_ルフレ" {
        if let Some(con) = conditions.iter_mut().find(|con| con.to_string() == "エンゲージ技")  {
            *con = "".into();
            if GameVariableManager::get_bool("G_Random_Names") {
                if unsafe { EMBLEM_NAMES[22] } != -1 {
                    let index: usize = unsafe { EMBLEM_NAMES[22] as usize };
                    let person = PersonData::get(PIDS[index]) ;
                    let job = person.unwrap().get_job().unwrap();
                    let item = crate::randomizer::job::get_weapon_for_asset_table(job);
                    result = asset_table_result_setup_person_hook(this, 2, person, None, item, conditions, None);
                }
            }
            else {
                let robin = unsafe { ROBIN } as usize;
                if robin > 21 { return result; }
                let rng = Random::get_system();
                match robin {
                    15|21 => {
                        let male = MALE_EMBLEMS[ rng.get_value(9) as usize ];
                        result = asset_table_result_god_setup(result, 2, GodData::get(EMBLEM_GIDS[male]), false, conditions, None); 
                    }
                    6|18 => { return result; }
                    19 => { // Alear
                        result.dress_model = "uBody_Drg0AM_c003".into();
                        result.head_model = "uHead_c003".into();
                        result.hair_model = "uHair_h003".into();
                    }
                    11 => {                     
                        result.dress_model = "uBody_Eph0AM_c536".into();
                        result.head_model = "uHead_c536".into();
                        result.hair_model = "uHair_h536".into();
                        return result; 
                    }
                    0|13 => {
                        result.dress_model = "uBody_Mar0AM_c530".into();
                        result.head_model = "uHead_c530".into();
                        result.hair_model = "uHair_h530".into();
                        return result; 
                    }
                    12|20 => {
                        let dimi_claude = if rng.get_value(2) == 1 { GodData::get("GID_ディミトリ")  }
                            else { GodData::get("GID_クロード")  };
                        result = asset_table_result_god_setup(result, 2, dimi_claude, false, conditions, None);
                    }
                    _ => {
                        if let Some(god) = GodData::get(EMBLEM_GIDS[robin]) {
                            if god.female == 0 { result = asset_table_result_god_setup(result, 2, Some(god), false, conditions, None); }
                            else { return result; }
                        }
                    }
                }
            }
            remove_mounts_accs(result);
            result.left_hand = "null".into();
            result.right_hand = "uWep_Mg26".into();
            result.magic = "MG_DLC6_2".into();
            result.trail = "cEff_EmblemA_Swd_00".into();
            result.body_anims.iter_mut().for_each(|str| *str = "Chr1AM-Mg1_c513_M".into());
        }
    }
    result
}

#[unity::from_offset("App", "Unit", "GetEngageLinkUnit")]
fn unit_get_engage_link_unit(this: &Unit, method_info: OptionalMethod) -> Option<&'static Unit>;