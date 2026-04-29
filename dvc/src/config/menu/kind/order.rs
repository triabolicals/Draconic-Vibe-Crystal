use super::*;
use engage::{gameuserdata::GameUserData, menu::{BasicMenuItemAttribute, BasicMenuResult}};
use crate::DVCConfig;
pub static mut CUSTOM_RECRUITMENT_ORDER: [u8; 42] = [255; 42];
#[repr(C)]
#[derive(Clone, PartialOrd, PartialEq, Copy)]
pub enum RecruitmentOrder {
    Unit,
    Emblem,
    UnitCustom,
}

impl DVCCMenuItem for RecruitmentOrder {
    fn minus_call(&self, this: &mut DVCConfigMenuItem) -> BasicMenuResult {
        let index = this.index as u8;
        let emblem = *self == RecruitmentOrder::Emblem;
        if GameUserData::get_sequence() != 0 { BasicMenuResult::new() }
        else if *self != RecruitmentOrder::UnitCustom {
            this.padding = index;
            DVCConfig::get().set_custom_index(this.index, this.padding, emblem);
            this.menu.full_menu_item_list.iter_mut().for_each(|item|{
                if item.index != index as i32 && item.padding == index {
                    item.padding = 50;
                    DVCConfig::get().set_custom_index(item.index, 50, emblem);
                    item.update_text();
                }
            });
            this.update_config_text();
            BasicMenuResult::se_cursor()
        }
        else {
            unsafe { CUSTOM_RECRUITMENT_ORDER[this.index as usize] = this.index as u8; }
            this.padding = index;
            this.menu.full_menu_item_list.iter_mut().for_each(|item|{
                if item.index != index as i32 && item.padding == index {
                    item.padding = unsafe { CUSTOM_RECRUITMENT_ORDER[41] };
                    unsafe { CUSTOM_RECRUITMENT_ORDER[item.index as usize] = CUSTOM_RECRUITMENT_ORDER[41]; }
                    item.update_text();
                }
            });
            this.update_config_text();
            BasicMenuResult::se_cursor()
        }
    }
    fn plus_call(&self, this: &mut DVCConfigMenuItem) -> BasicMenuResult {
        if GameUserData::get_sequence() != 0 { BasicMenuResult::new() }
        else if *self != RecruitmentOrder::UnitCustom {
            DVCConfig::get().set_custom_index(this.index, 50, *self == RecruitmentOrder::Emblem);
            this.update_config_text();
            BasicMenuResult::se_cursor()
        }
        else {
            if is_required(this.index) {
                unsafe { CUSTOM_RECRUITMENT_ORDER[this.index as usize] = this.index as u8; }
                this.padding = this.index as u8;
                BasicMenuResult::se_miss()
            }
            else {
                unsafe { CUSTOM_RECRUITMENT_ORDER[this.index as usize] = CUSTOM_RECRUITMENT_ORDER[41]; }
                this.padding = unsafe { CUSTOM_RECRUITMENT_ORDER[41] };
                this.update_config_text();
                BasicMenuResult::se_cursor()
            }
        }
    }
    fn custom_call(&self, item: &mut DVCConfigMenuItem) -> BasicMenuResult {
        if GameUserData::get_sequence() != 0 { return BasicMenuResult::new(); }
        if let Some(increase) = get_change(true){
            match self {
                RecruitmentOrder::Unit => { item.padding = DVCConfig::get().get_next_unit(item.index, increase); }
                RecruitmentOrder::Emblem => { item.padding = DVCConfig::get().get_next_emblem(item.index, increase); }
                RecruitmentOrder::UnitCustom => {
                    let old_value = item.padding;
                    let max = unsafe { CUSTOM_RECRUITMENT_ORDER[41] };
                    let mut avail = vec![];
                    for x in 1..max {
                        if is_required(x as i32) { continue; }
                        if unsafe { !CUSTOM_RECRUITMENT_ORDER.contains(&x) } { avail.push(x); }
                    }
                    if is_required(item.index) {
                        item.padding = item.index as u8;
                        return BasicMenuResult::se_miss();
                    }
                    else {
                        item.padding =
                            if increase {
                                if old_value >= max { avail.first() }
                                else { avail.iter().find(|v_new| **v_new > old_value) }
                                    .map(|v| *v).unwrap_or(max)
                            }
                            else {
                                if old_value < 1 { avail.last() }
                                else { avail.iter().rfind(|v_new| **v_new < old_value) }
                                    .map(|v| *v).unwrap_or(max)
                            };
                    }
                }
            };
            if *self == RecruitmentOrder::UnitCustom { unsafe { CUSTOM_RECRUITMENT_ORDER[item.index as usize] = item.padding; } }
            item.update_config_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    fn build_attribute(&self, item: &DVCConfigMenuItem) -> BasicMenuItemAttribute{
        match self {
            RecruitmentOrder::UnitCustom => {
                if is_required(item.index) { BasicMenuItemAttribute::Disable }
                else { BasicMenuItemAttribute::Enable }
            }
            _ => { BasicMenuItemAttribute::Enable }
        }
    }
}
pub fn is_required(person_index: i32) -> bool { person_index < 30 && ((1 << person_index) & 142753809 != 0) }