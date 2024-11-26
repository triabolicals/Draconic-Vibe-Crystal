use super::{*, animation::*, transform::asset_table_result_setup_person_hook};
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

#[skyline::hook(offset=0x01bb2d80)]
pub fn asset_table_result_god_setup(this: &mut AssetTableResult, mode: i32, god_data: Option<&GodData>, is_darkness: bool, conditions: &mut Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    if god_data.is_none() {  return call_original!(this, mode, god_data, is_darkness, conditions, method_info);  }
    if GameVariableManager::get_bool("G_Random_Names") {
        let god = god_data.unwrap();
        if let Some(emblem) = EMBLEM_ASSET.iter().position(|&x|god.gid.contains(x)) {
            if unsafe { EMBLEM_NAMES[emblem] } != -1 {
                let index: usize = unsafe { EMBLEM_NAMES[emblem] as usize };
                let person = PersonData::get(PIDS[index]) ;
                let hash = person.unwrap().parent.index + ( GameVariableManager::get_number("G_Random_Seed") as u32 + 10 ) as i32 >> 2 ;
                let job = person.unwrap().get_job().unwrap();
                let item = crate::randomizer::job::get_weapon_for_asset_table(job);
            //Engaging
                if let Some(engaging) = conditions.iter_mut().find(|str| str.contains("エンゲージ開始")) {
                    let mut body_vec: Vec<String> = Vec::new();
                    let reuslt1 = call_original!(this, mode, god_data, is_darkness, conditions, method_info);
                    reuslt1.body_anims.iter().for_each(|str|{ body_vec.push(str.to_string()); println!("Engaging Act: {}", str) });
                    let result = asset_table_result_setup_person_hook(this, mode, person, person.unwrap().get_job(), item, conditions, None);
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
                let result = asset_table_result_setup_person_hook(this, mode, person, person.unwrap().get_job(), item, conditions, None);

                if conditions.iter().any(|str| str.contains("協力エンゲージ技")) && mode == 2 {  //House Unite+
                    //result.body_anims.iter().for_each(|str| // println!("Act: {}", str));
                    if god.gid.contains("エーデルガルト") {
                        if let Some(last) = result.body_anims.iter_mut().last() { *last = "Thr2AF-Ax1_c563_N".into(); }
                        result.right_hand = "uWep_Ax20".into();
                    }
                    if god.gid.contains("ベレト") {
                        result.body_anims.iter_mut().for_each(|str| *str = "Thr2AM-Sw1_c535_N".into()); 
                        result.right_hand = "null".into();
                        result.left_hand = "null".into();
                    }
                    if god.gid.contains("ディミトリ") {
                        result.body_anims.iter_mut().for_each(|str| *str = "Thr2AM-Lc1_c514_N".into()); 
                        result.right_hand = "uWep_Lc21".into();
                    }
                    if god.gid.contains("クロード") {
                        result.body_anims.iter_mut().for_each(|str| *str = "Thr2AM-Bw1_c515_N".into()); 
                        result.right_hand = "uWep_Bw14-Ar".into();
                        result.left_hand = "uWep_Bw14-Bw".into();
                    }
                    if god.gid.contains("エフラム") {
                        result.body_anims.iter_mut().for_each(|str| *str = "Eir1AM-Lc1_c536_N".into());
                        result.right_hand = "uWep_Lc19".into(); 
                    }
                }
                if CONFIG.lock().unwrap().misc_option_1 >= 4.75 {
                    let rng = Random::instantiate().unwrap();
                    rng.ctor(hash as u32);
                    result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.03;
                }
                return result;
            }
        }
    }
    let hash = god_data.unwrap().parent.hash + ( GameVariableManager::get_number("G_Random_Seed") as u32 + 10 ) as i32 >> 2 ;


    if mode > 10 { let result = call_original!(this, mode-10, god_data, is_darkness, conditions, method_info); 
        if CONFIG.lock().unwrap().misc_option_1 >= 4.75 {
            let rng = Random::instantiate().unwrap();
            rng.ctor(hash as u32);
            result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.03;
        }
        println!("Create Chapter 22 Emblem!");
        return result;
    }

    let gid = god_data.unwrap().gid.to_string(); 
    let is_enemy_emblem = crate::randomizer::emblem::enemy::ENEMY_EMBLEMS.iter().find(|&x| x.0 == gid);
    if is_enemy_emblem.is_some() {
        let emblem_index = is_enemy_emblem.unwrap().1;
        let new_emblem = crate::randomizer::emblem::EMBLEM_ORDER.lock().unwrap()[emblem_index as usize] as usize;
        if new_emblem > 19 { return call_original!(this, mode, god_data, is_darkness, conditions, method_info);  }
        let replace_god = if new_emblem < 12 { GodData::get( EMBLEM_GIDS[new_emblem]).unwrap() }
            else { GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[new_emblem])).unwrap() };

        let is_m002 = gid == "GID_M002_シグルド";
        let result = call_original!(this, mode, Some(replace_god), !is_m002, conditions, method_info);
        
        if CONFIG.lock().unwrap().misc_option_1 >= 4.75 {
            let rng = Random::instantiate().unwrap();
            rng.ctor(hash as u32);
            result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.03;
        }
        return result;
    }
    else { 
        let result = call_original!(this, mode, god_data, is_darkness, conditions, method_info); 
        if CONFIG.lock().unwrap().misc_option_1 >= 4.75 {
            let rng = Random::instantiate().unwrap();
            rng.ctor(hash as u32);
            result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.03;
        }
        return result;
    }
}

#[skyline::hook(offset=0x01bb7ca0)]
pub fn asset_table_result_get_preset_name(name: &Il2CppString, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let result = call_original!(name, method_info);
    let asset_table = AssetTable::get_list().unwrap();
    
    if GameVariableManager::get_bool("G_Random_Names") {
        let pre_name = name.to_string();
        if pre_name == "エンゲ技/【エイリーク】ツインストリーム/エフラム" {
            for x in 1..1000 {
                if let Some(con) = &asset_table[x].conditions {
                    let new_conditions = Array::<&Il2CppString>::new_specific( con.get_class(), 1).unwrap();
                    new_conditions[0] = "協力エンゲージ技".into();
                    let god = GodData::get("GID_エフラム");
                    return asset_table_result_god_setup(result, 2, god, false, new_conditions, None);
                }
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
    if change_unique_class(unit, result, mode, gender, equipped, true) { return;  } // Check if Unique Class
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
        if let Some(pos) = ENGAGE_PREFIX.iter().position(|w| result.body_anims.iter().any(|act| str_contains(act, w))) {
            if pos != 1 && pos != 17 && job.jid.contains("紋章士_") {
                result.ride_dress_model = "null".into();
                result.ride_model = "null".into();
                result.ride_anim = None;
            }
            match pos { 
                1 => { // Remove Mount Sigurd   on not horse / not sword/lance
                    if kind > 2 || !is_mounted(act_type ) || (act_type != 0 && act_type != 11 && act_type != 14 ) {
                        result.ride_dress_model = "null".into();
                        result.ride_model = "null".into();
                        result.ride_anim = None;
                        body = emblem_generic;
                    }
                }
                17 => { // Remove Mount Camilla for not tome / axe and not wyvern 
                    if ( kind != 3 && kind != 6 ) || !is_mounted(act_type ) || (act_type != 4 && act_type != 12) {
                        result.ride_dress_model = "null".into();
                        result.ride_model = "null".into();
                        result.ride_anim = None;
                        body = emblem_generic;
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

// For Units somehow in "Emblem" Classes
/* 
pub fn assign_emblem_animations(result: &mut AssetTableResult, job: &JobData, female: bool, kind: i32) {
    let gender_str = if female { "F" } else { "M" };
    let emblem_generic = if mode == 2 { concat_string!("Enb0A", gender_str, "-",  WEP_PRE[kind as usize], "1_c000_N") } else { format!("UAS_Enb0A{}", gender_str) };
    if ( job.unit_icon_id_m.is_some() && female ) || ( job.unit_icon_id_f.is_some() && !female ) {  // Wrong Gender 
        result.ride_dress_model = "null".into();
        result.ride_model = "null".into();
        result.ride_anim = None;
        result.body_anims.add( Il2CppString::new_static(emblem_generic) );
        return;
    }
    let body;
    if let Some(pos) = EMBLEM_ASSET.iter().position(|&s| job.jid.contains(s)) {
        let act_prex = concat_string!( crate::enums::ENGAGE_PREFIX[pos], "0", gender_str);
        match pos { 
            1 => { // Remove Mount Sigurd   on not horse / not sword/lance
                if kind <= 2 {
                    result.ride_dress_model = "null".into();
                    result.ride_model = "null".into();
                    result.ride_anim = None;
                    body = emblem_generic;
                }
            }
            17 => { // Remove Mount Camilla for not tome / axe and not wyvern 
                if ( kind != 3 && kind != 6 ) || !is_mounted(act_type ) || (act_type != 4 && act_type != 12) {
                    result.ride_dress_model = "null".into();
                    result.ride_model = "null".into();
                    result.ride_anim = None;
                    body = emblem_generic;
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
*/