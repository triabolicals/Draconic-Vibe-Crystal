use engage::{
    app::{IBitField32, IJobData, IJobDataMethods, IPersonDataMethods, IRandom_2Methods, IStructBase, IUnit, IUnitMethods, Random_2, Unit},
    List_1Ext
};
use outfit_core::Randomizer;
use unity::Cast;
use crate::{dvc::{unit::util::for_each_unit}, DVCFlags};

pub fn adjust_missing_weapon_mask() {
    for_each_unit(9, |unit|{
        let (mask, weapon_select_count) = unit.get_job().get_selectable_weapon_mask();
        if !mask.is_null() {
            let selectable_mask = mask.m_value();
            let selected = unit.m_selected_weapon_mask().m_value();
            let count = (1..10).into_iter().filter(|x| (((1 << x) & selectable_mask) & selected) != 0).count() as i32;
            if count < weapon_select_count { randomize_selected_weapon_mask(unit, None); }
        }
    });
}
pub fn adjust_selected_mask_from_reclass(unit: Unit) {
    let job = unit.get_job();
    let (mask, n_selects) = job.get_selectable_weapon_mask();
    let mut new_mask = 0;
    if n_selects != 0 && !mask.is_null() {
        let apt = unit.get_person().get_aptitude().m_value();
        let sub = unit.get_person().get_sub_aptitude().m_value();
        let unit_mask = unit.m_selected_weapon_mask().m_value();
        let job_masks = mask.m_value();
        let mut count = 0;
        let mut avail = vec![];
        for i in 1..10 {
            if count == n_selects { break; }
            let k_mask = 1 << i;
            if ((job_masks & unit_mask) & k_mask) != 0 {
                count += 1;
                new_mask |= k_mask
            }
            else if (apt & job_masks) & k_mask != 0 {
                count += 1;
                new_mask |= k_mask
            }
            else if (sub & job_masks) & k_mask != 0 {
                count += 1;
                new_mask |= k_mask
            }
            else { avail.push(i); }
        }
        while count < n_selects {
            if let Some(k) = avail.get_remove(Random_2::get_game()){
                new_mask |= 1 << k;
                count += 1;
            }
        }
    }
    unit.m_selected_weapon_mask().set_m_value(new_mask);
    unit.update_weapon_mask();
}
pub fn assign_selected_weapon_mask_by_apt(unit: Unit, with_kind: Option<i32>) {
    let job = unit.get_job();
    let (mask, n_selects) = job.get_selectable_weapon_mask();
    if n_selects == 0 { return; }
    let mask = mask.m_value();
    let mut possible_kinds: Vec<_> = (1..10).into_iter().filter(|x| (1 << x) & mask != 0).collect();
    let mut new_mask = 0;
    let mut selected = 0;
    if let Some(kind) = with_kind.filter(|k| possible_kinds.contains(k)){
        selected += 1;
        new_mask |= 1 << kind;
        if let Some(pos) = possible_kinds.iter().position(|k| *k == kind) { possible_kinds.remove(pos); }
    }
    let person = unit.get_person();
    if selected < n_selects {
        for apt in [person.get_aptitude().m_value(), person.get_aptitude().m_value()] {
            if selected == n_selects { break; }
            if let Some(kind) = possible_kinds.iter().find(|v| apt & (1 << *v) != 0) {
                new_mask |= 1 << *kind;
                selected += 1;
                if let Some(pos) = possible_kinds.iter().position(|k| *k == *kind) { possible_kinds.remove(pos); }
            }
        }
    }
    if selected == n_selects { unit.m_selected_weapon_mask().set_m_value(new_mask); } else { randomize_selected_weapon_mask(unit, with_kind); }
}
pub fn randomize_selected_weapon_mask(unit: Unit, with_kind: Option<i32>) {
    let job = unit.get_job();
    let (mask, selectable_weapons) = job.get_selectable_weapon_mask();
    if mask.is_null() {
        unit.m_selected_weapon_mask().set_m_value(0);
        return;
    }
    let mask = mask.m_value();
    let mut selected = 0;
    let mut possible_kinds: Vec<_> = (1..10).into_iter().filter(|x| (1 << x) & mask != 0).collect();
    let mut mask = 0;
    if let Some(kind) = with_kind.filter(|k| possible_kinds.contains(k)){
        selected += 1;
        mask |= 1 << kind;
        if let Some(pos) = possible_kinds.iter().position(|k| *k == kind) { possible_kinds.remove(pos); }
    }
    while selected < selectable_weapons {
        if let Some(kind) = possible_kinds.get_remove(Random_2::get_game()) {
            mask |= 1 << kind;
            selected += 1;
        }
        else { break; }
    }
    unit.m_selected_weapon_mask().set_m_value(mask);
    unit.update_weapon_mask();
}

pub fn update_weapon_apt(unit: Unit, init: bool) {
    let mut mask = 0;
    let job = unit.get_job();
    let person = unit.get_person();
    let weapons = job.weapons();
    let original_apt = unit.m_original_aptitude();
    if init {
        original_apt.set_m_value(person.get_aptitude().m_value());
        let rng = Random_2::get_game();
        if DVCFlags::RandomStartingApt.get_value() {
            mask = 1 << (rng.get_value_2(9) + 1);
            if rng.get_value_2(3) == 0 { mask |= 1 << (rng.get_value_2(9) + 1); }
            original_apt.set_m_value(mask);
        }
        else if job.hash() != person.get_job().hash() {
            let mut kinds = vec![];
            kinds.extend(weapons.iter().enumerate().filter(|(i, v)| *v >= 1 && *i < 9).map(|(i, _)| i));
            if job.has_high_jobs() {
                job.get_high_jobs().iter()
                    .for_each(|j| { kinds.extend(j.weapons().iter().enumerate().filter(|(i, v)| *v >= 1 && *i < 9).map(|(i, _)| i)); });
            }
            if let Some(k) = kinds.get_remove(rng) {
                mask = 1 << k;
                original_apt.set_m_value(mask);
            }
            if let Some(k) = kinds.get_remove(rng) { mask |= 1 << k; }
        }
        else { mask = person.get_aptitude().m_value() | person.get_sub_aptitude().m_value(); }
        if mask == 0 { mask = person.get_aptitude().m_value(); }
    }
    else { mask = original_apt.m_value(); }
    for x in 1..9 { if weapons.get(x) == 1 { mask |= 1 << x; } }
    mask |= unit.m_selected_weapon_mask().m_value();
    unit.m_aptitude().set_m_value(mask);
}