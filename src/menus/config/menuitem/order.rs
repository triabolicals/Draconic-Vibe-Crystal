use crate::utils::to_opt;
use super::*;

#[repr(C)]
#[derive(Clone, PartialOrd, PartialEq, Copy)]
pub enum RecruitmentOrder {
    Unit,
    Emblem,
    UnitCustom,
}
impl RecruitmentOrder {
    pub fn from(i: i32) -> Option<Self> {
        match i {
            0 => Some(Self::Unit),
            1 => Some(Self::Emblem),
            2 => Some(Self::UnitCustom),
            _ => None,
        }
    }
    pub fn set_initial_value(self, menu_item: DVCBasicMenuItem) {
        if DVCVariables::is_main_menu() {
            let help = format!("Adjust order. {} Random / {} Reset.", 
               Mess::create_sprite_tag(Mess_IconCategory::system(), "Plus"), 
               Mess::create_sprite_tag(Mess_IconCategory::system(), "Minus"),
            );
            menu_item.set_m_help_text(help.into());
        }
        else { menu_item.set_m_help_text("View recruitment order.".into()); }
        self.set_text(menu_item);
    }
    pub fn set_text(self, item: DVCBasicMenuItem) {
        let recruitment = item.value2();
        if self != RecruitmentOrder::Emblem { item.set_title_text(PersonPool::get_playable_person_name(recruitment)); }
        else { item.set_title_text( EmblemPool::get_playable_name(recruitment)); }
        if !DVCVariables::is_main_menu() {
            item.set_m_is_arrow(false);
            item.set_m_is_command_icon( false);
            match self {
                RecruitmentOrder::Unit|RecruitmentOrder::UnitCustom => {
                    if let Some(p) = PersonPool::get().playable.get(recruitment as usize){
                        let person = p.get_person_data();
                        if person.index() == 1 { item.set_title_text(GameVariableManager::get_string(DVCVariables::LUEUR_NAME)); }
                        else { item.set_title_text(Mess::get(person.get_name())); }
                        if let Some(replacement) = p.get_replacement(false) {
                            if replacement.index() == 1 { item.set_m_command_text(GameVariableManager::get_string(DVCVariables::LUEUR_NAME)); }
                            else { item.set_m_command_text(Mess::get(replacement.get_name())); }
                        }
                    }
                }
                RecruitmentOrder::Emblem => {
                    if let Some(data) = EmblemPool::get().emblem_data.get(recruitment as usize) {
                        let key = format!("G_R_{}", data.get_data().get_gid());
                        item.set_m_command_text(Mess::get(data.get_data().get_mid()));
                        if GameVariableManager::is_exist(key.as_str()) {
                            if let Some(replacement) = to_opt(GodData::get(GameVariableManager::get_string(key.as_str()))) {
                                item.set_m_command_text(Mess::get(replacement.get_mid()));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        else {
            let config = DVCConfig::get();
            match self {
                RecruitmentOrder::Unit => {
                    let replacement_index = config.unit[recruitment as usize];
                    item.set_m_command_text(
                        if replacement_index < 41 { PersonPool::get_playable_person_name(replacement_index as i32 ) }
                        else { Mess::get("MID_MATCH_Random") }
                    )
                }
                RecruitmentOrder::UnitCustom => {
                    let replacement_index = config.custom_unit[recruitment as usize];
                    if is_required(recruitment) { item.set_m_is_arrow(false); }
                    item.set_m_command_text(
                        if replacement_index < 255 { PersonPool::get_playable_person_name(replacement_index as i32 ) }
                        else { Mess::get("MID_MATCH_Random") }
                    );
                }
                RecruitmentOrder::Emblem => {
                    let replacement_index = config.emblem[recruitment as usize];
                    item.set_m_command_text(
                        if replacement_index == 255 { Mess::get("MID_MATCH_Random") }
                        else { EmblemPool::get_playable_name(replacement_index as i32) }
                    );
                }
                _ => {}
            }
        }
        item.update_text();
    }
    pub fn plus_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        let i = menu_item.value2();
        DVCConfig::get().set_recruitment_random(i, self);
        self.set_text(menu_item);
        BasicMenu_Result::se_cursor()
    }
    pub fn minus_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        if DVCVariables::is_main_menu() {
            let i = menu_item.value2();
            DVCConfig::get().reset_recruitment_position(i, self);
            menu_item.get_dvc_menu().m_full_menu_item_list().iter().flat_map(|c| c.try_cast::<DVCBasicMenuItem>())
                .for_each(|i|{
                    self.set_text(i);
                    if !i.m_menu_item_content().is_null() { i.update_text(); }
                });
        }
        else {}
        BasicMenu_Result::se_cursor()
    }
    pub fn build_attribute(&self, item: DVCBasicMenuItem) -> BasicMenuItem_Attribute{
        let index = item.value2();
        match self {
            RecruitmentOrder::Emblem => {
                if index == 19 { BasicMenuItem_Attribute::hide() }
                else if index >= 12 && index < 19 && !DVCVariables::has_dlc() { BasicMenuItem_Attribute::hide() }
                else { BasicMenuItem_Attribute::enable() }
            }
            _ => {
                if !DVCVariables::has_dlc() && index >= 36 && index < 41 { BasicMenuItem_Attribute::hide() }
                else { BasicMenuItem_Attribute::enable() }
            }
        }
    }
    pub fn custom_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        let i = menu_item.value2();
        if let Some(increase) = DVCBasicMenuItem::get_change(true) {
            if self == RecruitmentOrder::UnitCustom && is_required(i) { BasicMenu_Result::se_miss() }
            else {
                DVCConfig::get().get_next_recruitment(i, increase, self);
                self.set_text(menu_item);
                BasicMenu_Result::se_cursor()
            }
        }
        else { BasicMenu_Result::pass() }
    }
}
pub fn is_required(person_index: i32) -> bool { person_index < 30 && ((1 << person_index) & 142753809 != 0) }