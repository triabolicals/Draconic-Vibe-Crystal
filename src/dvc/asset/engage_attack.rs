use super::{*, transform::*};
use engage::{
    app::{
        BattleSide_Type, ISkillDataMethods, IUnitEditMethods, IUnitMethods,
        IStructBase, IStructData_1Methods, Unit_Status
    },
    combat::{charactergamestatus::*, combatrecord::*, IPhase, __SkillStackEx},
    system::{collections::generic::IList_1Methods, IObjectMethods}
};
use unity::{Cast, Il2CppString, OptionalMethod};
use crate::{
    enums::{EMBLEM_GIDS},
    utils::{to_opt, il2str},
};

const ENGAGE_PREFIX: [&str; 29] = [
    "Mar", "Sig", "Cel", "Mic", "Roy", "Lei", "Luc", "Lyn", "Ike", "Byl", "Cor", "Eir",
    "Thr", "Tik", "Hec", "Ver", "Sor", "Cmi", "Chr", "Ler", "Ede", "Dim", "Cla", "Eph",
    "Enb", "Wng2D", "Cav2B", "Com0A", "Com0B"
];
const THREE_HOUSE_ACTS: [&str; 4] = ["Thr2AM-Sw1_c535_N", "Thr2AF-Ax1_c563_N", "Thr2AM-Lc1_c514_N", "Thr2AM-Bw1_c515_N"];
const THREE_HOUSES_RIGHT: [&str; 4] = ["null", "uWep_Ax20", "uWep_Lc21", "uWep_Bw14-Ar"];

#[unity::hook("Combat", "CombatRecord", "PostProcess")]
pub fn combat_record_post_process(this: CombatRecord, method_info: OptionalMethod) {
    let skills = this.get_passive_skills();
    let engage_attack_index =
        EmblemPool::GIDS.iter().position(|x|{
            let sid = format!("SID_{}エンゲージ技", x);
            __SkillStackEx::has(skills, sid.as_str())
        })
            .or_else(||{
                if __SkillStackEx::has(skills, "SID_三級長エンゲージ技＋") { Some(20) }
                else if __SkillStackEx::has(skills, "SID_三級長エンゲージ") { Some(12) }
                else { None }
            });
    if let Some(idx) = engage_attack_index {
        call_original!(this, method_info);
        let gs = this.get_game_status();
        let phase = this.get_phase_array();
        let is_enemy = (phase.base().get(0).attack_side() & 1 != 0) as usize;
        adjust_emblem_zone(gs.get(is_enemy));
        let unit = gs.get(is_enemy).get_unit();
        adjust_fx_fogato_for_engage_atk(this, BattleSide_Type::offense());
        let engage_atk_source = EngageAttackSource::from(unit);
        let emblem_idx = to_opt(unit.get_god_unit()).and_then(|v| EmblemPool::get_emblem_index(v.m_data().hash())).unwrap_or(-1);
        let dark = to_opt(unit.get_god_unit()).map(|v| v.get_is_darkness() || EmblemPool::is_enemy_dark(v.m_data())).unwrap_or(false);
        match idx {
            11 => {
                let result =
                    if emblem_idx == 11 { AssetTable_Result::get_from_god_2(AssetTable_Modes::combat(), GodData::get(EMBLEM_GIDS[23].into()), dark, CharacterAppearance::conditions()) }
                    else {
                        engage_atk_source.get_god_partner_result(unit, false)
                            .unwrap_or_else(|| AssetTable_Result::get_from_god_2(AssetTable_Modes::combat(), GodData::get(EMBLEM_GIDS[23].into()), dark, CharacterAppearance::conditions()))
                    };
                result.get_body_anims().clear();
                AnimData::remove(result, true, true);
                result.get_body_anims().add("Eir1AM-Lc1_c536_N".into());
                gs.get(2 + is_enemy).set_appearance(CharacterAppearance::create_from_result(result, 1));
            }
            18 => {
                let result =
                    if emblem_idx == 18 { AssetTable_Result::get_from_god_2(AssetTable_Modes::combat(), GodData::get(EMBLEM_GIDS[22].into()), dark, CharacterAppearance::conditions()) }
                    else {
                        engage_atk_source.get_god_partner_result(unit, false)
                            .unwrap_or_else(|| AssetTable_Result::get_from_god_2(AssetTable_Modes::combat(), GodData::get(EMBLEM_GIDS[22].into()), dark, CharacterAppearance::conditions()))
                    };
                AnimData::remove(result, true, true);
                result.get_body_anims().clear();
                result.get_body_anims().add("Chr1AM-Mg1_c513_M".into());
                result.set_left_hand("null");
                result.set_magic("MG_DLC6_2");
                result.set_trail("cEFF_EmblemA_Swd_00");
                gs.get(2 + is_enemy).set_appearance(CharacterAppearance::create_from_result(result, 1));
            }
            20 => {
                if emblem_idx == 12 || emblem_idx == 20 || emblem_idx == 21 {
                    [12, 20, 21, 9].into_iter().enumerate()
                        .map(|(i, v)| (i, AssetTable_Result::get_from_god_2(AssetTable_Modes::combat(), GodData::get(EMBLEM_GIDS[v].into()), dark, CharacterAppearance::conditions())))
                        .for_each(|(i, result)| {
                            result.get_body_anims().clear();
                            result.set_body_anim(THREE_HOUSE_ACTS[i]);
                            result.get_body_anims().add(THREE_HOUSE_ACTS[i].into());
                            AnimData::remove(result, true, false);
                            result.set_right_hand(THREE_HOUSES_RIGHT[i]);
                            result.set_left_hand(if i == 2 { "uWep_Bw14-Bw" } else { "null" });
                            gs.get(6 + i*2 + is_enemy).set_appearance(CharacterAppearance::create_from_result(result, 1));
                        });
                }
                else {
                    for i in 0..4 {
                        if let Some(result) = engage_atk_source.get_god_partner_result(unit, i == 1) {
                            result.get_body_anims().clear();
                            result.set_body_anim(THREE_HOUSE_ACTS[i]);
                            result.get_body_anims().add(THREE_HOUSE_ACTS[i].into());
                            AnimData::remove(result, true, false);
                            result.set_right_hand(THREE_HOUSES_RIGHT[i]);
                            result.set_left_hand(if i == 2 { "uWep_Bw14-Bw" } else { "null" });
                            gs.get(6 + i*2 + is_enemy).set_appearance(CharacterAppearance::create_from_result(result, 1));
                        }
                    }
                }
            }
            _ => { return; }
        }
    }
    else {
        if !this.get_game_status().get(0).get_unit().is_null() { do_transformation(this, BattleSide_Type::offense()); }
        if !this.get_game_status().get(1).get_unit().is_null()  { do_transformation(this, BattleSide_Type::defense()); }
        this.get_game_status_chain_atk().iter().filter(|x| !x.get_unit().is_null() && !x.get_weapon().is_null())
            .for_each(|gs|{ if let Some(result) = get_transformation2(gs.get_unit(), gs.get_weapon()){ gs.set_appearance(CharacterAppearance::create_from_result(result, 1)); } });
        println!("CombatRecord Style Before: {}", this.get_combat_style().value);
        call_original!(this, method_info);
        println!("CombatRecord Style After: {}", this.get_combat_style().value);
    }
}
fn adjust_emblem_zone(this: CharacterGameStatus) {
    if let Some(unit) = to_opt(this.get_unit()){
        if !this.get_emblem_identifier().is_null() {
            if unit.is_engaging_2() {
                if let Some(engage_atk) = to_opt(unit.get_engage_attack()) {
                    let sid = engage_atk.get_sid().to_rust_string();
                    let emblem_index = if let Some(pos) = EmblemPool::GIDS.iter().position(|god| sid.contains(god)) { pos }
                    else if sid.contains("三級長エンゲージ技") { 20 } else { 50 };
                    let new_emblem_id =
                        match emblem_index {
                            12|20 => { "Ede" },
                            16 => { "Sen"},
                            17 => { "Cam"},
                            19 => { "Ler"},
                            _ => { ENGAGE_PREFIX[emblem_index] }
                        };
                    this.set_emblem_identifier(new_emblem_id);
                }
            }
        }
    }
}


#[derive(Clone, Copy, PartialOrd, PartialEq)]
pub enum EngageAttackType {
    None,
    Normal,
    Override,
    TwinStrike,
    TikiDragon,
    HousesUnite,
    HousesUnitePlus,
    Veronica,
    DarkInferno,
    OtherHalf,
    DragonBlast,
    BondBlastMain,
    BondBlastSub,
    Other,
}
impl EngageAttackType {
    pub fn from_unit(unit: Unit) -> Self {
        let engage_atk = unit.get_engage_attack();
        if engage_atk.is_null() || !unit.check_status(Unit_Status::engage_attack()) { Self::None }
        else {
            let sid = engage_atk.get_sid().to_rust_string();
            if let Some(pos) = EmblemPool::GIDS.iter().position(|x| sid.contains(x)) {
                match pos {
                    1 => Self::Override,
                    13 => Self::TikiDragon,
                    15 => Self::Veronica,
                    17 => Self::DarkInferno,
                    18 => Self::OtherHalf,
                    19 => {
                        let god_unit = unit.get_god_unit();
                        if !god_unit.is_null() {
                            if !god_unit.m_child().is_null() {
                                let child_ident = god_unit.m_child().m_ident();
                                if child_ident == unit.m_ident() { Self::BondBlastSub } else { Self::BondBlastMain }
                            }
                            else { Self::DragonBlast }
                        }
                        else { Self::DragonBlast }
                    }
                    _ => Self::Normal,
                }

            }
            else if sid.contains("三級長エンゲージ技＋") { Self::HousesUnitePlus }
            else if sid.contains("三級長エンゲージ技") { Self::HousesUnite }
            else { Self::Other }
        }
    }
    pub fn setup_engage_attack_result_unit(self, result: AssetTable_Result, unit: Unit, dragon_tiki: bool, condition_flags: &AssetConditions) {
        let dark =
            if !unit.get_god_unit().is_null() {
                let gid = unit.get_god_unit().m_data().get_gid().to_rust_string();
                unit.get_god_unit().get_is_darkness() || (gid.contains("M00") || gid.contains("E00"))
            } else { false };

        let db = get_outfit_data();
        let gender = db.get_dress_gender(result.get_dress_model());
        let gen_str = if dragon_tiki { "F" } else { if gender.value == 1 { "M" } else { "F" } };
        if dragon_tiki && self != EngageAttackType::TikiDragon {
            /*
            if dark {   // Dark Tiki
                if let Some(appearance) = AppearanceManager::get().dark_emblem_app.get(13) {
                    appearance.apply_appearance(result, 2, false, None, &db.hashes, true);
                }
            }
            else {  // Emblem Tiki
                if let Some(appearance) = AppearanceManager::get().emblem_app.get(13) {
                    appearance.apply_appearance(result, 2, false, None, &db.hashes, true);
                }
            }
             */
        }
        result.set_ride_model(Il2CppString::null());
        result.set_ride_dress_model(Il2CppString::null());
        for i in 1..5 { result.commit_8(new_accessory(format!("reserve{}_loc", i), "null")); }
        if !unit.get_engage_attack().is_null() {
            println!("Engage Attack: {}", Mess::get(unit.get_engage_attack().get_name()));
        }
        println!("Three Stars: {} / {} ",condition_flags.flags.contains(AssetFlags::FiveStar), condition_flags.flags.contains(AssetFlags::ThreeStar));
        if condition_flags.flags.contains(AssetFlags::FiveStar) || condition_flags.flags.contains(AssetFlags::ThreeStar) || self == EngageAttackType::Veronica {
            let model = "uAcc_Event_SummonStone".to_string() + if condition_flags.flags.contains(AssetFlags::FiveStar) { "B" } else { "A" };
            let body_anim = "Ver".to_string() +   if condition_flags.flags.contains(AssetFlags::FiveStar) { "2A" } else { "1A" } + gen_str + "-Mg1_c000_N";
            result.get_body_anims().add(body_anim.into());
            result.commit_8(new_accessory("reserve4_loc", model));
            return;
        }

        match self {
            Self::None => { return }
            EngageAttackType::Override => {
                result.set_ride_model("uRig_HorsR");
                if dark { result.set_ride_dress_model("uBody_Sig0BR_c538"); }
                else { result.set_ride_dress_model("uBody_Sig0BR_c531"); }
            }
            EngageAttackType::TwinStrike => {}
            EngageAttackType::TikiDragon => {
                if dragon_tiki {
                    AppearanceManager::tiki_dragon(result, unit, dark, 2);
                    result.get_body_anims().add("Tik1AT-Mg1_c000_M".into());
                }
                else {
                    result.get_body_anims().clear();
                    let body_anim = "Mic1A".to_string() + gen_str + "-Mg1_c000_N";
                    result.get_body_anims().add(body_anim.into());
                }
                return;
            }
            EngageAttackType::DarkInferno => {
                result.set_ride_dress_model("uRig_DragR");
                if dark { result.set_ride_dress_model("uBody_Cmi0DR_c568"); }
                else { result.set_ride_dress_model("uBody_Cmi0DR_c561"); }
            }
            EngageAttackType::DragonBlast|EngageAttackType::BondBlastMain|EngageAttackType::BondBlastSub => {
                lueur_engage_atk(result, unit, condition_flags);
                return;
            }
            EngageAttackType::HousesUnite => {
                result.set_left_hand("uWep_Ax30L");
                result.set_right_hand("uWep_Ax30R");
                result.commit_8(new_accessory("reserve1_loc", "uWep_Ax20"));
                result.commit_8(new_accessory("reserve2_loc", "uWep_Lc21"));
                result.commit_8(new_accessory("reserve3_loc", "uWep_Bw16s-Bw"));
            }
            _ => {}
        }
        db.anims.set_engage_atk_anim(result, gender, unit);
    }
}
pub fn lueur_engage_atk(result: AssetTable_Result, unit: Unit, conditions: &AssetConditions) {
    let dress = get_outfit_data().get_dress_gender(result.get_dress_model()) == Gender::male() && !is_tiki_engage(unit);
    let gen_str = if dress { "M-Sw1" } else { "F-Sw1" };
    //if let Some(god_unit) = to_opt(unit.get_god_unit()){
        let anims = result.get_body_anims();
        let mut anim = "Ler".to_string();
        if conditions.flags.contains(AssetFlags::EngAtkCoopMain) || conditions.flags.contains(AssetFlags::EngAtkCoopSub){
            anim += "2A";
            anim += gen_str;
            anim += if conditions.flags.contains(AssetFlags::EngAtkCoopMain) { "_c" } else { "_p" };
            anim += if unit.m_edit().get_gender().value != 0 { if dress { "001_N" } else { "051_N" } } else { "000_N" };
            anims.add(anim.into());
        }
        else {
            anim += "1A";
            anim += gen_str;
            anim += "_c000_N";
            anims.add(anim.into());
        }
   // }
}
#[derive(Copy, Clone)]
enum EngageAttackSource {
    None,
    GodUnit,
    GodUnitLink,
    UnitLink,
    Unit,
}
impl EngageAttackSource {
    pub fn from(unit: Unit) -> Self {
        let engage_atk = unit.get_engage_attack();
        if engage_atk.is_null() { EngageAttackSource::None }
        else if let Some(god_unit) = to_opt(unit.get_god_unit()) {
            let engage_atk = unit.get_engage_attack();
            if engage_atk.is_null() { EngageAttackSource::None }
            else if to_opt(god_unit.get_link_god_unit())
                .and_then(|linked_god_unit| to_opt(god_unit.get_parent()))
                .is_some_and(|unit_b| GodUnit::is_around(unit, unit_b))
            {
                EngageAttackSource::GodUnit
            }
            else if to_opt(god_unit.get_child()).is_some_and(|unit_b| GodUnit::is_around(unit, unit_b)){
                EngageAttackSource::UnitLink
            }
            else { EngageAttackSource::GodUnit }
        }
        else { EngageAttackSource::Unit }
    }
    pub fn get_god_partner_result(self, unit: Unit, female: bool) -> Option<AssetTable_Result> {
        let dark;
        match self {
            Self::GodUnit => {
                let god = unit.get_god_unit().m_data();
                dark = EmblemPool::is_enemy_dark(god.get_main_data());
                let god_female = god.get_female() == 1;
                if female == god_female {
                    return Some(AssetTable_Result::get_from_god_2(AssetTable_Modes::combat(), god, dark, CharacterAppearance::conditions()))
                }
            }
            Self::GodUnitLink => {
                let link = unit.get_god_unit().get_link_god_unit();
                dark = EmblemPool::is_enemy_dark(link.m_data().get_main_data());
                let god_female = link.m_data().get_female() == 1;
                if female == god_female {
                    return Some(AssetTable_Result::get_from_god_2(AssetTable_Modes::combat(), link.m_data(), dark, CharacterAppearance::conditions()))
                }
            }
            Self::UnitLink => {
                let god_unit = unit.get_god_unit();
                let parent = god_unit.get_parent();
                let child = god_unit.get_child();
                if unit.equals(parent) && ((child.get_dress_gender().value == 2) == female) {
                    return Some(AssetTable_Result::get_from_unit(AssetTable_Modes::combat(), child, CharacterAppearance::conditions()))
                }
                else if unit.equals(child) && ((parent.get_dress_gender().value == 2) == female) {
                    return Some(AssetTable_Result::get_from_unit(AssetTable_Modes::combat(), parent, CharacterAppearance::conditions()))
                }
            }
            _ => {}
        }
        None
    }
}
pub fn random_engaging_voice(result: AssetTable_Result){
    let mut voice = result.get_sound();
    if il2str(voice.voice_id).is_none_or(|v| v.contains("MOB") || v.contains("ENEMY")) {
        let rng = Random_2::get_system();
        let index = rng.get_value_2(40) as usize + 1;
        voice.voice_id =
        match index {
            36 => { "DLC_42" }
            37 => { "DLC_43_2" }
            38 => { "DLC_44" }
            39 => { "DLC_45" }
            40 => { "DLC_46" }
            _ => { &crate::enums::MPIDS[index][5..] }
        }.into();
        result.set_sound(voice);
    }
}