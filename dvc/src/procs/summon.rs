use engage::{
    gamedata::{Gamedata, JobData, skill::SkillDataCategorys},
    map::mind::MapMind, random::Random,
    unit::{Gender, UnitUtil, UnitStatusField},
    sequence::mapsequence::summon::MapSequenceEngageSummon,
    
};
use crate::{
    config::DVCFlags,
    randomizer::{
        data::GameData,
        item::unit_items::adjust_missing_weapons,
        job::randomize_selected_weapon_mask,
        Randomizer,
    }
};
use outfit_core::clamp_value;
use unity::prelude::OptionalMethod;
pub extern "C" fn commit_summon(proc: &mut MapSequenceEngageSummon, _optional_method: OptionalMethod) {
    if let Some((owner, person)) = MapMind::get_unit().zip(proc.person_data) {
        let rank = proc.rank;
        if let Some(summon_unit) = UnitUtil::summon_create(owner, rank, person){ // unsafe { summon_create(owner, rank, person, None) }{
            if let Some(command_skill) = MapMind::get_command_skill().filter(|x| x.give_skills.len() > 0 ){
                command_skill.give_skills.iter().flat_map(|x| x.get_skill()).for_each(|s|{
                    summon_unit.private_skill.add_skill(s, SkillDataCategorys::Private, 0);
                });
            }
            if DVCFlags::PersonalSkills.get_value() {
                summon_unit.private_skill.add_skill(GameData::get_random_skill_dispos(2, Random::get_game()), SkillDataCategorys::Private, 0);
            }
            let gender = summon_unit.get_dress_gender();
            if crate::DVCVariables::ClassMode.get_value() == 1 && rank < 2 {
                let promoted = rank == 1;
                let color = summon_unit.person.summon_color;
                let internal = summon_unit.internal_level as i32;
                let level = summon_unit.level as i32;
                let total_level = internal + level;
                if let Some(random_job) = get_random_class_by_summon_color(color, promoted, gender) {
                    summon_unit.class_change(random_job);
                    if random_job.max_level >= 40 {
                        summon_unit.level = clamp_value(total_level, 1, random_job.max_level as i32) as u8;
                        summon_unit.internal_level = 0;
                    } 
                    else if random_job.is_high() {
                        summon_unit.internal_level = 20;
                        if total_level > 40 { summon_unit.level = 20; }
                        else { summon_unit.level = clamp_value(total_level - 20, 1, 20) as u8; }
                    }
                    else {
                        summon_unit.internal_level = 0;
                        if total_level > 20 { summon_unit.level = 20; } else { summon_unit.level = total_level as u8; }
                    }
                    summon_unit.selected_weapon_mask.value = 0;
                    let required_weapon = if color < 3 { Some(color + 1) } else { None };
                    randomize_selected_weapon_mask(summon_unit, required_weapon);
                    adjust_missing_weapons(summon_unit);
                    summon_unit.auto_equip();
                    summon_unit.reload_actor();
                }
            }
            summon_unit.update();
        }
        owner.set_status(UnitStatusField::EngageAttacked);
    }

}
pub fn get_random_class_by_summon_color(color: i32, promoted: bool, gender: Gender) -> Option<&'static JobData> {
    let jobs = JobData::get_list().unwrap();
    let gender_flags = if gender == Gender::Male { 4 } else if gender == Gender::Female { 16 } else { 0 } + 35;
    let hashes: Vec<i32> =
        match color {
            0..3 => {
                jobs.iter().filter(|x| x.weapons[color as usize + 1] != 0 && x.flag.value & gender_flags == 3).map(|j| j.parent.hash).collect()
            }
            _ => {  // Other
                jobs.iter().filter(|x|
                    x.weapons.iter().enumerate().any(|(i, x)| i > 3 && *x == 1) && x.weapons[1] != 1 && x.weapons[2] != 1 && x.weapons[3] != 1
                        && (promoted == (x.rank == 1 || x.max_level > 20)) && x.flag.value & gender_flags == 3
                ).map(|v| v.parent.hash).collect()
            }
        };
    hashes.get_random_element(Random::get_game()).and_then(|x| JobData::try_get_hash(*x))
}