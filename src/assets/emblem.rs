use super::*;
use crate::assets::accessory::add_accessory_to_list;
use crate::{utils, DVCVariables};
use accessory::change_accessory;
use conditions::{add_condition, remove_condition};
use data::search::search_by_2_keys;
use crate::assets::data::search::{search_by_iid, search_by_key};
use crate::assets::dress::change_result_colors_by_unit;
use crate::config::DVCFlags;
use crate::randomizer::names::get_emblem_person;
use crate::randomizer::person::ENEMY_PERSONS;

const DEMOS_PIDS: [&str; 6] = [
    "PID_デモ用_竜石なし_ラファール", "PID_青リュール_女性", "PID_青リュール_男性",
    "PID_デモ用_神竜王リュール", "PID_デモ用_神竜王リュール_女性", "PID_デモ用_神竜王リュール_男性"
];

const MALE_EMBLEMS: [usize; 9] = [0, 1, 4, 5, 8, 9, 14, 16, 18];
const FEMALE_EMBLEMS: [usize; 10] = [2, 3, 6, 7, 10, 11, 12, 13, 15, 17];
const THREE_HOUSE_ACTS: [&str; 4] = ["Thr2AF-Ax1_c563_N", "Thr2AM-Lc1_c514_N", "Thr2AM-Bw1_c515_N", "Thr2AM-Sw1_c535_N"];

fn houses_unite_plus_act(result: &mut AssetTableResult, kind: i32) {
    if kind >= 4 || kind < 0 { return; }
    result.body_anims.clear();
    result.body_anims.add(Il2CppString::new_static(THREE_HOUSE_ACTS[kind as usize]));
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

fn emblem_asset_rand(result: &mut AssetTableResult, mode: i32, god: &GodData) {
    let emblem_appearance = DVCVariables::get_emblem_appearance();

    let name = DVCVariables::get_flag(DVCFlags::GodNames, false);
    let rng = utils::create_rng(god.parent.hash, 1);
    let female = if god.parent.index == 13 { GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 } else { god.female == 1 };
    let hash =
        if name { rng.get_value(100000) }
        else {
            rng.get_value(100000);
            rng.get_value(200000)
        };
    if let Some(search) = SEARCH_LIST.get() {
        if emblem_appearance & 1 != 0 {
            let outfit = search.get_random_outfit(mode, hash as u32, female);
            if mode == 2 { result.dress_model = outfit.into(); }
            else { result.body_model = outfit.into(); }
        }
        else {
            search.check_dress_body(result, None, hash as u32, 2, female);
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
    bust_modifier_randomization(result, hash);
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
    if god_data.is_none() || mode == 20 {
        if mode == 20 { return call_original!(this, 2, god_data, is_darkness, conditions, method_info); }
        let mode = if mode > 10 { mode - 10 } else { mode };
        return call_original!(this, mode, god_data, is_darkness, conditions, method_info);
    }
    let mode2 = if mode > 10 { mode - 10 } else { mode };
    let god = god_data.unwrap();
    let is_lueur = god.gid.str_contains("リュール");
    let con_flags = ConditionFlags::get_from_conditions(conditions);

    // Houses Unite+ (協力エンゲージ技) Emblem Replacements
    if conditions.iter().any(|str| str.to_string() == "協力エンゲージ技") {
        let status = ASSET_STATUS.try_read().unwrap();
        let mut three_houses = status.engage_atk_3h as usize;
        let is_darkness = status.darkness;
        let engage_type = status.engage_atk_type;

        if three_houses == 21 { three_houses = 15 } else if three_houses > 19 { three_houses = 12 };
        let gid = god.gid.to_string();
        let kind =
            if gid.contains("エーデルガルト") { 0 }
            else if gid.contains("ディミトリ") { 1 }
            else if gid.contains("クロード") { 2 }
            else { 3 };

        let link_god = GodData::try_get_hash(status.link_god);
        let result = call_original!(this, mode2, god_data, is_darkness, conditions, method_info);

        if let Some(con) = conditions.iter_mut().find(|con| con.to_string() == "協力エンゲージ技")  { *con = "".into();  }
        let replace_male = link_god.is_some_and(|f| f.female != 1) && engage_type == 2;
        let replace_female = link_god.is_some_and(|f| f.female == 1) && engage_type == 2;

        let god = 
        if replace_female && kind == 0 || replace_male && kind == 1 { link_god  }
        else if kind == 0 && FEMALE_EMBLEMS.iter().any(|&female| female == three_houses) && three_houses != 12 { GodData::get(EMBLEM_GIDS[three_houses]) }
        else if kind == 1 && MALE_EMBLEMS.iter().any(|&male | male == three_houses) && three_houses != 12 { GodData::get(EMBLEM_GIDS[three_houses]) }
        else { houses_unite_partner(three_houses, kind,  Random::get_system()) };

        asset_table_result_god_setup(this, mode2, god, is_darkness, conditions, None);
        houses_unite_plus_act(result, kind);
        remove_mounts_accs(result);
        return result;
    }
    // Swapping Emblem appearance to Playable Characters appearance
    if DVCVariables::get_flag(DVCFlags::GodNames, false) {
        if let Some(person) = get_emblem_person(god.mid) {
            let rng = Random::get_system();
            let is_engaging = conditions.iter_mut().any(|str| str.str_contains("エンゲージ開始"));
            let old_result = call_original!(this, mode2, god_data, is_darkness, conditions, method_info);
            let voice =
                if is_lueur {
                    if DVCVariables::is_lueur_female() { Some("PlayerF".into()) } else { Some("PlayerM".into()) }
                }
                else { old_result.sound.voice.clone() };

            let item = crate::randomizer::job::get_weapon_for_asset_table(person.get_job().unwrap());
            let old_anim = old_result.body_anims.iter().last().map(|s|s.to_string());

            let result = this.setup_for_person_job_item(mode2, Some(person), person.get_job(), item, conditions);

            if is_darkness || mode > 10 {
                add_accessory_to_list(result.accessory_list, "uAcc_Eff_EmblemAura-02-00", "c_trans");
            }
            else {
                add_accessory_to_list(result.accessory_list, "uAcc_Eff_EmblemAura-01-00", "c_trans");
            }
            result.sound.voice = voice;
            if let Some(search) = SEARCH_LIST.get() {
                if is_engaging && mode2 == 2 {
                    let gen_str = if unsafe { get_dress_gender(person, None) } == 1 { "AM" } else { "AF"};
                    let anims: Vec<_> = search.engaging.iter()
                        .filter_map(|x| AnimSetDB::try_index_get(*x).filter(|x| x.name.str_contains(gen_str)))
                        .collect();
                    if let Some(anim) = anims.get(rng.get_value(anims.len() as i32) as usize) {
                        result.body_anims.clear();
                        result.body_anims.add(format!("Com0{}-No1_c000_N",gen_str).into());
                        if let Some(old_anim) = old_anim { result.body_anims.add(old_anim.into()); }
                        result.body_anims.add(anim.name);
                        remove_mounts_accs(result);
                        result.body_anim = Some(anim.name);
                    }
                }
                if DVCVariables::get_assets() & 1 != 0 { edit_asset_weapon(result, true, mode2, item); }
                else if let Some(asset) = item.and_then(|item| search_by_iid(item.iid,2)){
                    result.commit_asset_table(asset);
                }
            }
            result.sound.footstep = Some("Emblem".into());
            emblem_asset_rand(result, mode2, god);
            return result;
        }
        else if !is_lueur { return call_original!(this, mode2, god_data, is_darkness, conditions, method_info); }
    }
    // Chapter 22 Emblem
    if mode > 10 {
        println!("Is Chapter 22 Emblem Unit: {}", god.mid);
        if god.flag.value & 32 != 0 {
            let new_god = GodData::get(god.gid.to_string().replace("GID_", "GID_E006_敵"));
            if new_god.is_some() {
                let result = call_original!(this, mode2, new_god, true, conditions, method_info);
                emblem_asset_rand(result, mode2, god);
                return result;
            }
        }
        let result = call_original!(this, mode2, Some(god), true, conditions, method_info);
        emblem_asset_rand(result, mode2, god);
        return result;
    }
    if is_lueur {   // Emblem Alear
        let result = call_original!(this,  mode2 ,god_data, is_darkness, conditions, method_info);
        let gen =
            if conditions.iter_mut().any(|str| str.str_contains("女装")) {  "女装" }
            else if conditions.iter_mut().any(|str| str.str_contains("男装")) { "男装" }
            else if DVCVariables::is_lueur_female()  { "女装" } else { "男装" };

        remove_condition("男装");
        remove_condition("女装");
        add_condition(gen);
        result.body_anims.clear();
        if is_darkness {
            result.commit_asset_table(search_by_2_keys(0, "MPID_Lueur", gen).unwrap());
            result.commit_asset_table(search_by_2_keys(mode2, "MPID_Lueur", gen).unwrap());
            if let Some(entry) = search_by_2_keys(mode2, "JID_邪竜ノ子", gen) {
                result.commit_asset_table(entry);
                result.commit_mode(mode2);
                result.replace(mode2);
                if gen == "男装" { result.info_anims = Some("AOC_Info_c001".into()); }
                else { result.info_anims = Some("AOC_Info_c051".into()); }
                change_accessory(result.accessory_list, "uAcc_Eff_EmblemAura-02-00", "c_trans");
            }
        }
        else {
            result.commit_mode(mode2);
            if mode2 == 2 {
                result.right_hand = "uWep_Sw09-Sw".into();
                if gen == "男装" {
                    result.head_model = "uHead_c003".into();
                    result.hair_model = "uHair_h003".into();
                    result.info_anims = Some("AOC_Info_c001".into());
                    result.body_anims.add(Il2CppString::new_static("End0AM-No1_c000_N"));
                    result.body_anims.add(Il2CppString::new_static("Mar0AM-Sw1_c530_N"));
                }
                else {
                    result.head_model = "uHead_c053".into();
                    result.info_anims = Some("AOC_Info_c051".into());
                    result.body_anims.add(Il2CppString::new_static("End0AF-No1_c000_N"));
                    result.body_anims.add(Il2CppString::new_static("Luc0AF-Sw1_c584_N"));
                    change_accessory(result.accessory_list, "uAcc_spine2_Hair053", "c_spine1_jnt");
                    change_accessory(result.accessory_list, "uAcc_head_Band053", "c_head_loc");
                }
                change_accessory(result.accessory_list, "uAcc_Eff_EmblemAura-01-00", "c_trans");
            }
            if con_flags.contains(ConditionFlags::UnitInfo) {
                if god.parent.index == 13 {
                    if gen == "男装" { result.sound.voice = Some("Lueur1".into()); }
                    else { result.sound.voice = Some("Lueur2".into()); }
                }
            }
        }
        emblem_asset_rand(result, mode2, god);
        return result;
    }
    else if god.gid.str_contains("ルフレ") {   // Robin
        if let Some(con) = conditions.iter_mut()
            .find(|str| str.to_string() == "エンゲージ技") { *con = "".into(); }

        let pid = if is_darkness { "PID_闇ルフレ"} else { "PID_ルフレ" };
        let result = this.setup_for_person(mode2, PersonData::get(pid), conditions);
        emblem_asset_rand(result, mode2, god);
        return result;
    }


    let index = god_data.unwrap().parent.index;
    let gid = god.gid.to_string();
    if gid.contains("GID_相手") {
        let opp_god = GodData::get(gid.replace("_相手", "_")).or_else(|| god_data);
        return asset_table_result_god_setup(this, mode2, opp_god, true, conditions, method_info);
    }
    else if let Some(is_enemy_emblem) =  crate::randomizer::emblem::enemy::ENEMY_EMBLEMS.get().unwrap().iter().find(|&x| x.0 == index) {
        let emblem_index = is_enemy_emblem.1;
        if let Some(replace_god) = DVCVariables::get_god_from_index(emblem_index, true) {
            let is_m002 = gid == "GID_M002_シグルド";
            let new_emblem = crate::randomizer::emblem::EMBLEM_LIST.get().unwrap().iter().position(|&hash| hash == replace_god.parent.hash).unwrap();
            let emblem = 
                if new_emblem < 12 || new_emblem >= 19 || is_m002 { replace_god }
                else { GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[new_emblem])).unwrap() };
            return asset_table_result_god_setup(this, mode2, Some(emblem), !is_m002, conditions, method_info);
        }
    }
    let result = call_original!(this, mode2, god_data, is_darkness, conditions, method_info);
    emblem_asset_rand(result, mode2, god);
    result
}

pub fn bust_modifier_randomization(result: &mut AssetTableResult, hash: i32){
    let rng = Random::get_system();
    let value = CONFIG.lock().unwrap().misc_option_1;
    if value >= 4.75 {
        if hash != 0 {
            let rng = Random::instantiate().unwrap();
            rng.ctor(hash as u32);
            result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.035;
        }
        else {
            result.scale_stuff[9] = 1.0 + rng.get_value(50) as f32 * 0.035;
        }
    }
    else if value >= 0.09 {
        let range = 2.5 * value / 5.0;
        let var = value - range;
        if hash != 0 {
            let rng = Random::instantiate().unwrap();
            rng.ctor(hash as u32);
            result.scale_stuff[9] = range + rng.get_value(25) as f32 * var * 0.1;
        }
        else { result.scale_stuff[9] = range + rng.get_value(25)  as f32 * var * 0.1; }
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
                // Animation Replacement 
                remove_mounts_accs(result);
                bust_modifier_randomization(result, 0);
                result.body_anims.clear();
                result.body_anims.add(Il2CppString::new_static("Eir1AM-Lc1_c536_N"));
                result.right_hand = "uWep_Lc19".into(); 
                result.left_hand = "null".into(); 
                return result;
            }
        }
    }
    result
}

pub fn adjust_engage_attack_animation(result: &mut AssetTableResult, unit: &Unit, equipped: Option<&ItemData>, flags: ConditionFlags) {
    let engage_status = &mut ASSET_STATUS.try_write().unwrap();
    engage_status.reset_engage_atk();

    if let Some(engage_attack) = unit.get_engage_attack()  {
        // println!("{} is Engage Attacking: {} hash: {}", Mess::get_name(unit.person.pid), Mess::get(engage_attack.name.unwrap()), engage_attack.parent.hash);
        let engage_sid = engage_attack.sid.to_string();
        engage_status.engage_atk_type = get_engage_attack_source(unit);
        engage_status.unit = unit.ident;
        let _ = unit.god_link.or(unit.god_unit).map(|g_unit|{
            engage_status.darkness = g_unit.data.gid.str_contains("M0") || g_unit.data.gid.str_contains("E00");
            engage_status.link_unit1 = g_unit.parent_unit.map(|unit| unit.ident).unwrap_or(0);
            engage_status.link_unit2 = g_unit.child.map(|unit| unit.ident).unwrap_or(0);
            //engage_status.link_god = 
            engage_status.link_god = g_unit.data.get_link_god_data().map_or(0, |d| d.parent.hash);
        });

        let gender = if flags.contains(ConditionFlags::Female) { 2 } else { 1 };
        let gender_str = if flags.contains(ConditionFlags::Female) { "F" } else { "M" };
        let mut gender_con = SEARCH_LIST.get().unwrap().get_gender_condition(gender);
    // Replace Generic with random character voices
        random_engage_voice(result);
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
                    result.body_anims.add( Il2CppString::new_static("Tik1AT-Mg1_c000_M"));
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
            }
            (_, _) => { 
                if emblem_index == old_engage {
                    if enemy_tiki { gender_con = SEARCH_LIST.get().unwrap().get_gender_condition(2);  }
                    let _ = engage_atk.iter().find(|emblem| emblem.original_god_index == emblem_index as i32 ).map(|engage| engage.apply(result, unit, gender_con));
                    if enemy_tiki {
                        if old_engage == 1 { result.ride_dress_model = "uBody_Sig0BR_c538".into();  }
                        else if old_engage == 17 { result.ride_dress_model = "uBody_Cmi0DR_c568".into(); }
                    }
                    return;
                }
            }
        }
        match emblem_index {    //Marth, Roy, Leif, Lucina, Ike, Byleth, Dragon Blast
            13 => {
                engage_atk.iter().find(|emblem| emblem.original_god_index == 3).map(|engage| engage.apply(result, unit, gender_con)).unwrap();
                return;
            }
            11 => {  engage_status.engage_atk_eirika = old_engage as i32; }
            15 =>  {change_accessory(result.accessory_list, "uAcc_Event_SummonStoneB", "reserve4_loc"); }
            18 => { engage_status.engage_atk_chrom = old_engage as i32;  }
            19 => { 
                lueur_engage_atk(result, unit, flags);
                return;
            }
            20 => { 
                engage_status.engage_atk_3h = old_engage as i32; 
                engage_atk.iter().find(|emblem| emblem.original_god_index == 12 ).map(|engage| engage.apply(result, unit, gender_con));
                result.body_anims.clear();
                result.body_anims.add(Il2CppString::new_static(concat_string!("Thr2A", gender_str, "-Ax1_c000_N")));
                return;
            }
            _ => {}
        }
        if let Some(engage) = engage_atk.iter().find(|emblem| emblem.original_god_index == emblem_index as i32 ){
            engage.apply(result, unit, gender_con);
        }
    }
}
pub fn engage_animation_mode_1(this: &mut AssetTableResult, engage_atk_index: i32, gender: i32) {
    let gen_str = if gender == 1 { "M" } else { "F" };
    match engage_atk_index {
        0..13|14..19 => { this.body_anim = Some(concat_string!("UAS_", "Mar1A", gen_str).into()); }
        13 => { return; }
        21 => { this.body_anim = Some(concat_string!("UAS_", "Ler1A", gen_str).into()); }
        _ => {this.body_anim = Some(concat_string!("UAS_", "Mar1A", gen_str).into()); }
    }
}

#[skyline::hook(offset=0x01bb4180)]
pub fn asset_table_robin_hook(this: &mut AssetTableResult, mode: i32, person: &mut PersonData, conditions: &mut Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let pid = person.pid.to_string();
    // println!("Asset Person: {}", person.parent.index);
    if mode == 2 && pid == "PID_ルフレ" {
        let mut result = call_original!(this, mode, person, conditions, method_info);
        if let Some(con) = conditions.iter_mut().find(|con| con.to_string() == "エンゲージ技")  {
            *con = "".into();
            let status = ASSET_STATUS.try_read().unwrap();
            let robin = status.engage_atk_chrom as usize;
            let is_dark = status.darkness;
            let engage_type = status.engage_atk_type;
            if robin > 21 { return result; }
            let link_god = GodData::try_get_hash(status.link_god);
            if link_god.is_some_and(|d| d.female != 1) && engage_type == 2 { asset_table_result_god_setup(result, 2, link_god, is_dark, conditions, None); }
            else {
                let partner = combo_engage_attack_male_emblem_index(robin, true);
                match partner {
                    18|22|50 => {
                        bust_modifier_randomization(result, 0);
                        return result;
                    }
                    19 => {
                        result.dress_model = "uBody_Drg0AM_c003".into();
                        result.head_model = "uHead_c003".into();
                        result.hair_model = "uHair_h003".into();
                        result.sound.voice = Some("PlayerM".into());
                    }
                    23 => {
                        result.dress_model = "uBody_Eph0AM_c536".into();
                        result.head_model = "uHead_c536".into();
                        result.hair_model = "uHair_h536".into();
                        result.sound.voice = Some("Ephraim".into());
                    }
                    _ => {
                        let gid = format!("GID_{}", EMBLEM_ASSET[partner]);
                        result = asset_table_result_god_setup(result, 2, GodData::get(gid), false, conditions, None);
                    }
                }
            }
            remove_mounts_accs(result);
            result.left_hand = "null".into();
            result.right_hand = "uWep_Mg26".into();
            result.magic = "MG_DLC6_2".into();
            result.trail = "cEff_EmblemA_Swd_00".into();
            result.body_anims.iter_mut().for_each(|str| *str = "Chr1AM-Mg1_c513_M".into());
            bust_modifier_randomization(result, 0);
        }
        return result;
    }
    if mode == 2 && DVCVariables::get_flag(DVCFlags::RandomBossesNPCs, false) && person.get_flag().value & 2048 != 0 {
        if let Some(new_person) = ENEMY_PERSONS.get()
            .and_then(|v| v.iter().find(|x| x.1 == person.parent.index && x.0 >= 150))
            .and_then(|p| crate::randomizer::names::get_new_npc_person(p.0 as usize - 150))
        {
            let original_result = call_original!(this, mode, person, conditions, method_info);
            let demo = original_result.demo_anims.clone();
            let hub = original_result.hub_anims.clone();
            let result = call_original!(this, mode, new_person, conditions, method_info);
            let name = person.name.unwrap().to_string().replace("MPID", "XPID");
            person.set_name(name.into());
            result.demo_anims = demo;
            result.hub_anims = hub;
            return result;
        }
    }
    if !DVCVariables::is_main_menu() {
        if let Some(pos) = DEMOS_PIDS.iter().position(|x| *x == pid && GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0) {
            match pos {
                0 => {  //Rafal
                    if let Some(new_person) = PersonData::get_mut(DVCVariables::get_dvc_person(37, false)) {
                        return call_original!(this, mode, new_person, conditions, method_info);
                    }
                }
                _ => {  // Alear
                    if let Some(new_person) = PersonData::get_mut(DVCVariables::get_dvc_person(0, false))
                        .filter(|p| p.parent.index > 1)
                    {
                        let result = call_original!(this, mode, new_person, conditions, method_info);
                        if pos == 1 || pos ==2 { // Blue Alear
                            if let Some(asset) = search_by_key(2, pid, None) {
                                for x in 0..2 {
                                    if asset.unity_colors[x].r > 0.0 &&
                                        asset.unity_colors[x].b > 0.0 &&
                                        asset.unity_colors[x].g > 0.0
                                    {
                                        result.unity_colors[x] = asset.unity_colors[x];
                                    }
                                }
                            }
                        }
                        return result;
                    }
                }
            }
        }
    }
    call_original!(this, mode, person, conditions, method_info)
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

fn combo_engage_attack_male_emblem_index(emblem: usize, partner: bool) -> usize {
    let rng = Random::get_system();
    match emblem {
        0|13 => { 1 },
        1 => { if partner { 5 } else { 1 } }
        3 => { if rng.get_value(2) < 1 { 8 } else { 16 } }
        5 => { if partner { 1 } else { 5 } }
        6 => { 18 }
        7|14 => { 14 }
        8 => { if partner { 16 } else { 8 } }
        9 => { 
            if partner {
                if rng.get_value(2) < 1 { 20 } else { 21 }
            }
            else { 9}
        }
        11 => { 23 } //Ephraim
        12|20 => { if rng.get_value(2) < 1 { 20 } else { 21 } }
        15|21 => { MALE_EMBLEMS[ rng.get_value(9) as usize ] }
        16 => { if partner { 8 } else { 16 } }
        18 => { if partner { 22 } else { 18 } }
        4|19 => { emblem }
        _ => { 50 }
    }
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
        37 => { "DLC_43"}
        38 => { "DLC_44"}
        39 => { "DLC_45"}
        40 => { "DLC_46"}
        _ =>  { &MPIDS[index][5..] }
    }
}

pub fn other(current: usize, v1: usize, v2: usize) -> usize { if current == v1 { v2 } else { v1}  }

fn get_engage_attack_source(unit: &Unit) -> i32 {
    if let Some(engage_atk) = unit.get_engage_attack() {
        let style = unit.job.style as usize;
        if let Some(link_god) = unit.god_link {
            if link_god.data.main_data.engage_attack
                .is_some_and(|sid| SkillData::get(sid).is_some_and(|skill| skill.style_skills[style].parent.index == engage_atk.parent.index) )
            { return 3; }
            else if link_god.data.main_data.engage_attack_link
                .is_some_and(|sid| SkillData::get(sid).is_some_and(|skill| skill.style_skills[style].parent.index == engage_atk.parent.index) )
            { return 4; }
        }
        if let Some(g_unit) = unit.god_unit {
            if g_unit.data.main_data.engage_attack
                .is_some_and(|sid| SkillData::get(sid).is_some_and(|skill| skill.style_skills[style].parent.index == engage_atk.parent.index) )
            { return 1; }
            else if g_unit.data.main_data.engage_attack_link
                .is_some_and(|sid| SkillData::get(sid).is_some_and(|skill| skill.style_skills[style].parent.index == engage_atk.parent.index) )
            { return 2; }
        }
    }
    0
}

fn houses_unite_partner(index: usize, kind: i32, rng: &Random) -> Option<&GodData> {
    if kind == 0 {
        GodData::get(EMBLEM_GIDS[index]).filter(|g| g.female == 1 || (index == 19 && GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2  ))
        .or_else(|| GodData::get(EMBLEM_GIDS[index]).unwrap()
            .get_link_god_data().filter(|g| g.female == 1 || (index == 19 && GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2  ))
        )
        .or_else(||{
            let female = match index {
                0 => { 13 }
                4|14 => { 7 }
                8|16 => { 3 }
                18 => { 6 }
                19 => { FEMALE_EMBLEMS[rng.get_value(10) as usize] }
                _ => { 12 }
            };
            GodData::get(EMBLEM_GIDS[female])
        })
    }
    else {
        GodData::get(EMBLEM_GIDS[index]).filter(|g|  index != 3 && ( g.female == 0 || (index == 19 && GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 1) ) )
        .or_else(|| GodData::get(EMBLEM_GIDS[index]).unwrap().get_link_god_data()
            .filter(|g| kind != 3 && ( g.female == 0 || (index == 19 && GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 1) )))
        .or_else(||{
            let male = match (index, kind) {
                (0|1|4|5|8|14|16, 3) => { index }
                (2|13, _) => { 0 }
                (1, _) => { 5 }
                (3, _) => { if rng.get_value(2) == 0 { 8 } else { 16 } }
                (5, 1|2) => { 1 }
                (6|18, 1|2) => { 18 }
                (6|18, 3) => { 22 }
                (4|7, _) => { 14 }
                (8, 1|2) => { 16 }
                (11, _) => { 23 }
                (15|19, _)  => { MALE_EMBLEMS[rng.get_value(9) as usize] }
                (_, 1) => { 20 }
                (_, 2) => { 21 }
                (_, _) => { 9 }
            };
            GodData::get(concat_string!("GID_",EMBLEM_ASSET[male]))
        })
    }
}
fn engage_atk_result_clear(result: &mut AssetTableResult, equipped: Option<&ItemData>) {
    result.ride_model = "null".into();
    result.ride_dress_model = "null".into();
    result.left_hand = "null".into();
    result.right_hand = "null".into();
    accessory::clear_accessory_at_locator(result.accessory_list, "reserve1_loc");
    accessory::clear_accessory_at_locator(result.accessory_list, "reserve2_loc");
    accessory::clear_accessory_at_locator(result.accessory_list, "reserve3_loc");
    accessory::clear_accessory_at_locator(result.accessory_list, "reserve4_loc");
    edit_asset_weapon(result, true, 2, equipped);
}