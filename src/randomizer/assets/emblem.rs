use super::{*, transform::asset_table_result_setup_person};
use crate::randomizer::assets::accessory::add_accessory_to_list;
use crate::randomizer::assets::accessory::clear_accessory_from_list;
use crate::DVCVariables;
use accessory::change_accessory;
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
                if conditions.iter_mut().any(|str| str.to_string().contains("エンゲージ開始")) {
                    let mut body_vec: Vec<String> = Vec::new();
                    let reuslt1 = call_original!(this, mode2, god_data, is_darkness, conditions, method_info);
                    reuslt1.body_anims.iter().for_each(|str|{ body_vec.push(str.to_string()); println!("Engaging Act: {}", str) });
                    let result = unsafe { asset_table_result_setup_person(this, mode2, person, person.unwrap().get_job(), item, conditions, None) };
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
                let result = unsafe { asset_table_result_setup_person(this, mode2, person, person.unwrap().get_job(), item, conditions, None) };

                if conditions.iter().any(|str| str.to_string().contains("協力エンゲージ技")) && mode2 == 2 {  //House Unite+
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
    let hash = god_data.unwrap().parent.hash + ( DVCVariables::get_seed() as u32 + 10 ) as i32 >> 2 ;

    if mode > 10 { 
        let result = call_original!(this, mode-10, god_data, is_darkness, conditions, method_info); 
        emblem_bust_randomization(result, hash);
        println!("Create Chapter 22 Emblem!");
        return result;
    }

    let gid = god_data.unwrap().gid.to_string(); 
    let index = god_data.unwrap().parent.index;
    if gid.contains("GID_相手") {
        let result = call_original!(this, mode, god_data, true, conditions, method_info);
        emblem_bust_randomization(result, hash);
        return result;
        /*
        if let Some(emblem) = EMBLEM_ASSET.iter().position(|asset| gid.contains(asset)) {
            if emblem < 12 || emblem == 23 {
                let result = call_original!(this, mode, god_data, true, conditions, method_info);
                emblem_bust_randomization(result, hash);
                return result;
            }
            else if emblem != 19 {
                let result = call_original!(this, mode, GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[emblem])), true, conditions, method_info); 
                emblem_bust_randomization(result, hash);
                return result;
            }
        }
        */
    }
    if let Some(is_enemy_emblem) =  crate::randomizer::emblem::enemy::ENEMY_EMBLEMS.get().unwrap().iter().find(|&x| x.0 == index) {
        let emblem_index = is_enemy_emblem.1;

        //et new_emblem = crate::randomizer::emblem::EMBLEM_ORDER.lock().unwrap()[emblem_index as usize] as usize;
        if let Some(replace_god) = DVCVariables::get_god_from_index(emblem_index, true) {
            let is_m002 = gid == "GID_M002_シグルド";
            let new_emblem = crate::randomizer::emblem::EMBLEM_LIST.get().unwrap().iter().position(|&hash| hash == replace_god.parent.hash).unwrap();
            let emblem = 
                if new_emblem < 12 || new_emblem >= 19 || is_m002 { replace_god }
                // else if new_emblem == 13 { return call_original!(this, mode, god_data, is_darkness, conditions, method_info); }
                else { GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[new_emblem])).unwrap() };
        
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

pub fn adjust_engage_attack_animation(result: &mut AssetTableResult, unit: &Unit, equipped: Option<&ItemData>, mode: i32) {
    unsafe { 
        TWIN_STRIKE_EMBLEM= 11;
        THREE_HOUSES = 12;
        ROBIN = 22;
    }
    if let Some(engage_attack) =  unsafe { get_engage_attack(unit, None) } {
        let mut animation_index = 0;
        let mut old_engage = 50;
    // Replace Generic with random character voices
        if result.sound.voice.is_none() || result.sound.voice.is_some_and(|str|{
            let str1 = str.to_string();
            str1.contains("_MOB_Enemy") || str1.contains("ENEMY") }){
            let rng = Random::get_system();
            let index = rng.get_value(40) as usize + 1;
            let name = match index {
                36 => { "DLC_42"}
                37 => { "DLC_43"}
                38 => { "DLC_44"}
                39 => { "DLC_45"}
                40 => { "DLC_46"}
                _ =>  { &MPIDS[index][5..] }
            };
            result.sound.voice = Some(name.into());
        }
                // Finding the Engage Animation Index in Body Animations
        result.body_anims.iter()
            .for_each(|act|{
                if let Some(animation) = EATK_ACT.iter().position(|&y| act.to_string().contains(y)) { old_engage = animation; }
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
                    .filter(|act| act.to_string().contains("Ler2A"))
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
        let engage_sid = engage_attack.sid.to_string();
        let emblem_index = if let Some(pos) = EMBLEM_ASSET.iter().position(|god| engage_sid.contains(god)) { pos }
            else if engage_sid.contains("三級長エンゲージ技＋") { 20 }
            else if engage_sid.contains("三級長エンゲージ") { 12 }
            else { 50 };

        println!("Found Engage Attack {} and Old Emblem: {}", emblem_index, old_engage);
        if emblem_index == 50 { return; }

        if emblem_index == old_engage { 
            if emblem_index == 1 {
                result.ride_model = "uRig_HorsR".into();
                result.ride_dress_model = "uBody_Sig0BR_c531".into();
                result.body_anims.add(Il2CppString::new_static(concat_string!("Sig1B", gender_str, "-Sw1_c000_N")));
            }
            else if emblem_index == 17 {
                result.ride_model = "uRig_DragR".into();
                result.ride_dress_model = "uBody_Cmi0DR_c561".into(); 
                result.body_anims.add(Il2CppString::new_static(concat_string!("Cmi1D", gender_str ,"-Ax1_c000_N")));
            }
            else if emblem_index == 13 {
                if result.body_model.to_string() != "uRig_Tik1AT" {
                    tiki_engage(result, unit, 2);
                    result.body_anims.add( Il2CppString::new_static("Tik1AT-Mg1_c000_M"));
                }
                super::animation::change_hair_change(unit, result);
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
        if unit.god_unit.is_some_and(|gunit| gunit.data.gid.to_string().contains("敵チキ") || ( gunit.data.mid.to_string().contains("Tiki") && !gunit.data.gid.to_string().contains("チキ") )) { 
            // FX Tiki or Enemy Tiki
            return;
        }
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
                if emblem_index != 13 && !unit.god_unit.is_some_and(|god| god.data.gid.to_string().contains("敵チキ")){
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
                else { super::edit_asset_weapon(result, true, equipped); }
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
                super::edit_asset_weapon(result, true, equipped); 
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
                super::edit_asset_weapon(result, true, equipped); 
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
            14 => { new_engage_animation = concat_string!("Hec1A", gender_str, "-Ax1_c000_N"); super::edit_asset_weapon(result,true, equipped); } // Hector
            17 => { //Camilla 
                result.ride_model = "uRig_DragR".into();
                result.ride_dress_model = "uBody_Cmi0DR_c561".into(); 
                new_engage_animation = concat_string!("Cmi1D", gender_str,"-Ax1_c000_N");
                super::edit_asset_weapon(result, true, equipped);
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
                super::edit_asset_weapon(result, true, equipped);
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
                    result = unsafe { asset_table_result_setup_person(this, 2, person, None, item, conditions, None) };
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

pub fn tiki_engage(result: &mut AssetTableResult, unit: &Unit, mode: i32) {
    if mode == 2 {
        result.body_model = "uRig_Tik1AT".into();
        result.dress_model = "uBody_Tik1AT_c000".into();
        result.head_model = "null".into();
        result.hair_model = "null".into();
        result.info_anims = Some("AOC_Info_c745".into());
        result.talk_anims = Some("AOC_Talk_c745".into());
        ["c_spine1_jnt", "c_spine2_jnt", "c_hip_jnt", "l_arm3_jnt", "r_arm3_jnt", "l_leg3_jnt", "r_leg3_jnt"].iter().for_each(|locator|  change_accessory(result.accessory_list, "null", *locator));
        result.body_anims.add(Il2CppString::new_static("Ent0AT-Mg1_c000_N"));
        result.body_anims.add(Il2CppString::new_static("Ent0AT-Ft1_c000_N"));
        result.body_anims.add(Il2CppString::new_static("Ent0AT-Ft2_c000_N"));
    }
    else {
        result.body_model = "oBody_Tik1AT_c000".into();
        result.head_model = "oHair_null".into();
        ["c_spine1_jnt", "c_spine2_jnt", "c_hip_jnt", "l_arm3_jnt", "r_arm3_jnt", "l_leg3_jnt", "r_leg3_jnt"].iter().for_each(|locator|  change_accessory(result.accessory_list, "null", *locator));
        result.scale_stuff[18] = 0.50;
        result.scale_stuff[16] = 1.0;
        result.body_anim = Some("UAS_Ent0AT".into());
    }
    change_hair_change(unit, result);

}

pub fn get_emblem_attack_index(unit: &Unit) -> usize {
    if let Some(engage_attack) =  unsafe { get_engage_attack(unit, None) } {
        let sid = engage_attack.sid.to_string();
        if let Some(pos) = EMBLEM_ASSET.iter().position(|god| sid.contains(god)) { pos }
            else if sid.contains("三級長エンゲージ技＋") { 20 }
            else if sid.contains("三級長エンゲージ") { 12 }
            else { 50 }
    }
    else { 50 }
}
