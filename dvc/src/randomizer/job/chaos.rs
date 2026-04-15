use engage::{
    unit::{Gender, Unit},
    gamedata::{Gamedata, JobData},
    gameuserdata::GameUserData,
    proc::ProcInstFields,
    random::Random,
};
use engage::proc::Bindable;
use unity::prelude::*;
use crate::assets::get_unit_dress;
use crate::assets::gmap::GmapPlayerUnit;
use crate::DVCVariables;
use crate::randomizer::item::unit_items::{adjust_player_weapons, get_number_of_usable_weapons};

#[unity::class("App", "LevelUpSequnece")]
pub struct LevelUpSequence{
    proc: ProcInstFields,
    res_name_level_up: &'static Il2CppString,
    res_name_class_change: &'static Il2CppString,
    pub unit: &'static mut Unit,
    pub grow: Option<&'static mut Unit>,
    pub level: i32,
    pub class_change: bool,
}
impl Bindable for LevelUpSequence {}
impl LevelUpSequence {
    #[unity::class_method(5)] pub fn prepare(&self); // Offset: 0x1BE1260 Flags: 0
    #[unity::class_method(6)] pub fn reflect(&self); // Offset: 0x1BE13A0 Flags: 0
}
pub extern "C" fn level_up_prepare(this: &mut LevelUpSequence, _optional_method: OptionalMethod) {
    this.prepare();
    if this.class_change || DVCVariables::Reclassing.get_value() != 3 { return; }
    if let Some(grow) = this.grow.as_mut() {
        if let Some((kind, rank)) = grow.item_list.unit_items.iter().flat_map(|x| x).find(|item| item.flags & 1 != 0)
            .map(|u| (u.item.kind, u.item.get_weapon_level()))
            .or(Some((0, 1)))
        {
            let gender = get_unit_dress(grow);
            let mut jobs =
                JobData::get_list().unwrap().iter()
                    .filter(|j|{
                        let max_weapon_level = if kind != 0 { j.get_max_weapon_level(kind as i32) } else if grow.job.rank == 0 && grow.job.max_level == 20 { 3 } else { 5 };
                        (max_weapon_level >= rank || ((max_weapon_level >= (rank - 1)) && j.weapon_mask_plus.value & (1 << rank) != 0)) && j.base[10] > 0
                        && ((gender == Gender::Male && j.flag.value & 16 != 0) || (gender == Gender::Female && j.flag.value & 4 != 0) || (j.flag.value & 20 == 0))
                        && (j.flag.value & 3 != 0 || j.get_max_weapon_level(9) > 1)
                    }).collect::<Vec<_>>();
            
            if jobs.len() > 1 {
                let previous_max = grow.job.max_level;
                let rng = Random::get_system();
                let selection = rng.get_value(jobs.len() as i32);
                let current_level = grow.level as i32;
                let mut current_internal = grow.internal_level as i32;
                grow.set_job(jobs[selection as usize]);
                if previous_max == 40 && grow.job.max_level == 20 {
                    if current_level >= 20 {
                        grow.set_level(current_level - 20);
                        grow.set_internal_level(current_internal + 20);
                    }
                }
                else if previous_max == 20 && grow.job.max_level == 40 {
                    if current_internal > 20 { current_internal -= 20; }
                    if current_internal > 20 { current_internal = 20; }
                    grow.set_level(current_level + current_internal);
                    grow.set_internal_level(0);
                }
                else if grow.job.rank == 0 { grow.set_internal_level(0); }
                let required_kind = if kind != 0 { Some(kind as i32) } else { None };
                crate::randomizer::job::randomize_selected_weapon_mask(grow, required_kind);
                // println!("Chaos Reclassing New Weapon Mask: Selected: {} / {}", grow.selected_weapon_mask.value, grow.weapon_mask.value);
                if grow.job.learn_skill.is_some() && ((current_level >= 5 && grow.job.max_level == 20) || (current_level >= 25 && grow.job.max_level == 40)) {
                    grow.learned_job_skill = None;
                }
                if grow.level == grow.job.max_level { grow.level -= 1; }
                return;
            }
        }
    }
}

pub extern "C" fn level_up_reflect(this: &mut LevelUpSequence, _optional_method: OptionalMethod) {
    let chaos_reclassing = DVCVariables::Reclassing.get_value() == 3;
    if !this.class_change && chaos_reclassing{
        if let Some(grow) = this.grow.as_mut() {
            this.unit.selected_weapon_mask.value = grow.selected_weapon_mask.value;
            this.unit.update_weapon_mask();
        }
    }
    this.reflect();
    let sequence = GameUserData::get_sequence();
    if sequence == 6 && this.unit.person.is_hero() { GmapPlayerUnit::load_actor(); }

    if chaos_reclassing && !this.class_change {
        if sequence == 3 { this.unit.reload_actor(); }
        if get_number_of_usable_weapons(this.unit) == 0 { adjust_player_weapons(this.unit); }
    }
}

