use crate::dvc::autolevel::{try_autoscale_unit, try_change_unit_tier_for_scale};
use crate::get_chapter_from_idx;
use super::*;

#[repr(C)]
#[derive(Clone, PartialOrd, PartialEq, Copy)]
pub enum DVCDebug {
    ClearMap,
    AverageLevel,
    OpenWorldMap,
    ChangeChapterState,
}
const CHAPTER_BOUNDS: [(i32, i32); 5] = [(4, 25), (31, 45), (51, 56), (61, 67), (4, 25)];

impl DVCDebug {
    pub fn from(i: i32) -> Option<Self> {
        match i {
            0 => Some(Self::ClearMap),
            1 => Some(Self::AverageLevel),
            2 => Some(Self::OpenWorldMap),
            3 => Some(Self::ChangeChapterState),
            _ => None,
        }
    }
    pub fn a_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        match self {
            Self::ClearMap => {
                GameVariableManager::set_bool("勝利", true);
                BasicMenu_Result::se_decide()
            }
            Self::AverageLevel => {
                DVCVariable::AverageLevel.set_value(menu_item.value());
                if DVCVariables::get_sequence() == 3 || DVCVariables::get_sequence() == 2 {
                    for_each_unit(2, |u|{
                        try_change_unit_tier_for_scale(u);
                        try_autoscale_unit(u);
                    });
                    BasicMenu_Result::se_decide()
                }
                else { BasicMenu_Result::pass() }
            }
            Self::OpenWorldMap => {
                GameVariableManager::find_starts_with("G_GmapSpot")
                    .iter()
                    .for_each(|v|{ GameVariableManager::set_number(v, 3); });
                BasicMenu_Result::se_decide()
            }
            Self::ChangeChapterState => {
                let index = get_chapter_from_idx(menu_item.value());
                let key = format!("G_Cleared_{}", index);
                let current_state = GameVariableManager::get_bool(key.as_str());
                GameVariableManager::set_bool(key.as_str(), !current_state);
                menu_item.update_text1();
                BasicMenu_Result::se_decide()
            }
        }
    }
    pub fn custom_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result  {
        if let Some(increase) = DVCBasicMenuItem::get_change(true) {
            match self {
                Self::AverageLevel => {
                    let current = menu_item.value();
                    if (current == 1 && !increase) || (current == 99 && increase) { BasicMenu_Result::se_miss() }
                    else {
                        let new_value = if increase { current + 1 } else { current - 1 };
                        DVCVariable::AverageLevel.set_value(new_value);
                        menu_item.set_value(new_value);
                        menu_item.update_text1();
                        BasicMenu_Result::se_cursor()
                    }
                }
                Self::ChangeChapterState => {
                    let current = menu_item.value();
                    let new_value =
                        if increase {
                            CHAPTER_BOUNDS.iter().position(|v| v.1 == current)
                                .and_then(|p| CHAPTER_BOUNDS.get(p + 1)).map(|p| p.0)
                                .unwrap_or_else(|| current + 1)
                        } else {
                            CHAPTER_BOUNDS.iter().rposition(|v| v.0 == current)
                                .and_then(|p| CHAPTER_BOUNDS.get(p - 1)).map(|p| p.1)
                                .unwrap_or_else(|| current - 1)
                        };
                    menu_item.set_value(new_value);
                    menu_item.update_text1();
                    BasicMenu_Result::se_cursor()
                }
                _ => {
                    if DVCBasicMenuItem::get_change(true).is_some() { BasicMenu_Result::se_miss() } else { BasicMenu_Result::pass() }
                }
            }
        }
        else { BasicMenu_Result::pass() }
    }
    pub fn build_attribute(self) -> BasicMenuItem_Attribute {
        if !DVCConfig::get().debug { return BasicMenuItem_Attribute::hide(); }
        match self {
            Self::ClearMap => {
                if DVCVariables::get_sequence() == 3 { BasicMenuItem_Attribute::enable() }
                else { BasicMenuItem_Attribute::hide() }
            }
            Self::AverageLevel => {
                if DVCFlags::Autolevel.get_value() { BasicMenuItem_Attribute::enable() }
                else { BasicMenuItem_Attribute::hide() }
            }
            _ => BasicMenuItem_Attribute::enable()
        }
    }
}