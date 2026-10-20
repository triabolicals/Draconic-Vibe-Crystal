use engage::{
    app::{
        Gender, 
          IBitField32, IJobData, IJobDataMethods, IMapMindMethods, 
          IMapSequenceEngageSummonMethods, 
          IPersonDataMethods, IProcInst, ISingletonClass_1Methods, 
          ISkillData, IStructBase, IStructData_1Methods, IUnitMethods, 
          ItemData, JobData, MapMind, MapSequenceEngageSummon, 
          PersonData_Ranks, Proc, ProcInst, ProcVoidMethod, Random_2, UnitUtil, Unit_Status
    },
    List_1Ext,
    ProcVoidMethodExt,
};
use outfit_core::Randomizer;
use unity::{Cast, IlNull, OptionalMethod};
use crate::{DVCFlags, DVCVariable, gamedata::GameData, utils::{clamp_value, to_opt}};

pub fn map_engage_summon_edit(proc: ProcInst) {
    let descs = proc.m_descs();
    if let Some(proc) = proc.try_cast::<MapSequenceEngageSummon>() { 
        descs.set(13, Proc::call_2(ProcVoidMethod::from_fn(proc, commit_summon).unwrap()));
    }
}

pub extern "C" fn commit_summon(proc: MapSequenceEngageSummon, _: OptionalMethod) {
    let map_mind = MapMind::get_instance();
    let unit = map_mind.get_unit();
    let person = proc.get_person();
    let rank = proc.get_rank();
    let personal = DVCFlags::PersonalSkills.get_value();
    let random_class = DVCVariable::ClassMode.get_value() == 1;
    if (!personal && !random_class) || rank == PersonData_Ranks::rank3() {
        proc.commit();
        return;
    }
    if !unit.is_null() && !person.is_null(){
        unit.set_status(Unit_Status::engage_attacked());
        if let Some(summon_unit) = to_opt(UnitUtil::summon_create(unit, rank, person)) { // unsafe { summon_create(owner, rank, person, None) }{
            if let Some(command_skill) = to_opt(map_mind.get_command_skill()).filter(|x| x.m_style_skills().len() > 0) {
                command_skill.m_style_skills().iter().for_each(|s| { summon_unit.add_private_skill_2(s); });
            }
            if random_class {
                let gender = summon_unit.get_dress_gender();
                let promoted = rank.value == 1;
                let color = summon_unit.get_person().get_summon_color().value;
                let internal = summon_unit.get_internal_level();
                let level = summon_unit.get_level();
                let total_level = internal + level;
                if let Some(random_job) = get_random_class_by_summon_color(color, promoted, gender) {
                    summon_unit.class_change(random_job, ItemData::null());
                    if random_job.get_max_level() >= 40 {
                        summon_unit.set_level(clamp_value(total_level, 1, random_job.get_max_level() as i32));
                        summon_unit.set_internal_level(0);
                    } 
                    else if random_job.is_high() {
                        summon_unit.set_internal_level(20);
                        if total_level > 40 { summon_unit.set_level(20); } else { summon_unit.set_level(clamp_value(total_level - 20, 1, 20)); }
                    } else {
                        summon_unit.set_internal_level(0);
                        if total_level > 20 { summon_unit.set_level(20); } else { summon_unit.set_level(total_level); }
                    }
                    let required_weapon = if color < 3 { Some(color + 1) } else { None };
                    crate::dvc::unit::randomize_selected_weapon_mask(summon_unit, required_weapon);
                    crate::dvc::unit::item::adjust_unit_items(summon_unit);
                    summon_unit.item_equip();
                    summon_unit.reload_actor();
                }
            }
            if personal {
                let skill = GameData::get_random_skill_dispos(2, Random_2::get_game());
                summon_unit.add_private_skill_2(skill);
            }
            summon_unit.update_state();
        }
        else { proc.commit(); }
    }
    else { proc.commit(); }
}
pub fn get_random_class_by_summon_color(color: i32, promoted: bool, gender: Gender) -> Option<JobData> {
    let jobs = JobData::get_list();
    let gender_flags = if gender == Gender::male() { 4 } else if gender == Gender::female() { 16 } else { 0 } + 35;
    let hashes: Vec<i32> =
        match color {
            0..3 => {
                jobs.iter().filter(|x| x.weapons().get(color as usize + 1) != 0 && x.get_flag().m_value() & gender_flags == 3).map(|j| j.hash()).collect()
            }
            _ => {  // Other
                jobs.iter().filter(|x|
                    x.weapons().iter().enumerate().any(|(i, x)| i > 3 && x == 1) && x.weapons().get(1) != 1 && x.weapons().get(2) != 1 && x.weapons().get(3) != 1
                        && (promoted == (x.is_high() || x.get_max_level() > 20))
                        && x.get_flag().m_value() & gender_flags == 3
                ).map(|v| v.hash()).collect()
            }
        };
    hashes.get_random_element(Random_2::get_game()).and_then(|x| to_opt(JobData::try_get_from_hash(*x)))
}