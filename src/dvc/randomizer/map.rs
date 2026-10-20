use super::*;
use engage::app::IUnitItemListMethods;
use crate::{
    dvc::unit::for_each_unit,
    gamedata::{ClassTier, JobDataBase}
};

pub mod dispos;
pub mod terrain;
pub mod fow;

pub fn setup_for_fx(){
    let chapter = DVCVariables::get_current_chapter();
    let hold_level = chapter.get_hold_level() as i32;
    if hold_level == 0 { return; }
    let db = JobDataBase::get();
    for_each_unit(25, |unit|{
        unit.m_item_list().clear();
        let job = unit.get_job();
        let (level, il) =
            if hold_level > 20 {
                if job.get_max_level() > 20 { (hold_level, 0) }
                else {
                    if job.is_low(){
                        if let Some(new_job) = db.get_reclass_job(unit, job, ClassTier::Promoted) {
                            unit.set_job(new_job);
                            unit::weapon_mask::adjust_selected_mask_from_reclass(unit);
                        }
                    }
                    (hold_level - 20, 20)
                }
            }
            else { (hold_level, 0) };
        unit.set_level(level);
        unit.set_m_learned_job_skill(SkillData::null());
        unit.set_internal_level(il);
        unit.item_add_on_dlc_evil(unit.get_job().get_unique_items(), chapter, hold_level);
        if DVCVariable::UnitInventory.get_value() & 1 != 0 || unit::item::get_number_of_usable_weapons(unit) == 0 {
            unit::item::adjust_unit_items(unit);
        }
        unit.update_state_with_auto_equip();
        unit.reset_engage_count();
        let adjust = level + il + if hold_level > 20 && job.get_max_level() == 20 && job.is_low() { 5 } else { 2 };
        unit.auto_grow_capability(level, adjust);
        unit.set_m_exp(0);
        unit::learn_skill::update_learn_skill(unit);
    });
}