use super::*;
pub use engage::{
    combat::{character::*, characterappearance::*},
    app::{Gender, godunit::*, assettable::*}
};
use engage::{
    app::{mapmind::*, UnitPool},
    combat::{animasset::*, ICharacterAnimset, ICharacterAssetT_1Methods},
};
pub use outfit_core::*;
use outfit_core::{PersonalDressData, anim::AnimData, Randomizer};
use crate::{
    gamedata::{EmblemPool, PersonPool},
    utils::to_opt,
    dvc::{
        asset::dress::{change_color_by_rng},
        asset::emblem::{ENGAGE_F, ENGAGE_M},
        HeroAppearanceMode,
        DVC
    }
};
pub use crate::gamedata::AppearanceManager;

pub mod dress;
pub mod transform;
pub mod emblem;
pub mod color;
pub mod sprite;
pub mod facethumb;
pub mod gmapunit;
pub mod engage_attack;
pub mod weapon;
pub mod other;

pub use other::SEARCH_LIST;
use crate::dvc::asset::engage_attack::EngageAttackType;
use crate::dvc::asset::transform::{is_dragonstone, is_tiki_dragonstone};

#[skyline::hook(offset=0x01bb4180)]
pub fn asset_table_setup_person(
    this: AssetTable_Result,
    mode: i32,
    person:PersonData,
    conditions: Array<Il2CppString>,
    method_info: OptionalMethod
) -> AssetTable_Result
{
    if person.is_null() { return call_original!(this, mode, person, conditions, method_info); }
    let pid = person.get_pid().to_rust_string();
    if pid.contains("PID_デモ用_視線設定用") || person.get_gender().value == 0 { return call_original!(this, mode, person, conditions, method_info); }
    let hash = person.hash();
    let result = 
    if PersonPool::DIVINE_LUEUR.contains(&hash) && PersonPool::get_replacement_index(0, false).is_some_and(|v| v != 0){
        let hero = UnitPool::get_hero(false);
        let result =
            if !hero.is_null() { call_original!(this, mode, hero.get_person(), conditions, method_info) }
            else { call_original!(this, mode, person, conditions, method_info) };
        this.set_pid(person.get_pid());
        result
    }
    else if PersonPool::VEYRES.contains(&pid.as_str()) {
        if let Some(replacement) = PersonPool::get_npc_veyle_replacement(person, true){
            let result = call_original!(this, mode, replacement, conditions, method_info);
            AppearanceManager::evil_enemy_asset_addition(result, person, mode);
            this.set_pid(person.get_pid());
            result
        }
        else { call_original!(this, mode, person, conditions, method_info) }
    }
    else if PersonPool::is_playable(person) {
        let result = call_original!(this, mode, person, conditions, method_info);
        random_body_scale(result, None, false);
        if let Some(data) = UnitAssetMenuData::get_by_person_data(person.hash(), true) {
            data.set_result(result, mode, false, false);
        }
        result
    }
    else if let Some(r_person) = PersonPool::get_enemy_player_replacement(person, true) {
        let result = call_original!(this, mode, r_person, conditions, method_info);
        AppearanceManager::evil_enemy_asset_addition(result, person, mode);
        this.set_pid(person.get_pid());
        result
    }
    else if DVCVariables::get_sequence() != 0 {
        let original_result = call_original!(this, mode, person, conditions, method_info);
        if DVCFlags::RandomBossesNPCs.get_value() {
            let db = get_outfit_data();
            if let Some(data) = AppearanceManager::get_appearance_person(person, None){
                data.apply_appearance(original_result, mode, false, None, &db.hashes, true);
            }
            else {
                let gender = person.get_dress_gender().value;
                let rng = Random_2::get_system();
                let index =
                if gender == 1 {
                    let s = PersonalDressData::get_male_indexes(false, |_| true);
                    s.get_random_element(rng).cloned()
                }
                else {
                    let s = PersonalDressData::get_female_indexes(false, |_| true);
                    s.get_random_element(rng).cloned()
                };
                if let Some(data) = index.and_then(|i| db.dress.personal.get(i)) {
                    data.apply_appearance(original_result, mode, false, None, &db.hashes, true);
                }
                if DVCVariable::GenericAppearance.get_value() & 2 != 0 { change_color_by_rng(original_result, rng); }
            }
        }
        original_result
    }
    else {  call_original!(this, mode, person, conditions, method_info) };
    if !PersonPool::is_playable(person) { random_body_scale(result, None, false); }
    result
}

#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook(
    this: AssetTable_Result,
    mode: i32,
    unit: Unit,
    equipped: ItemData,
    conditions: Array<Il2CppString>,
    method_info: OptionalMethod) -> AssetTable_Result
{
    if let Some(app) = DVC::get().hero_appearance {
        if unit.get_person().index() == 1 {
            let hero = UnitPool::get_hero(false);
            if !hero.is_null() && hero.get_person().hash() != PersonPool::LUEUR {
                let result: AssetTable_Result = call_original!(this, mode, hero, equipped, conditions, method_info);
                if app == HeroAppearanceMode::Training {
                    AssetFlags::set_condition_key("AID_鍛錬着", true);
                }
                result.commit(AssetTable_Modes::combat());
                result.replace(AssetTable_Modes::combat());
                if hero.is_female() { result.set_m_hub_anim("AOC_Hub_c051".into()); }
                else { result.set_m_hub_anim("AOC_Hub_c001".into()); }
                DVC::get().hero_appearance = None;
                return result;
            }
        }
    }
    if mode == 3 && unit.get_person().hash() == PersonPool::LUEUR {
        let result: AssetTable_Result = call_original!(this, mode, unit, equipped, conditions, method_info);
        result.commit(AssetTable_Modes::gmap());
        return result;
    }
    let result: AssetTable_Result = call_original!(this, if mode == 3 { 1 } else { mode }, unit, equipped, conditions, method_info);
    let conditions = dress::commit_for_unit_dress(result, if mode == 3 { 1 } else { mode }, unit, equipped, conditions);
    if mode == 3 { gmapunit::edit_result_for_gmap(result); }
    else if conditions.flags.contains(AssetFlags::Engaging) {
        AnimData::remove(result, true, true);
        let target = MapMind::get_instance().get_target_unit();
        engage_attack::random_engaging_voice(result);
        if !target.is_null() {
            let female = get_outfit_data().get_dress_gender(result.get_dress_model()).value == 2;
            let anim =
                match (female, unit.is_hero()) {
                    (true, true) => "Tsf0AF-No1_c051_N",
                    (true, false) => "Com0AF-No1_c000_N",
                    (false, false) => "Com0AM-No1_c000_N",
                    (false, true) => "Tsf0AM-No1_c001_N",
                };
            result.get_body_anims().clear();
            result.set_body_anim(anim);
            result.get_body_anims().add(anim.into());
        }
    }
    else if conditions.flags.contains(AssetFlags::ThreeStar) || conditions.flags.contains(AssetFlags::FiveStar) {
        EngageAttackType::Veronica.setup_engage_attack_result_unit(result, unit, false, &conditions);
    }
    else if conditions.flags.contains(AssetFlags::EngageTiki) || unit.get_job().get_jid().to_rust_string().contains("紋章士") ||
        (conditions.flags.contains(AssetFlags::MapTransform) && mode == 1) || conditions.flags.contains(AssetFlags::Monster)
    {
        return result;
    }
    else if mode == 2 && (conditions.flags.contains(AssetFlags::EngAtkCoopMain) || conditions.flags.contains(AssetFlags::EngAtkCoopSub)) { // DragonBlast or BondBlast
        engage_attack::lueur_engage_atk(result, unit, &conditions);
    }
    else if conditions.flags.contains(AssetFlags::EngageAttack) && mode == 2 {
        if !equipped.is_null() {
            if equipped.get_kind().value < 9 {
                let item_hash = equipped.hash();
                if let Some(data) = get_outfit_data().item.iter().find(|i| i.hash == item_hash){
                    data.apply(result);
                }
            }
        }
        EngageAttackType::from_unit(unit).setup_engage_attack_result_unit(result, unit, false, &conditions);
    }
    else if conditions.emblem_unit {}
    else {  // anim correction
        if conditions.flags.contains(AssetFlags::EngageTiki) { return result; }
        get_outfit_data().correct_anims(result, unit, conditions.profile_flag, &conditions);
    }
    if !equipped.is_null() && mode == 2 {  // weapon asset
        weapon::edit_asset_weapon(result, equipped);
    }
    //if mode != 2 { print_asset_table_result(result, mode); }
    result
}

#[skyline::hook(offset= 0x2b0ed80)]
pub fn appearance_create_from_result_2(this: AssetTable_Result, map_distance: i32, o: OptionalMethod) -> CharacterAppearance {
    let appearance: CharacterAppearance = call_original!(this, map_distance, o);
    if !appearance.is_null() {
        if !this.get_pid().is_null() {
            let person = PersonData::get(this.get_pid());
            if !person.is_null() {
                if to_opt(UnitPool::get_from_person(person, false)).is_some_and(|p| p.is_hero() && p.is_engaging_2() && to_opt(p.get_god_unit()).is_some_and(|h| h.is_hero())){
                    unity::field_set_value_at_offset::<i32>(appearance, 0xd4, 1);
                }
                else { unity::field_set_value_at_offset::<i32>(appearance, 0xd4, person.hash()); }
            }
            else {
                let god = GodData::get(this.get_pid());
                if !god.is_null() {
                    unity::field_set_value_at_offset::<i32>(appearance, 0xd4, god.hash());
                    if AssetFlags::get_condition_index("エンゲージ開始")
                        .map(|v| AssetTable::s_condition_flags().test(v)).unwrap_or(false)
                    {
                        let anim_set = appearance.get_anim_set();
                        let female = this.get_dress_model().to_rust_string().contains("F_c");
                        if !anim_set.is_null() {
                            if god.get_flag().m_value() & 32 != 0 {
                                if anim_set.engage3().is_null() || il2str(anim_set.engage3().get_name()).is_none_or(|v| v.contains("Tsf")){
                                    let rng = Random_2::get_system();
                                    let set = if female { ENGAGE_F[rng.get_value_2(6) as usize] } else { ENGAGE_M[rng.get_value_2(6) as usize ] };
                                    let anim = format!("uAnim_{}", set);
                                    let asset = AnimAsset::new_2(anim.into());
                                    asset.set_hash(0x143dd064);
                                    anim_set.set_engage3(asset);
                                }
                            }
                            else if god.get_flag().m_value() & 32 == 0 {
                                if anim_set.engage1().is_null() || il2str(anim_set.engage1().get_name()).is_none_or(|v| v.contains("Tsf")){
                                    let rng = Random_2::get_system();
                                    let set = if female { ENGAGE_F[rng.get_value_2(6) as usize] } else { ENGAGE_M[rng.get_value_2(6) as usize ] };
                                    let anim = format!("uAnim_{}", set);
                                    let asset = AnimAsset::new_2(anim.into());
                                    asset.set_hash(-0x5cc4eb8);
                                    anim_set.set_engage1(asset);
                                }
                            }
                        }
                    }
                }
            }
        }
        if DVC::get().print_events {
            // print_asset_table_result(this, 2);
            for i in 0..20 {
                if let Some(asset) = to_opt(appearance.assets().get(i)){
                    if !asset.get_name().is_null() { println!("Asset[{}] = {}", i, asset.get_name()); }
                }
            }
        }
    }
    appearance
}
pub fn new_accessory(locator: impl Into<Il2CppString>, model: impl Into<Il2CppString>) -> AssetTable_Accessory {
    let acc = AssetTable_Accessory::new();
    acc.set_locator(locator.into());
    acc.set_model(model.into());
    acc
}

pub(crate) fn random_body_scale(result: AssetTable_Result, hash: Option<i32>, _is_unit_info: bool) {
    let scale_mode = DVCVariable::BodyScaling.get_value();
    if scale_mode == 0 { return; }
    let rng = if let Some(hash) = hash { DVCVariables::create_hash_rng(hash, 1) } else { Random_2::get_system() };
    if scale_mode & 1 != 0 { set_result_scale_u16(result, 9, get_random_scaling(9, rng) as u16); }
    if scale_mode & 2 != 0 {
        for x in 0..13 {
            if x == 9 { continue; }
            set_result_scale_u16(result, x as usize, get_random_scaling(x, rng) as u16);
        }
    }
}
pub fn print_asset_table_result(result: AssetTable_Result, mode: i32) {
    // #[cfg(feature="debug")] {
        let jid = il2str(result.get_jid()).map(|v| engage::app::Mess::get_game_data_name(v).to_rust_string()).unwrap_or("---".to_string());
        if let Some(pid) = il2str(result.get_pid()) { println!("Asset Table Result PID: {} / {} [Mode: {}]", engage::app::Mess::get_game_data_name(pid), jid, mode); }
        if let Some(ride) = il2str(result.get_ride_model()) { println!("Ride Model: {}", ride); }
        if mode == 2 {
            if let Some(dress) = il2str(result.get_dress_model()) { println!("Dress Model: {}/ {}", dress, il2str(result.get_ride_dress_model()).unwrap_or("---".to_string())); }
        }
        println!("Body/Head/Hair: {} / {} / {}",
            il2str(result.get_body_model()).unwrap_or("--".to_string()),
            il2str(result.get_head_model()).unwrap_or("--".to_string()),
            il2str(result.get_hair_model()).unwrap_or("---".to_string())
        );
        if mode != 2 {
            if let Some(body) = il2str(result.get_body_anim()) { println!("Body Anim: {} / {}", body, il2str(result.get_ride_anim()).unwrap_or("---".to_string())); }
        } else {
            if let Some(body) = il2str(result.get_body_anim()) { println!("Body Anim: {}", body); }
        }
        println!("Hands: R: {}/ L: {}",  il2str(result.get_right_hand()).unwrap_or("--".to_string()), il2str(result.get_left_hand()).unwrap_or("--".to_string()));
        let mut acc = String::new();
        result.get_accessories().base().iter().filter(|v| !v.is_null())
            .filter(|v| !v.get_locator().is_null() && !v.get_model().is_null())
            .for_each(|v| {
                acc += format!("({} @ {}) ", v.get_model(), v.get_locator()).as_str();
            });
        if !acc.is_empty() { println!("Acc: {}", acc); }
        let mut color_str = String::new();
        for i in 0..8 {
            let color = get_result_color_u8(result, i);
            if color[0] != 0 || color[1] != 0 || color[2] != 0 { color_str += format!("#{}: [{}/{}/{}] ", i + 1, color[0], color[1], color[2]).as_str() }
        }
        if !color_str.is_empty() { println!("Colors: {}", color_str); }
        if mode == 1 {
            println!("Scale(All,Head,Wing)= {}, {}, {}", get_result_scale_f32(result, 16), get_result_scale_f32(result, 17), get_result_scale_f32(result, 18));
        } else {
            let scale_str = ["All,Head,Neck,Torso", "Shoulder/Arm/Hands/Legs", "Feet/VBust/VAbdomen/VTorso", "VArm/VBLegs/VSArm/VSLegs"];
            for i in 0..4 {
                println!("Scale[{}]: {}, {}, {}, {}",
                    scale_str[i], get_result_scale_f32(result, 4 * i), get_result_scale_f32(result, 4 * i + 1), get_result_scale_f32(result, 4 * i + 2), get_result_scale_f32(result, 4 * i + 3)
                )
            }
        }
        let mut anims = String::new();
        result.get_body_anims().iter().for_each(|v| { if !anims.is_empty() { anims += "/"; }anims += v.to_rust_string().as_str(); });
        println!("Anim: {}", anims);
   // }
}