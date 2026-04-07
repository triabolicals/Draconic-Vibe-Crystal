use engage::unit::UnitPool;
use outfit_core::{get_outfit_data, UnitAssetMenuData};
use super::*;
use crate::{utils, DVCVariables};
use accessory::change_accessory;
use crate::assets::dress::*;
use crate::config::DVCFlags;
use crate::randomizer::data::{GameData, RandomizedGameData};
use crate::randomizer::{Randomizer};
use crate::randomizer::names::get_emblem_person;
use crate::randomizer::person::is_playable_person;

const LUEUR_DEMO_PIDS: [&str; 5] = ["M024_リュール", "PID_青リュール_女性", "PID_青リュール_男性", "PID_デモ用_神竜王リュール_女性", "PID_デモ用_神竜王リュール_男性"];
fn emblem_asset_rand(result: &mut AssetTableResult, mode: i32, god: &GodData, is_dark: bool) {
    let emblem_appearance = DVCVariables::EmblemAppearance.get_value();
    let name = DVCFlags::GodNames.get_value();
    let rng = utils::create_rng(god.parent.hash, 1);
    let female = if god.parent.index == 13 { GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 } else { god.female == 1 };
    let menu_data = UnitAssetMenuData::get();
    let hash =
        if name { rng.get_value(100000) }
        else {
            rng.get_value(100000);
            rng.get_value(200000)
        };
    if let Some(search) = SEARCH_LIST.get() {
        if emblem_appearance & 1 != 0 {
            let outfit = search.get_random_outfit(mode, hash as u32, female);
            if mode == 2 { result.dress_model = outfit; }
            else { result.body_model = outfit; }
        }
    }
    if emblem_appearance & 2 != 0 {
        for x in [0, 1, 4, 5, 6, 7] {
            let value2 = rng.value();
            result.unity_colors[x].r = ( value2 & 255 ) as f32 / 255.0;
            result.unity_colors[x].g = (( value2 >> 4 ) & 255 ) as f32 / 255.0;
            result.unity_colors[x].b = (( value2 >> 8 ) & 255 ) as f32 / 255.0;
        }
    }
    random_body_scale(result, Some(hash), false);
    if menu_data.is_preview && menu_data.god_mode {
        menu_data.preview.preview_data.set_result(result, mode, false, false);
    }
    else { UnitAssetMenuData::set_god_assets(result, mode, god, is_dark); }
    /*
    if crate::DeploymentConfig::get().debug {
        if mode == 2 { println!("God: {} Body / Dress: {} / {}",  Mess::get(god.mid), result.body_model ,result.dress_model); }
        outfit_core::print_asset_table_result(result, mode);
    }

     */
}

#[skyline::hook(offset=0x01bb2d80)]
pub fn asset_table_result_god_setup(
    this: &mut AssetTableResult,
    mode: i32,
    god_data: Option<&GodData>,
    is_darkness: bool,
    conditions: &mut Array<&'static Il2CppString>,
    method_info: OptionalMethod) -> &'static mut AssetTableResult
{
    if god_data.is_none() { return call_original!(this, mode, god_data, is_darkness, conditions, method_info); }
    let god = god_data.unwrap();
    let is_lueur = god.gid.str_contains("リュール");
    let gid = god.gid.to_string();
    // Swapping Emblem appearance to Playable Characters appearance
    if DVCFlags::GodNames.get_value() {
        if let Some(person) = get_emblem_person(god.mid){
            let rng = Random::get_system();
            let is_engaging = conditions.iter_mut().any(|str| str.str_contains("エンゲージ開始"));
            let old_result = call_original!(this, mode, god_data, is_darkness, conditions, method_info);
            let demo_anim = old_result.demo_anims.map(|v| v.to_string());
            let voice =
                if is_lueur { if DVCVariables::is_lueur_female() { Some("PlayerF".into()) } else { Some("PlayerM".into()) } }
                else { old_result.sound.voice.clone() };
            
            let item = crate::randomizer::job::get_weapon_for_asset_table(person.get_job().unwrap());
            let old_anim = old_result.body_anims.iter().last().map(|s|s.to_string());

            let result = this.setup_for_person_job_item(mode, Some(person), person.get_job(), item, conditions);
            if is_darkness || gid.contains("E0") || (gid.contains("M0") && !gid.contains("M002")){
                result.unity_colors[0].r = 0.69;
                result.unity_colors[0].g = 0.19;
                result.unity_colors[0].b = 0.19;
                let masks = [60, 70, 40, 70];
                for x in 0..4 {
                    let value = masks[x] as f32 / 255.0;
                    result.unity_colors[x+4].r = value;
                    result.unity_colors[x+4].g = value;
                    result.unity_colors[x+4].b = value;
                }
            }
            change_accessory(result.accessory_list, format!("uAcc_Eff_EmblemAura-0{}-00", if is_darkness { "2" } else { "1" }).as_str(), "c_trans");
            result.sound.voice = voice;
            result.demo_anims = demo_anim.map(|v| v.into());
            if is_engaging && mode == 2 {
                if let Some(search) = SEARCH_LIST.get() {
                    let gen_str = if person.get_dress_gender() == Gender::Male { "AM" } else { "AF"};
                    let anims: Vec<_> = search.engaging.iter()
                        .filter_map(|x| AnimSetDB::try_index_get(*x).filter(|x| x.name.str_contains(gen_str)))
                        .collect();
                    if let Some(anim) = anims.get(rng.get_value(anims.len() as i32) as usize) {
                        result.body_anims.clear();
                        result.body_anims.add(format!("Com0{}-No1_c000_N",gen_str).into());
                        if let Some(old_anim) = old_anim { result.body_anims.add(old_anim.into()); }
                        result.body_anims.add(anim.name);
                        result.body_anim = Some(anim.name);
                    }
                }
            }
            result.sound.footstep = Some("Emblem".into());
            emblem_asset_rand(result, mode, god, is_darkness);
            return result;
        }
        else if !is_lueur { return call_original!(this, mode, god_data, is_darkness, conditions, method_info); }
    }
    if is_lueur {
        let mut result: &mut AssetTableResult = call_original!(this, mode, god_data, is_darkness, conditions, method_info);
        let aura = format!("uAcc_Eff_EmblemAura-0{}-00", if is_darkness { "2" } else { "1" });
        if let Some(person) = god.link.and_then(|pid|PersonData::get(pid)).filter(|x| x.parent.index > 1) {
            let db =  get_outfit_data();
            if let Some(appearance) = UnitPool::get_hero(false).and_then(|unit| db.dress.get_personal_dress(unit)){
                appearance.apply_appearance(result, mode, false, None, &db.hashes, true);
                /*
                if let Some(hair) = outfit_core::get_hair(result, mode).and_then(|hair| db.hashes.get_engaged_hair(hair)) {
                    outfit_core::apply_hair(&hair.to_string(), result);
                }
                 */
            }
            if is_darkness { lueur_fell_child_hair(result); } else { lueur_god_hair(result) }
            result.body_anims.clear();
            if person.get_dress_gender() == Gender::Female {
                if mode == 2 {
                    result.body_anims.add("End0AF-No1_c000_N".into());
                    result.body_anims.add("Luc0AF-Sw1_c584_N".into());
                }
                else { result.body_anim = Some("UAS_Enb0AF".into()); }
            }
            else {
                if mode == 2 {
                    result.body_anims.add("End0AM-No1_c000_N".into());
                    result.body_anims.add("Mar0AM-Sw1_c530_N".into());
                }
                else { result.body_anim = Some("UAS_Enb0AM".into()); }
            }
        }
        else {
            let female = DVCVariables::is_lueur_female();
            let suffix = if female { ("05", "F") } else { ("00", "M") };
            let suffix_end = if is_darkness { "2" } else { "3" };
            if mode == 2 {
                result.dress_model = format!("uBody_Drg0A{}_c{}{}", suffix.1, suffix.0, suffix_end).into();
                result.head_model = format!("uHead_c{}{}", suffix.0, suffix_end).into();
                result.body_anims.clear();
                if female {
                    result.hair_model = "uHair_null".into();
                    change_accessory(result.accessory_list, format!("uAcc_spine2_Hair05{}", suffix_end).as_str(), "c_spine1_jnt");
                    change_accessory(result.accessory_list, format!("uAcc_head_Band05{}", suffix_end).as_str(), "c_head_loc");
                    result.body_anims.add("End0AF-No1_c000_N".into());
                    result.body_anims.add("Luc0AF-Sw1_c584_N".into());
                    result.info_anims = Some("AOC_Info_c051".into());
                    result.talk_anims = Some("AOC_Talk_c051".into());
                }
                else {
                    result.hair_model = format!("uHair_h{}{}", suffix.0, suffix_end).into();
                    change_accessory(result.accessory_list, "null", "c_spine1_jnt");
                    result.body_anims.add("End0AM-No1_c000_N".into());
                    result.body_anims.add("Mar0AM-Sw1_c530_N".into());
                    result.info_anims = Some("AOC_Info_c001".into());
                    result.talk_anims = Some("AOC_Talk_c001".into());
                }

                let aura = format!("uAcc_Eff_EmblemAura-0{}-00", if is_darkness { "2" } else { "1" });
                change_accessory(result.accessory_list, aura.as_str(), "c_trans");
            }
            else {
                result.body_model = format!("oBody_Drg0A{}_c{}{}", suffix.1, suffix.0, suffix_end).into();
                result.head_model = format!("oHair_h{}{}", suffix.0, suffix_end).into();
            }
            change_accessory(result.accessory_list, "null", "c_spine2_jnt");
            change_accessory(result.accessory_list, "null", "c_hip_jnt");
        }
        change_accessory(result.accessory_list, aura.as_str(), "c_trans");
        change_accessory(result.accessory_list, "null", "c_spine2_jnt");
        change_accessory(result.accessory_list, "null", "c_hip_jnt");
        emblem_asset_rand(result, mode, god, is_darkness);
        return result;
    }
    else if gid.contains("ルフレ") {   // Robin
        if let Some(con) = conditions.iter_mut().find(|str| str.to_string() == "エンゲージ技") { *con = "".into(); }
        let pid = if is_darkness { "PID_闇ルフレ"} else { "PID_ルフレ" };
        let result = this.setup_for_person(mode, PersonData::get(pid), conditions);
        emblem_asset_rand(result, mode, god, is_darkness);
        return result;
    }
    if is_darkness {
        if god.flag.value & 32 != 0 {
            let new_god = GodData::get(god.gid.to_string().replace("GID_", "GID_E006_敵"));
            if new_god.is_some() {
                let result = call_original!(this, mode, new_god, true, conditions, method_info);
                emblem_asset_rand(result, mode, god, is_darkness);
                return result;
            }
        }
        let result = call_original!(this, mode, Some(god), true, conditions, method_info);
        emblem_asset_rand(result, mode, god, is_darkness);
        return result;
    }
    let hash = god_data.unwrap().parent.hash;
    let gid = god.gid.to_string();
    if gid.contains("GID_相手") {
        let opp_god = GodData::get(gid.replace("_相手", "_")).or_else(|| god_data);
        return asset_table_result_god_setup(this, mode, opp_god, true, conditions, method_info);
    }
    else if let Some(enemy_emblem) = GameData::get().emblem_pool.enemy_emblem.iter().find(|&x| x.emblem_data.hash == hash) {
        let emblem_index = enemy_emblem.emblem_index;
        if let Some(replace_god) = DVCVariables::get_god_from_index(emblem_index as i32, true) {
            let is_m002 = gid == "GID_M002_シグルド";
            let new_emblem = GameData::get_playable_emblem_hashes().iter().position(|&hash| hash == replace_god.parent.hash).unwrap();
            let emblem = 
                if new_emblem < 12 || new_emblem >= 19 || is_m002 { replace_god }
                else { GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[new_emblem])).unwrap() };
            return call_original!(this, mode, Some(emblem), !is_m002, conditions, method_info);
        }
    }
    let result = call_original!(this, mode, god_data, is_darkness, conditions, method_info);
    emblem_asset_rand(result, mode, god, is_darkness);
    result
}
/*
#[skyline::hook(offset=0x01bb7ca0)]
pub fn asset_table_result_get_preset_name(name: &Il2CppString, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let mut result = call_original!(name, method_info);
    let asset_table = AssetTable::get_list().unwrap();
    let pre_name = name.to_string();

    if pre_name == "エンゲ技/【エイリーク】ツインストリーム/エフラム" { // Replacing Ephraim in Twin Strike
        for x in 1..1000 {
            if let Some(con) = &asset_table[x].conditions {
                let status = ASSET_STATUS.try_read().unwrap();
                let new_conditions = Array::<&Il2CppString>::new_specific( con.get_class(), 1).unwrap();
                let eirika = status.engage_atk_eirika;
                let is_dark = status.darkness;
                let engage_type = status.engage_atk_type;
                if eirika > 21 { break; }
                new_conditions[0] = "".into();
                let link_god = GodData::try_get_hash(status.link_god);
                if link_god.is_some_and(|d| d.female != 1) && engage_type == 2 { asset_table_result_god_setup(result, 2, link_god, is_dark, new_conditions, None); }
                else {               
                    let partner = combo_engage_attack_male_emblem_index(eirika as usize, false);
                    match partner {
                        23|50 => { asset_table_result_god_setup(result, 2, GodData::get("GID_エフラム"), is_dark, new_conditions, None); } // Ephraim
                        19 => { // Male Alear
                            new_conditions[0] = "男装".into();
                            asset_table_result_god_setup(result, 2, GodData::get("GID_リュール"), is_dark, new_conditions, None);
                            result.sound.voice = Some("PlayerM".into());
                        }
                        22 => { //
                            let pid = if is_dark { "PID_闇ルフレ"} else { "PID_ルフレ" };
                            result.setup_for_person(2, PersonData::get(pid), new_conditions);
                        }
                        _ => {
                            let gid = format!("GID_{}", EMBLEM_ASSET[partner]);
                            result = asset_table_result_god_setup(result, 2, GodData::get(gid), is_dark, new_conditions, None); 
                        }
                    }
                }
                // Animation Replacement ;
                dress::random_body_scale(result, None, false);
                result.body_anims.clear();
                result.body_anims.add("Eir1AM-Lc1_c536_N".into());
                result.right_hand = "uWep_Lc19".into(); 
                result.left_hand = "null".into(); 
                return result;
            }
        }
    }
    result
}
 */


/*
println!("{} is Engage Attacking: {} hash: {}", Mess::get_name(unit.person.pid), Mess::get(engage_attack.name.unwrap()), engage_attack.parent.hash);
        let engage_sid = engage_attack.sid.to_string();


         // Replace Generic with random character voices

        let mut old_engage = EATK_ACT.iter().position(|prefix|{ result.body_anims.iter().any(|act| act.to_string().contains(prefix)) }).unwrap_or(50);

        if old_engage == 22 {
            lueur_engage_atk(result, unit, flags);
            return;
        }
        else if old_engage == 50 {
            if let Some(god) = unit.god_link.or(unit.god_unit){
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
        let emblem_index = if let Some(pos) = EMBLEM_ASSET.iter().position(|god| engage_sid.contains(god)) { pos }
            else if engage_sid.contains("三級長エンゲージ技＋") { 20 }
            else if engage_sid.contains("三級長エンゲージ") { 12 }
            else { 50 };
        let engage_atk = &SEARCH_LIST.get().unwrap().engage_atks;
        if emblem_index == 50 { 
            if let Some(engage_atk_data) = engage_atk.iter().find(|x| x.original_god_index == 50 && x.is_engage_atk(engage_attack)) {
                engage_atk_result_clear(result, equipped);
                if is_tiki_engage(result) { SEARCH_LIST.get().unwrap().replace_with_god(result, 2, 13, false); }
                engage_atk_data.apply(result, unit, gender_con);
            }
            return;
        }
        engage_atk_result_clear(result, equipped);
        let enemy_tiki = unit.god_unit.is_some_and(|g_unit| g_unit.data.gid.to_string().contains("敵チキ") || ( g_unit.data.mid.to_string().contains("Tiki") && !g_unit.data.gid.to_string().contains("チキ") ));
        match (emblem_index, old_engage) {
            (13, 13) => {    // No Change
                if result.body_model.to_string() != "uRig_Tik1AT" {
                    tiki_engage(result, unit, 2);
                    result.body_anims.add("Tik1AT-Mg1_c000_M".into());
                }
                change_result_colors_by_unit(unit, result);
                return;
            }
            (_, 13) => {    // Other Tiki
                if !unit.god_unit.is_some_and(|god| god.data.gid.to_string().contains("敵チキ")) {
                    result.dress_model = "uBody_Tik0AF_c560".into();
                    result.body_model = "uRig_GodF1".into();
                    result.head_model = "uHead_c560".into();
                    result.hair_model = "uHair_null".into();
                    add_accessory_to_list(result.accessory_list, "uAcc_spine2_Hair560", "c_spine1_jnt");
                    add_accessory_to_list(result.accessory_list, "uAcc_head_Tiara560", "c_head_loc");
                    gender_con = SEARCH_LIST.get().unwrap().get_gender_condition(2);
                }
*/
pub fn engage_animation_mode_1(this: &mut AssetTableResult, engage_atk_index: i32, gender: i32) {
    let gen_str = if gender == 1 { "M" } else { "F" };
    match engage_atk_index {
        0..13|14..19 => { this.body_anim = Some(format!("UAS_Mar1A{}", gen_str).into()); }
        13 => { return; }
        21 => { this.body_anim =  Some(format!("UAS_Ler1A{}", gen_str).into()); }
        _ => {this.body_anim =  Some(format!("UAS_Mar1A{}", gen_str).into()); }
    }
}

#[skyline::hook(offset=0x01bb4180)]
pub fn asset_table_robin_hook(
    this: &mut AssetTableResult, 
    mode: i32, 
    person: &mut PersonData, 
    conditions: &mut Array<&'static Il2CppString>, 
    method_info: OptionalMethod) -> &'static mut AssetTableResult 
{
    let pid = person.pid.to_string();
    if is_playable_person(person) {
        let result = call_original!(this, mode, person, conditions, method_info);
        if let Some(data) = UnitAssetMenuData::get_by_person_data(person.parent.hash, true) {
            data.set_result(result, mode, false, false);
            return result;
        }
    }
    if mode == 2 && person.gender != 0 && DVCFlags::RandomBossesNPCs.get_value() && GameUserData::get_sequence() > 0 {
        let db = get_outfit_data();
        let original_result = call_original!(this, mode, person, conditions, method_info);
        outfit_core::print_asset_table_result(original_result, 2);
        if let Some(appearance) = RandomizedGameData::get_read().person_appearance.get_person_appearance(person).as_ref(){
            appearance.apply_appearance(original_result, 2, false, None, &db.hashes, true);
            random_body_scale(original_result, None, false);
            println!("RandomNPC");
            outfit_core::print_asset_table_result(original_result, 2);
            return original_result;
        }
        else if person.flag.value & 128 == 0 && person.gender != 0 && ( person.get_job().is_none_or(|v| v.parent.hash == 499211320 || v.parent.index == 0)){
            let female = (person.gender == 2 && person.flag.value & 32 == 0) || (person.gender == 1 && person.flag.value & 32 != 0);

            let s: Vec<_> = db.dress.personal.iter().filter(|x| x.is_female == female).collect();
            let system = Random::get_system();
            if let Some(v) = s.get_random_element(system){
                v.apply(original_result, 2, false, None, &db.hashes);
                let generic_mode =  DVCVariables::GenericAppearance.get_value();
                if generic_mode & 2 != 0 { change_color_by_rng(original_result, system); }
            }
            println!("RandomGeneric");
            random_body_scale(original_result, None, false);
            return original_result;
        }
    }
    if !DVCVariables::is_main_menu() && DVCVariables::is_changed_recruitment_order(false){
        if let Some(pos) = LUEUR_DEMO_PIDS.iter().position(|x| pid.contains(*x)) {
            if let Some(new_person) = PersonData::get_mut(DVCVariables::get_dvc_person(0, false)).filter(|p| p.parent.index > 1) {
                let result = call_original!(this, mode, new_person, conditions, method_info);
                match pos {
                    0 => { lueur_fell_child_hair(result); }
                    1|2 => { lueur_god_hair(result); }
                    _ => {}
                }
                random_body_scale(result, Some(new_person.parent.hash), false);
                return result;
            }
        }
    }
    let result = call_original!(this, mode, person, conditions, method_info);
    random_body_scale(result, Some(person.parent.hash), false);
    result
}

pub fn tiki_engage(result: &mut AssetTableResult, unit: &Unit, mode: i32) {
    if mode == 2 {
        result.body_model = "uRig_Tik1AT".into();
        result.dress_model = "uBody_Tik1AT_c000".into();
        result.head_model = "null".into();
        result.hair_model = "null".into();
        result.info_anims = Some("AOC_Info_c745".into());
        result.talk_anims = Some("AOC_Talk_c745".into());
        ["c_spine1_jnt", "c_spine2_jnt", "c_hip_jnt", "l_arm3_jnt", "r_arm3_jnt", "l_leg3_jnt", "r_leg3_jnt"]
            .iter().for_each(|locator|  change_accessory(result.accessory_list, "null", *locator));
        result.body_anims.add(Il2CppString::new_static("Ent0AT-Mg1_c000_N"));
        result.body_anims.add(Il2CppString::new_static("Ent0AT-Ft1_c000_N"));
        result.body_anims.add(Il2CppString::new_static("Ent0AT-Ft2_c000_N"));
    }
    else {
        result.body_model = "oBody_Tik1AT_c000".into();
        result.head_model = "oHair_null".into();
        ["c_spine1_jnt", "c_spine2_jnt", "c_hip_jnt", "l_arm3_jnt", "r_arm3_jnt", "l_leg3_jnt", "r_leg3_jnt"]
            .iter().for_each(|locator|  change_accessory(result.accessory_list, "null", *locator));
        result.scale_stuff[18] = 0.50;
        result.scale_stuff[16] = 1.0;
        result.body_anim = Some("UAS_Ent0AT".into());
    }
    change_result_colors_by_unit(unit, result);
}

pub fn get_emblem_attack_index(unit: &Unit) -> usize {
    if let Some(engage_attack) = unit.get_engage_attack()  {
        let sid = engage_attack.sid.to_string();
        if let Some(pos) = EMBLEM_ASSET.iter().position(|god| sid.contains(god)) { pos }
        else if sid.contains("三級長エンゲージ技＋") { 20 }
        else if sid.contains("三級長エンゲージ") { 12 }
        else { 50 }
    }
    else { 50 }
}

pub fn random_engage_voice(result: &mut AssetTableResult) {
    if result.sound.voice.is_none() || result.sound.voice.is_some_and(|str|{
        let str1 = str.to_string();
        str1.contains("_MOB_Enemy") || str1.contains("ENEMY") })
    {
        result.sound.voice = Some(get_random_engage_voice().into());
    }
}

pub fn has_engage_decide(person: &str) -> bool {
    MPIDS.iter().any(|x| x.contains(person)) || person.contains("Player") || person.contains("DLC_4")
}
pub fn get_random_engage_voice() -> &'static str {
    let rng = Random::get_system();
    let index = rng.get_value(40) as usize + 1;
    match index {
        36 => { "DLC_42"}
        37 => { "DLC_43_2"}
        38 => { "DLC_44"}
        39 => { "DLC_45"}
        40 => { "DLC_46"}
        _ =>  { &MPIDS[index][5..] }
    }
}

