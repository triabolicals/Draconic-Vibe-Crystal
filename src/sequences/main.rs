use engage::app::{GodData, IBitField32Methods, IGodDataMethods, ISkillData, ISkillDataMethods, IStructData_1Methods, IWeaponLevels};
use std::sync::RwLock;
use engage::{app::{mainsequence::*, spriteatlasmanager_2::*, FaceThumbnail, GameIcon}, system::collections::generic::{IDictionary_2Methods, InsertionBehavior}};
use unity::{Cast, OptionalMethod};
use crate::{
    dvc::{
        mess, DVC,
        mess::ReplacementMessData,
        talk::window::WindowReplacementType
    },
    gamedata::PersonPool,
    utils::to_opt,
};
use crate::gamedata::struct_list_edit;
use super::*;

pub fn main_sequence_descs_edit(proc: ProcInst){
    let descs = proc.m_descs();
    struct_list_edit();
    dvc::unitstatus::unit_status_tick_method_edit();
    if let Some(proc) = proc.try_cast::<MainSequence>() {
        descs.set(18, Proc::call_2(ProcVoidMethod::from_fn(proc, main_sequence_initialize).unwrap()));
        descs.set(31, Proc::call_2(ProcVoidMethod::from_fn(proc, main_sequence_post_load_resource).unwrap()));
        descs.set(32, Proc::call_2(ProcVoidMethod::from_fn(proc, main_sequence_post_initialize).unwrap()));
        descs.set(43, Proc::call_2(ProcVoidMethod::from_fn(proc, main_sequence_game_reset).unwrap()));
    }
}

extern "C" fn main_sequence_initialize(proc: MainSequence, _: OptionalMethod) {
    outfit_core::UnitAssetMenuData::get().is_dvc = true;
    proc.initialize();
    DVC::get().initialize();
}

extern "C" fn main_sequence_post_load_resource(main_sequence: MainSequence, _: OptionalMethod) {
    main_sequence.post_load_resource();
    menus::CONFIG_TEXT.get_or_init(|| menus::DVCConfigText::init());
    unity::field_set_value_at_offset(main_sequence, 0x74, 1);
    mess::MESS_REPLACEMENT_DATA.get_or_init(|| RwLock::new(mess::MessageList::init()));
    ReplacementMessData::init_random_data(false);
    ReplacementMessData::init_generic_mess_data();
    PersonPool::get().fill_in_missing_data();
    mess::hub::replace_for_recruitment_demo_name("MID_HUB_Mascot_01", false);
    mess::hub::replace_for_recruitment_demo_name("MID_HUB_Mascot_02", false);
    mess::hub::replace_for_recruitment_demo_name("MID_HUB_DLC1", false);
    mess::hub::replace_for_recruitment_demo_name("MID_HUB_DLC2", false);
    dvc::unitstatus::add_unit_status_key_help_data();
    let person_pool = &PersonPool::get().pids;
    let lueur = person_pool[0].0.trim_start_matches("PID_").to_string();
    WindowReplacementType::HeroSwap.edit_label_by_string("MID_DIE_CASUAL_Lueur", &lueur, 0);
    WindowReplacementType::HeroSwap.edit_label_by_string("MID_DIE_Lueur", &lueur, 0);
    for i in 36..41 {
        if i == 37 { continue; }
        let mid = if i > 36 { format!("MID_DIE_CASUAL_{}_E", person_pool[i].1) } else { format!("MID_DIE_{}_E", person_pool[i].1) };
        let pid = person_pool[i].0.trim_start_matches("PID_").to_string();
        WindowReplacementType::RecruitmentSwap.edit_label_by_string(mid.as_str(), &pid, i as u16);
    }
    for (unit, job) in [("102Louis", "630SwordArmor"), ("153Chloe", "646SwordPegasus"), ("203Umber", "637SwordKnight"), ("250Jade" ,"631SwordArmor")]{
        let original_key = format!("{}_{}_NoWeapon", unit, job);
        let default_key = format!("{}_{}", unit, unit);
        let sprite = GameIcon::tyr_get_unit_icon_index(original_key.as_str());
        if !sprite.is_null() {
            GameIcon::s_unit_icon_index().m_cache_table().try_insert(default_key.into(), sprite, InsertionBehavior::overwrite_existing());
            let default_key = format!("{}_{}_NoWeapon", unit, unit);
            GameIcon::s_unit_icon_index().m_cache_table().try_insert(default_key.into(), sprite, InsertionBehavior::overwrite_existing());
        }
    }
}
extern "C" fn main_sequence_post_initialize(proc: MainSequence, _: OptionalMethod) {
    proc.post_initialize();
    let lueur = ["Lueur", "LueurW", "Lueur_God", "LueurW_God"];
    let face = FaceThumbnail::s_face_thumb();
    for i in 0..4 {
        if let Some(sprite) = to_opt(face.get(lueur[i])){
            let tag = format!("TempDVCLueur{}", i);
            face.m_cache_table().try_insert(tag.into(), sprite, InsertionBehavior::overwrite_existing());
        }
    }
}
extern "C" fn main_sequence_game_reset(main_sequence: MainSequence, _: OptionalMethod) {
    PersonPool::reset_reliance_data();
    main_sequence.game_reset();
    DVC::get().reset();
}