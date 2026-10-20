use engage::{
    app::{
        latertalksetter::*, latertalkdata::*, latertalksequence::*, random_2::*,
        PersonData, Unit, UnitPool, UnitReliance, GameSound, GameParam,
        IStructDataArray_1Methods, IUnitMethods, IStructBase, IPersonDataMethods,
        IStructData_1Methods, IJobDataMethods, IStructDataArrayList_1Methods,
    },
    List_1Ext,
    system::collections::generic::list_1::*,
    Mess,
    combat::Character
};
use outfit_core::Randomizer;
use unity::{Cast, ClassIdentity, IlNull, OptionalMethod};
use crate::{DVCFlags, gamedata::PersonPool, utils::{clamp_value, to_opt}};
use super::*;

pub fn later_talk_edit(proc: ProcInst) {
    let descs = proc.m_descs();
    if let Some(proc) = proc.try_cast::<LaterTalkSequence>() {
        let new_descs = Array::<ProcDesc>::new(ProcDesc::class().raw(), descs.len()+1).unwrap();
        new_descs.set(0, descs.get(0));
        new_descs.set(1, Proc::call_2(ProcVoidMethod::from_fn(proc, later_talk_reorder_units).unwrap()));
        for x in 1..descs.len() { new_descs.set(x+1, descs.get(x)); }
        new_descs.set(6, Proc::call_2(ProcVoidMethod::from_fn(proc, start_next_load).unwrap()));
        proc.set_m_descs(new_descs);
    }
}

extern "C" fn later_talk_reorder_units(this: LaterTalkSequence, _: OptionalMethod) {
    let setter = this.get_setter();
    if setter.is_null() { return; }
    let units = setter.m_unit_data();
    let list = LaterTalkData::get_list();
    units.clear();
    let mut unit_count = 0;
    let mut dlc = false;
    let hero = UnitPool::get_hero(false);
    let hero_data = LaterTalkSetter_UnitData::new();
    hero_data.set_unit(hero);
    hero_data.set_is_alive(true);
    units.add(hero_data);
    let backgrounds = &dvc::asset::SEARCH_LIST.get().unwrap().map_events;
    let mut total_count = 0;
    let mut elapase_time = 0.0;
    let mut s_support_set = false;
    let mut full_unit_list: Vec<_> = UnitPool::s_units().iter().filter(|u| !u.get_person().is_null() && !u.get_force().is_null())
        .filter(|u| u.get_force_mask() & 25 != 0)
        .map(|u| u.get_person().hash())
        .collect();

    list.base().iter().flat_map(|g| g.base().base().iter())
        .for_each(|data|{ if let Some(unit) = PersonPool::get_replacement_unit_2(data.get_person(), false){ data.set_person(unit.get_pid()); } });
    let group_count = list.base().iter().count();
    let rng = Random_2::get_system();
    list.base().iter().enumerate().for_each(|(gi, group)|{
        if !dlc {
            total_count += unit_count;
            unit_count = 0;
        }
        group.base().base().iter().for_each(|data|{
            if let Some(unit) = to_opt(UnitPool::get_from_person_2(data.get_person(), false)){
                let hash = unit.get_person().hash();
                full_unit_list.retain(|v| *v != hash);
                let alive = unit.get_force_type().value == 3;
                let copy = Unit::new(true);
                copy.copy_from(unit);
                if copy.get_item_equipped().is_null() {
                    let gunner = copy.get_job().is_gunner();
                    let mask = copy.get_weapon_mask();
                    copy.set_optimal_weapon_for_class_change(mask, gunner);
                }
                if unit.get_person().is_hero() {
                    hero_data.set_light_degree(data.get_light_degree());
                    hero_data.set_map_degree(data.get_back_degree());
                    if DVCFlags::CutsceneBackground.get_value() { hero_data.set_map_name( backgrounds.get_random_element(rng).unwrap().as_str()); }
                    else { hero_data.set_map_name(data.get_field()); }
                    unit_count += 1;
                }
                else {
                    let level = UnitReliance::try_get_level_3(hero,unit);
                    let marriage = level.0 && level.1.value == 4;
                    if marriage && alive && s_support_set {
                        if !s_support_set {
                            s_support_set = true;
                            hero_data.set_marriage(copy);
                        }
                    }
                    else {
                        let new = LaterTalkSetter_UnitData::new();
                        if marriage {
                            new.set_marriage(hero);
                            new.set_unit(copy);
                        }
                        else { new.set_unit(copy); }
                        new.set_is_alive(alive);
                        new.set_light_degree(data.get_light_degree());
                        new.set_map_degree(data.get_back_degree());
                        if DVCFlags::CutsceneBackground.get_value() { new.set_map_name( backgrounds.get_random_element(rng).unwrap().as_str()); }
                        else { new.set_map_name(data.get_field()); }
                        units.add(new);
                        unit_count += 1;
                    }
                }
            }
        });

        let count = group.base().base().iter().count() as f32;
        if unit_count > 0 {
            let list_size = units.iter().count() as i32;
            if group.get_array_name().to_rust_string().contains("DLC") { dlc = true; }
            else if gi < (group_count -1){
                let group_time = setter.m_base_disp_time() * count;
                let dt = group_time / (unit_count as f32);
                for x in 0..unit_count {
                    elapase_time += dt;
                    if (total_count + x) < list_size {
                        units.get(total_count + x).set_end_disp_time(elapase_time);
                    }
                }
            }
        }
    });
    let mut bgm;
    if dlc { bgm = clamp_value(unit_count / 12 + 4, 4, 5); }
    else {
        bgm = if unit_count > 4 {  unit_count - 3 } else { unit_count - 2 } >> 1;
        bgm = clamp_value(bgm, 0, 2);
    }
    let param_name = format!("後日談：BGM時間{}", bgm);
    let other_time = GameParam::get_float(param_name);
    let dt_other_1 = other_time / (unit_count as f32);
    let mut other_time = 0.0;
    full_unit_list.iter().map(|&p| UnitPool::get_from_person(PersonData::try_get_from_hash(p), false))
        .for_each(|unit|{
            let new = LaterTalkSetter_UnitData::new();
            let alive = unit.get_force_type().value == 3;
            let copy = Unit::new(true);
            copy.copy_from(unit);
            if copy.get_item_equipped().is_null() {
                let gunner = copy.get_job().is_gunner();
                let mask = copy.get_weapon_mask();
                copy.set_optimal_weapon_for_class_change(mask, gunner);
            }
            let rng = rng.get_value_2(3);
            let level = UnitReliance::try_get_level_3(hero,unit);
            let marriage = level.0 && level.1.value == 4;
            if marriage && alive && !s_support_set {
                s_support_set = true;
                hero_data.set_marriage(copy);
            }
            else {
                new.set_map_degree(if rng == 1 { 90 } else if rng == 2 { 180 } else { 0 });
                new.set_map_name(backgrounds.get_random_element(Random_2::get_system()).unwrap().as_str());
                if marriage {
                    new.set_marriage(hero);
                    new.set_unit(copy);
                }
                else { new.set_unit(copy); }
                new.set_is_alive(alive);
                new.set_light_degree(50);
                units.add(new);
                unit_count += 1;
            }
        });
    let list_size = units.iter().count() as i32;
    for x in 0..unit_count {
        if (total_count + x) < list_size {
            let dt = if has_ending( units.get(total_count + x)) { dt_other_1 } else { 0.4 * dt_other_1 };
            other_time += dt;
            elapase_time += dt;
            units.get(total_count + x).set_end_disp_time(elapase_time);
        }
    }
    setter.set_m_others_time(other_time);
}
fn has_ending(data: LaterTalkSetter_UnitData) -> bool {
    let name = data.get_unit().get_person().get_ascii_name().to_rust_string();
    Mess::is_exist(format!("MID_AFTER_{}_Text", name).as_str()) || !data.get_marriage().is_null()
}
extern "C" fn start_next_load(this: LaterTalkSequence, _: OptionalMethod) {
    let index = this.get_setter().m_unit_index();
    if index == 41 {
        GameSound::post_event(LaterTalkSequence::bgm_branch(), Character::null());
    }
    this.start_load_next();
}
