use engage::battle::BattleSideType;
use engage::combat::{CharacterAppearance, CharacterGameStatus, CombatRecord};
use engage::gamedata::{Gamedata, GodData};
use engage::gamedata::assettable::{AssetTableConditionFlags, AssetTableResult, AssetTableStates, AssetTableStaticFields};
use engage::gamedata::item::ItemData;
use engage::gamedata::skill::SkillData;
use engage::unit::{Gender, Unit};
use engage::random::Random;
use outfit_core::anim::AnimData;
use outfit_core::{get_outfit_data, print_asset_table_result, AssetConditions, AssetFlags};
use unity::prelude::OptionalMethod;
use crate::assets::accessory::change_accessory;
use crate::assets::emblem::random_engage_voice;
use crate::assets::{accessory, is_tiki_engage};
use crate::assets::transform::get_transformation2;
use crate::DVCVariables;
use crate::enums::{EMBLEM_ASSET, EMBLEM_GIDS, ENGAGE_PREFIX, RINGS};
use crate::randomizer::Randomizer;

const MALE_EMBLEMS: [usize; 9] = [0, 1, 4, 5, 8, 9, 14, 16, 18];
const FEMALE_EMBLEMS: [usize; 10] = [2, 3, 6, 7, 10, 11, 12, 13, 15, 17];

const THREE_HOUSE_ACTS: [&str; 4] = ["Thr2AF-Ax1_c563_N", "Thr2AM-Lc1_c514_N", "Thr2AM-Bw1_c515_N", "Thr2AM-Sw1_c535_N"];

const THREE_HOUSES_RIGHT: [&str; 4] = ["uWep_Ax20", "uWep_Lc21", "uWep_Bw14-Ar", "null"];

fn adjust_emblem_zone(this: &mut CharacterGameStatus) {
    if let Some(unit) = this.unit {
        if this.emblem_identifier.is_some() {
            if AssetTableConditionFlags::get_state(unit) > AssetTableStates::Engaging {
                if let Some(engage_attack) = unit.get_engage_attack()  {
                    let sid = engage_attack.sid.to_string();
                    let emblem_index = if let Some(pos) = EMBLEM_ASSET.iter().position(|god| sid.contains(god)) { pos }
                    else if sid.contains("三級長エンゲージ技") { 20 }
                    else { 50 };
                    if emblem_index > 20 { return; }
                    let new_emblem_id =
                        match emblem_index {
                            12|20 => { "Ede" },
                            16 => { "Sen"},
                            17 => { "Cam"},
                            _ => { ENGAGE_PREFIX[emblem_index] }
                        };
                    this.emblem_identifier = Some(new_emblem_id.into());
                }
            }
        }
    }
}

pub fn adjust_engage_attack_animation(result: &mut AssetTableResult, unit: &Unit, equipped: Option<&ItemData>, flags: &AssetConditions) {
    if unit.get_engage_attack().is_some() {
        // println!("Adjusting Engage Attack: {}", Mess::get(engage.name.unwrap()));
        let db = get_outfit_data();
        if flags.flags.contains(AssetFlags::EngAtkCoopMain) || flags.flags.contains(AssetFlags::EngAtkCoopSub) {
            lueur_engage_atk(result, unit, flags);
            return;
        }
        let gender = db.get_dress_gender(result.dress_model);
        let r = db.anims.set_engage_atk_anim(result, gender, unit);

        engage_atk_result_clear(result, equipped);
        if r == 2 {
            let last_anim = result.body_anims.last().map(|v| v.to_string());
            let tiki = AssetTableStaticFields::get_condition_index("AID_Person_チキ");
            if let Some(entry) = AssetTableStaticFields::get().search_lists[2].iter().find(|x| x.condition_indexes.has_condition_index(tiki)) {
                result.commit_asset_table(entry);
                if let Some(las) = last_anim { result.body_anims.add(las.into()); }
                result.replace(2);
            }
        }
        else if r == 1 {
            if flags.flags.contains(AssetFlags::ThreeStar) { change_accessory(result.accessory_list, "uAcc_Event_SummonStoneA", "reserve4_loc"); }
            else if flags.flags.contains(AssetFlags::FiveStar) {
                change_accessory(result.accessory_list, "uAcc_Event_SummonStoneB", "reserve4_loc");
                result.body_anims.iter_mut().for_each(|anim| {
                    let anims = anim.to_string();
                    if anims.contains("Ver1A") && anims.contains("Mg1_c000") { *anim = anims.replace("Ver1A", "Ver2A").into(); }
                });
            }
            random_engage_voice(result);
            result.replace(2);
        }
        if let Some(engaged) = unit.get_engage_attack() {
            let is_dark = unit.get_actual_god_unit().is_some_and(|v| v.darkness || v.data.main_data.force_type != 0);
            let v = if is_dark { 8 } else { 1 };
            if engaged.sid.str_contains("シグル") {
                result.ride_model = Some("uRig_HorsR".into());
                result.ride_dress_model = Some(format!("uBody_Sig0BR_c53{}", v).into());
            }
            else if engaged.sid.str_contains("カミラ") {
                result.ride_model = Some("uRig_DragR".into());
                result.ride_dress_model = Some(format!("uBody_Cmi0DR_c56{}", v).into());
            }
        }
        print_asset_table_result(result, 2);
    }
}
#[unity::hook("Combat", "CombatRecord", "PostProcess")]
pub fn combat_record_post_process(this: &mut CombatRecord, method_info: OptionalMethod) {
    let engage_attack_index =
        EMBLEM_ASSET.iter().position(|x|{
            let sid = format!("SID_{}エンゲージ技", x);
            this.passive_skills.has(sid.as_str())
        })
            .or_else(||{
                if this.passive_skills.has("SID_三級長エンゲージ技＋") { Some(20) }
                else if this.passive_skills.has("SID_三級長エンゲージ") { Some(12) }
                else { None }
            });
    if engage_attack_index.is_none() {
        if this.game_status[0].unit.is_some() {
            crate::assets::transform::do_transformation(this, BattleSideType::Offense);
            if this.combat_style & 0x10400000 != 0 {}
        }
        if this.game_status[1].unit.is_some() {
            crate::assets::transform::do_transformation(this, BattleSideType::Defense);
            if this.combat_style & 0x10400000 != 0 {}
        }
        this.chain_atk.iter_mut().filter(|x| x.unit.is_some() && x.weapon.is_some())
            .for_each(|gs|{
                if let Some(result) = get_transformation2(gs.unit.unwrap(), gs.weapon.unwrap()){
                    gs.appearance = CharacterAppearance::create_from_result(result, 1);
                }
            });
        call_original!(this, method_info);
        return;
    }
    call_original!(this, method_info);
    if this.phase_array.parent.len() > 0 {
        let is_enemy = (this.phase_array.parent[0].attack_side & 1 != 0) as usize;
        adjust_emblem_zone(this.game_status[is_enemy]);
        if let Some(engage_atk) = engage_attack_index {
            // println!("Found Engage Attack: #{}", engage_atk);
            if let Some((Some(god), god1)) = this.game_status[is_enemy].unit.map(|x| get_engage_attack_source(x)) {
                match engage_atk {
                    11 => {
                        if let Some(male) = god1.filter(|x| x.female == 0).or_else(|| get_male_partner(god)) {
                            let result = AssetTableResult::get_from_god_data(2, male, false, CharacterAppearance::get_constions(None));
                            result.body_anims.clear();
                            result.body_anims.add("Eir1AM-Lc1_c536_N".into());
                            result.right_hand = "uWep_Lc19".into();
                            result.left_hand = "null".into();
                            AnimData::remove(result, true, true);
                            this.game_status[2 + is_enemy].appearance = CharacterAppearance::create_from_result(result, 1);
                        }
                    }
                    20 => {
                        if let Some(hashes) = houses_unite_partners(god, god1, Random::get_system()) {
                            hashes.iter().flat_map(|g| GodData::try_get_hash(*g))
                                .enumerate()
                                .for_each(|(i, god)| {
                                    let result = AssetTableResult::get_from_god_data(2, god, false, CharacterAppearance::get_constions(None));
                                    result.body_anims.clear();
                                    result.body_anim = Some(THREE_HOUSE_ACTS[i].into());
                                    result.body_anims.add(THREE_HOUSE_ACTS[i].into());
                                    AnimData::remove(result, true, true);
                                    result.left_hand = if i == 2 { "uWep_Bw14-Bw" } else { "null" }.into();
                                    result.right_hand = THREE_HOUSES_RIGHT[i].into();
                                    this.game_status[6 + i * 2 + is_enemy].appearance = CharacterAppearance::create_from_result(result, 1);
                                });
                        }
                    }
                    18 => {
                        if let Some(male) = god1.filter(|x| x.female == 0).or_else(|| get_male_partner(god)) {
                            let result = AssetTableResult::get_from_god_data(2, male, false, CharacterAppearance::get_constions(None));
                            result.body_anims.clear();
                            result.body_anims.add("Chr1AM-Mg1_c513_M".into());
                            result.left_hand = "null".into();
                            result.right_hand = "uWep_Mg26".into();
                            result.magic = "MG_DLC6_2".into();
                            result.trail = "cEff_EmblemA_Swd_00".into();
                            AnimData::remove(result, true, true);
                            this.game_status[2 + is_enemy].appearance = CharacterAppearance::create_from_result(result, 1);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
fn get_engage_attack_source(unit: &Unit) -> (Option<&'static GodData>, Option<&'static GodData>) {
    if let Some(engage_atk) = unit.get_engage_attack() {
        let style = unit.job.style as usize;
        if let Some(link_god) = unit.god_link.or(unit.god_unit){
            let link = 
                if link_god.data.main_data.engage_attack_link.is_some_and(|sid| SkillData::get(sid).is_some_and(|skill| skill.style_skills[style].parent.index == engage_atk.parent.index)) {
                    link_god.data.main_data.get_link_god_data()
                }
                else { None };
                return (Some(link_god.data), link);
        }
        if let Some(g_unit) = unit.god_unit {
            let link =
                if g_unit.data.main_data.engage_attack_link.is_some_and(|sid| SkillData::get(sid).is_some_and(|skill| skill.style_skills[style].parent.index == engage_atk.parent.index)) {
                g_unit.data.main_data.get_link_god_data()
            }
            else { None };
            return (Some(g_unit.data), link);
        }
    }
    (None, None)
}
pub fn lueur_engage_atk(result: &mut AssetTableResult, unit: &Unit, conditions: &AssetConditions) {
    let mut gen_str = if get_outfit_data().get_dress_gender(result.dress_model) == Gender::Male { "M" } else { "F" };
    if is_tiki_engage(result) { gen_str = "F"; }
    if let Some(god) = unit.god_link.or(unit.god_unit) {
        result.body_anims.clear();
        if god.child.is_none() { result.body_anims.add(format!("Ler1A{}-Sw1_c000_N", gen_str).into()); }
        else if conditions.flags.contains(AssetFlags::EngAtkCoopMain) { result.body_anims.add(format!("Ler2A{}-Sw1_c000_N", gen_str).into()); }
        else { result.body_anims.add(format!("Ler2A{}-Sw1_p000_N", gen_str).into()); }
    }
}
fn get_god_hash_from_index(index: usize) -> i32 { GodData::get(EMBLEM_GIDS[index]).map(|g|g.parent.hash).unwrap_or(0) }

fn male_lead(emblem1: usize, emblem2: usize, sel: usize) -> (usize, usize) {
    if emblem1 == sel { (emblem1, emblem2) } else { (emblem2, emblem1) }
}
fn get_male_partner(god_data: &GodData) -> Option<&'static GodData> {
    let gid = god_data.main_data.mid.to_string();
    let emblem = RINGS.iter().position(|&x| gid.contains(x))?;
    let rng = Random::get_system();
    let i =
        match emblem {
            3 => { if rng.get_value(2) == 1 { 8 } else { 16 } }
            4|7 => { 14 }
            6 => { 18 }
            8 => { 16 }
            9 => { if rng.get_value(2) == 1 { 20 } else { 21 } }
            11 => { 23 }
            12 => {
                let v = rng.get_value(3);
                if v == 0 { 9 } else if v == 1 { 20 } else { 21 }
            }
            13 => { 0 }
            15 => {
                let mut male_indexes = MALE_EMBLEMS.iter().map(|&x| x).collect::<Vec<usize>>();
                if !DVCVariables::is_lueur_female() { male_indexes.push(19); }
                male_indexes.get_remove(rng).unwrap()
            }
            18 => { 22 }
            _ => {
                if MALE_EMBLEMS.iter().any(|x| *x == emblem) { emblem }
                else { return None; }
            }
        };
    GodData::get(EMBLEM_GIDS[i])
}
fn houses_unite_partners(god_data: &GodData, linked: Option<&GodData>, rng: &Random) -> Option<[i32; 4]> {
    let mut hashes = [get_god_hash_from_index(12), get_god_hash_from_index(20), get_god_hash_from_index(21), get_god_hash_from_index(9)];
    let mut set: [bool; 4] = [false; 4];

    let gid = god_data.main_data.mid.to_string();
    let emblem_index = RINGS.iter().position(|&x| gid.contains(x))?;
    if let Some(female) = linked.filter(|x| x.female == 1 && god_data.female != 1){
        hashes[0] = female.parent.hash;
        set[0] = true;
    }
    if !set[0] && god_data.female == 1 { hashes[0] = god_data.parent.hash; set[0] = true; }
    else if !set[1] && god_data.female == 0 {
        hashes[1] = god_data.parent.hash;
        set[1] = true;
    }
    else if let Some(male) = linked.filter(|x| x.female == 0).map(|x| x.parent.hash) {
        if !hashes.contains(&male) {
            hashes[3] = male;
            set[3] = true;
        }
    }
    if emblem_index == 12 || emblem_index == 20 || emblem_index == 21 || emblem_index == 9 { Some(hashes) }
    else {
        let mut male_indexes = MALE_EMBLEMS.iter().map(|&x| x).collect::<Vec<usize>>();
        let mut female_indexes = FEMALE_EMBLEMS.iter().map(|&x| x).collect::<Vec<usize>>();
        if !DVCVariables::is_lueur_female() { male_indexes.push(19); }
        else { female_indexes.push(19); }
        if !set[0] {
            if let Some(index) = female_indexes.get_remove(rng){ hashes[0] = get_god_hash_from_index(index); }
        }
        for x in 1..4 { 
            if !set[x] {
                if let Some(index) = male_indexes.get_remove(rng){ hashes[x] = get_god_hash_from_index(index); }
            } 
        }
        match emblem_index{
            13 => {
                if !set[0] { hashes[0] = get_god_hash_from_index(13); }
                for x in 1..4 {
                    if !set[x] { hashes[x] = get_god_hash_from_index(0); break;  }
                }
            }
            1|5 => {
                let (_, support) = male_lead(1, 5, emblem_index);
                for x in 1..4 {
                    if !set[x] { hashes[x] = get_god_hash_from_index(support); break; }
                }
            }
            3|8|16 => {
                if !set[0] { hashes[0] = get_god_hash_from_index(3); }
                let (_, support) = male_lead(1, 5, emblem_index);
                for x in 1..4 { if !set[x] { hashes[x] = get_god_hash_from_index(support); break; } }
            }
            4|7|14 => {
                let (_, support) = male_lead(4, 14, emblem_index);
                if !set[0] { hashes[0] = get_god_hash_from_index(7); }
                for x in 1..4 { if !set[x] { hashes[x] = get_god_hash_from_index(support); break; } }
            }
            6|18|22 => {
                if !set[0] { hashes[0] = get_god_hash_from_index(6); }
                let (_, support) = male_lead(18, 22, emblem_index);
                for x in 1..4 { if !set[x] { hashes[x] = get_god_hash_from_index(support); break; } }
            }
            11|23 => {
                if !set[0] { hashes[0] = get_god_hash_from_index(13); }
                for x in 1..4 { if !set[x] { hashes[x] = get_god_hash_from_index(23); break; } }
            }
            15 => {
                hashes[0] =  get_god_hash_from_index(emblem_index);
            }
            _ => {}
        }
        Some(hashes)
    }
}
fn engage_atk_result_clear(result: &mut AssetTableResult, equipped: Option<&ItemData>) {
    result.ride_model = Some("null".into());
    result.ride_dress_model = None;
    accessory::clear_accessory_at_locator(result.accessory_list, "reserve1_loc");
    accessory::clear_accessory_at_locator(result.accessory_list, "reserve2_loc");
    accessory::clear_accessory_at_locator(result.accessory_list, "reserve3_loc");
    accessory::clear_accessory_at_locator(result.accessory_list, "reserve4_loc");
}