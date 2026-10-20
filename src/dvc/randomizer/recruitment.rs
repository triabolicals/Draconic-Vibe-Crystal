use super::*;
use engage::{
    app::{
        spriteatlasmanager_2::*, 
        hubdisposdata::*, hubfacilitydata::*,
        Force_Type, SkillData_Categorys, UnitPool, UnitUtil,
        IStructDataArray_1Methods, IBitField32, IStructBase, IStructData_1Methods,
        structdataarraylist_1::prelude::IList_1,
        FaceThumbnail,
    },
    GameVariableManager,
    List_1Ext,
    system::collections::generic::{IDictionary_2Methods, InsertionBehavior}
};
use engage::app::Gender;
use outfit_core::Randomizer;
use skyline::patching::Patch;
use unity::IlNull;
use crate::{
    config::{DVCConfig, DVCFlags, DVCVariable, DVCVariables},
    enums::PIDS,
    gamedata::PersonPool,
    utils::{mov_1, mov_x0_0, to_opt},
    dvc::{unit, unit::post_unit_creation_adjustment},
    gamedata::{EmblemPool, ReclassType}
};

const PROTAG_SKILLS: [&str; 4] = ["SID_主人公", "SID_王族", "SID_リベラシオン装備可能", "SID_ヴィレグランツ装備可能"];
const RECRUIT_CID: [&str; 41] = [
    "M001", "M001", "M002", "M002", "M003", "M003", "M003", "M004", "M004", "M004", "M006", "M007",
    "M007", "M007", "M008", "M008", "M009", "M011", "M011", "M011", "M012", "M012", "M012", "M013",
    "M013", "M013", "M014", "M015", "M016", "M016", "M018", "M019", "M022", "M021", "S002", "S001",
    "E006", "E006", "E006", "E006", "E006"
];
const HUB_FAC: [(&str, &str, usize); 5] = [
    ("AID_蚤の市", "LocatorSell01", 23), ("AID_筋肉体操", "LocatorTraining01", 4), 
    ("AID_ドラゴンライド", "LocatorDragon01", 17), ("AID_釣り", "LocatorFish01", 14), ("AID_占い小屋", "LocatorFortune01", 27)
];
pub const RR_ORDER: [u8; 41] = [
    32, 33, 31, 30, 29, 28, 27, 26, 25, 24,
    22, 20, 19, 18, 17, 16, 15, 14, 13, 12,
    11, 34, 10, 35, 9, 8, 7, 6, 5, 4, 3, 2,
    0, 1, 21, 23, 36, 37, 38, 39, 40
];
pub struct RecruitmentRandomizer;

impl RecruitmentRandomizer {
    pub fn setup_unit_recruitment_order(female_lueur: bool){
        if !DVCFlags::RandomEnable.get_value()  {
            if DVCFlags::OnlyRecruitmentSetting.get_from_config() { DVCVariable::UnitRecruitment.set_value(DVCConfig::get().random_recruitment); }
            else { DVCVariable::UnitRecruitment.set_value(0); }
        }
        let v = DVCVariable::UnitRecruitment.get_value();
        let rng = DVCVariables::get_rng();
        DVCVariable::LueurGender.init_var(female_lueur as i32 + 1, true);
        for i in 0..41 {
            DVCVariables::set_variable_key_string(format!("G_R_{}", PIDS[i]), PIDS[i]);
            DVCVariables::set_variable_key_string(format!("G_R2_{}", PIDS[i]), PIDS[i]);
        }
        match v {
            1 => {
                let with_gender = DVCFlags::RRGenderUnitMatch.get_value();
                let no_dlc = !DVCVariables::has_dlc() | DVCFlags::ExcludeDLCUnitRR.get_value();
                let with_custom_units = DVCFlags::CustomUnits.get_value();
                let list = &PersonPool::get().playable;
                let mut playable_list: Vec<_> =
                    list.iter()
                        .enumerate()
                        .map(|(i, person)| { if i == 0 { (i, female_lueur) } else { (i, person.female) } })
                        .filter(|&(i, _u)| i < 36 || (i >= 36 && i < 41 && !no_dlc) || (i >= 41 && with_custom_units))
                        .collect();
                let mut to_replace_list: Vec<_> = playable_list.clone();
                let pids: Vec<String> = list.iter().map(|x| x.get_person_data().get_pid().to_rust_string()).collect();
                let royals = [0, 23, 4, 17, 14, 27];
                for x_royal in royals {
                    if let Some((pos, royal)) = playable_list.iter().enumerate().find(|(_i, x)| x.0 == x_royal) {
                        if let Some(royal_replacement) = to_replace_list.get_remove_filter(rng, |(i, gender)| *i < 36 && *i != 30 && ((with_gender && *gender == royal.1) || !with_gender)) {
                            DVCVariables::set_person_recruitment(royal_replacement.0 as i32, royal.0 as i32);
                            playable_list.remove(pos);
                        }
                    }
                }
                to_replace_list.iter().for_each(|(index_x, gender_x)| {
                    if let Some((index_y, _)) = playable_list.get_remove_filter(rng, |(_, gender_y)| (with_gender && gender_x == gender_y) || !with_gender)
                        .or_else(|| playable_list.get_remove(rng))
                    {
                        DVCVariables::set_variable_key_string(format!("G_R_{}", pids[*index_x].as_str()), pids[index_y].as_str());
                        DVCVariables::set_variable_key_string(format!("G_R2_{}", pids[index_y].as_str()), pids[*index_x].as_str());
                    }
                });
            }
            2 => {
                RR_ORDER.iter().enumerate().for_each(|(i1, i2)| {
                    let i2 = *i2 as usize;
                    let (key1, key2) = (format!("G_R_{}", PIDS[i1]), format!("G_R2_{}", PIDS[i2]));
                    GameVariableManager::set_string(key1.as_str(), PIDS[i2]);
                    GameVariableManager::set_string(key2.as_str(), PIDS[i1]);
                })
            }
            3 => {
                let order = DVCConfig::get().get_custom_recruitment(false);
                order.iter().for_each(|(p1, p2)| { DVCVariables::set_person_recruitment(*p1, *p2); });
            }
            4 => {
                let order = DVCConfig::get().get_custom_recruitment(false);
                let pids: Vec<String> = PersonPool::get().playable.iter().map(|x| x.get_person_data().get_pid().to_rust_string()).collect();
                order.iter().for_each(|(p1, p2)| {
                    DVCVariables::set_variable_key_string(format!("G_R_{}", pids[*p1 as usize].as_str()), pids[*p2 as usize].as_str());
                    DVCVariables::set_variable_key_string(format!("G_R2_{}", pids[*p2 as usize].as_str()), pids[*p1 as usize].as_str());
                });
            }
            _ => {}
        }
        Self::setup_hero(true);
        Self::setup_hub();
    }
    fn setup_lueur_job(lueur_unit: Unit){
        let mode = DVCVariable::ClassMode.get_setting_value();
        if mode == 1 && !DVCFlags::RandomEnable.get_value() { return; }
        if mode == 6 {
            let lueur_gender = DVCVariable::LueurGender.get_setting_value();
            if let Some(job) = to_opt(JobData::try_get_from_hash(DVC::get().unit_job[0])){
                let flag = job.get_flag().m_value();
                if flag & 4 != 0 && lueur_gender == 1 {
                    lueur_unit.m_edit().set_gender(Gender::female());
                    DVCVariable::LueurGender.set_value(2);
                }
                else if flag & 16 != 0 && lueur_gender == 2 {
                    lueur_unit.m_edit().set_gender(Gender::male());
                    DVCVariable::LueurGender.set_value(1);
                }
            }
        }
        unit::job::unit_reclass(lueur_unit, ReclassType::get_from_settings(true));
        unit::item::adjust_unit_items(lueur_unit);
        post_unit_creation_adjustment(lueur_unit);
    }
    pub fn setup_hero(start: bool) {
        let new_hero = PersonPool::get_replacement(0, false, true);
        let lueur = PersonData::try_get_2(1);   // Lueur
        if new_hero.is_none_or(|p| p.hash() == PersonPool::LUEUR){
            let lueur_unit = UnitPool::get_from_person(lueur, false);
            DVCVariables::set_lueur_name_gender(lueur_unit);
            if !start {
                if (1 << lueur_unit.get_force_type().value) & 9 == 0 { lueur_unit.transfer(Force_Type::absent(), true); }
            }
            else { Self::setup_lueur_job(lueur_unit); }
            return;
        }
        let new_hero = new_hero.unwrap();
        if DVCVariable::HeroPerson.get_value() == 0 {
            DVCVariable::HeroPerson.init_var(new_hero.hash(), true);
        }
        let new_hero_mask = new_hero.get_mask_skill();
        let lueur_mask = lueur.get_mask_skill();
        let linked_god = lueur.get_link_god();
        new_hero.set_link_god(linked_god);
        lueur.set_link_god(GodData::null());
        PROTAG_SKILLS.iter().for_each(|sid| {
            new_hero_mask.add(*sid, SkillData_Categorys::private(), 0);
            lueur_mask.remove(*sid);
        });
        if start {
            if let Some(lueur_unit) = to_opt(UnitPool::get_from_person(lueur, false)) {
                DVCVariables::set_lueur_name_gender(lueur_unit);
                Self::setup_lueur_job(lueur_unit);
                lueur_unit.transfer(Force_Type::lost(), false);
            }
            if let Some(hero_unit) = to_opt(UnitUtil::join_unit_2(new_hero)) {
                unit::job::unit_reclass(hero_unit, ReclassType::get_from_settings(true));
                post_unit_creation_adjustment(hero_unit);
                hero_unit.transfer(Force_Type::absent(), false);
            }
        }
        Patch::in_text(0x02d524e0).nop().unwrap();
        Patch::in_text(0x02d524e4).nop().unwrap();

        // LueurW_God or Lueur_God in GetPath
        let thumb = FaceThumbnail::s_face_thumb();
        if DVCVariable::LueurGender.get_value() == 2 {
            let lueur_sprite = thumb.get("TempDVCLueur1");
            thumb.m_cache_table().try_insert("Lueur".into(), lueur_sprite, InsertionBehavior::overwrite_existing());
            Patch::in_text(0x02d524e8).bytes(&[0x48, 0x00, 0x80, 0x52]).unwrap();
            lueur.set_gender(Gender::female());
        } else {
            Patch::in_text(0x02d524e8).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap();
            lueur.set_gender(Gender::male());
        }
        let lueur_god_offsets = [0x02d51dec, 0x021e12ac]; //, 0x02915844, 0x02915694, 0x01c666ac, 0x02081edc, 0x01c69d60, 0x01c66588];
        for x in lueur_god_offsets { mov_x0_0(x); }
        // For Hub-Related Activities
        let offsets = [0x02ae8d28, 0x02ae9000, 0x02a5d0f4, 0x01cfd4c4, 0x01d03184, 0x01e5fe00, 0x01e5ff4c, 0x027049c8];
        let new_hero_gender = if new_hero.get_gender().value == 2 || (new_hero.get_gender().value == 1 && new_hero.get_flag().m_value() & 32 != 0) { 2 } else { 1 };

        for x in offsets { if new_hero_gender == 1 { mov_x0_0(x); } else { mov_1(x); } }
    }
    pub fn setup_hub() {
        if let Some(hub_solanel) = to_opt(HubDisposData::get("Hub_Solanel".into())) {
            HUB_FAC.iter().for_each(|(aid, locator, recruitment_idx)| {
                if let Some(new_recruit_idx) = PersonPool::get_replacement_index(*recruitment_idx as i32, true).filter(|i| *i < 41){
                    if let Some(dispos) = hub_solanel.items().iter().find(|v| utils::il2str(v.get_locator()).is_some_and(|v| v == *locator)) { dispos.set_chapter(RECRUIT_CID[new_recruit_idx as usize]); }
                    if let Some(facility) = to_opt(HubFacilityData::get((*aid).into())) { facility.set_condition_cid(format!("CID_{}", RECRUIT_CID[new_recruit_idx as usize])); }
                }
            });
        }
    }
    pub fn reset_hub() {
        if let Some(hub_solanel) = to_opt(HubDisposData::get("Hub_Solanel".into())) {
            HUB_FAC.iter().for_each(|&(aid, locator, recruitment_idx)| {
                if let Some(dispos) = hub_solanel.base().base().iter()
                    .find(|v| utils::il2str(v.get_locator()).is_some_and(|v| v == *locator))
                {
                    dispos.set_chapter(RECRUIT_CID[recruitment_idx]);
                }
                if let Some(facility) = to_opt(HubFacilityData::get((*aid).into())) { facility.set_condition_cid(format!("CID_{}", RECRUIT_CID[recruitment_idx])); }
            });
        }
    }
    pub fn setup_emblem_recruitment_order() {
        let rng = DVCVariables::get_rng();
        DVCVariables::create_recruitment_variables(true);
        let no_dlc = DVCVariables::has_dlc() || DVCConfig::get().dlc & 2 != 0;
        if !DVCFlags::RandomEnable.get_value()  {
            if DVCFlags::OnlyRecruitmentSetting.get_from_config() { DVCVariable::EmblemRecruitment.set_value(DVCConfig::get().emblem_mode); }
            else { DVCVariable::EmblemRecruitment.set_value(0); }
        }
        let emblem_recruitment = DVCVariable::EmblemRecruitment.get_value();
        match emblem_recruitment {
            1 => {
                let custom_emblems = DVCFlags::CustomEmblemsRecruit.get_value();
                let gids = EmblemPool::get_recruitment_gids();
                let mut available =  gids.clone();
                if gids.len() > 19 && !custom_emblems { available.drain(19..); }
                if !no_dlc { available.drain(12..19); }
                gids.iter().enumerate().for_each(|(i, gid)|{
                   if let Some(gid2) = available.get_remove(rng) {
                       let key1 = format!("G_R_{}", gid);
                       let key2 = format!("G_R2_{}", gid2);
                       DVCVariables::set_variable_key_string(key1, gid2.as_str());
                       DVCVariables::set_variable_key_string(key2, gid.as_str());
                   }
                });
            },
            2 => {  // Reverse
                let gids =  EmblemPool::get_recruitment_gids();
                for i in 0..12 {
                    let key1 = format!("G_R_{}", gids[i]);
                    let key2 = format!("G_R2_{}", gids[11-i]);
                    DVCVariables::set_variable_key_string(key1, gids[11-i].as_str());
                    DVCVariables::set_variable_key_string(key2, gids[i].as_str());
                }
            },
            3 => {  // Custom
                let order = DVCConfig::get().get_custom_recruitment(true);
                let gids = EmblemPool::get_recruitment_gids();
                let list_size = gids.len();
                order.iter().for_each(|&x| {
                    let idx = x.0 as usize;
                    let new_idx = x.1 as usize;
                    if idx < list_size && new_idx < list_size {
                        DVCVariables::set_variable_key_string(format!("G_R_{}", gids[idx]), gids[new_idx].as_str());
                        DVCVariables::set_variable_key_string(format!("G_R2_{}", gids[new_idx]), gids[idx].as_str());
                    }
                });
            },
            _ => { return; },
        }
    }
}