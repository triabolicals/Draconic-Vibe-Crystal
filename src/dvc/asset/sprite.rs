use super::*;
use engage::app::{uniticon::*, IStructBase, GameIcon};
use unity::{Cast, OptionalMethod};
use crate::utils::il2str;

#[skyline::hook(offset=0x1f807b0)]
pub fn unit_icon_set_icon(this: UnitIcon, unit: Unit, _: OptionalMethod) {
    if unit.is_null() { return call_original!(this, unit, None); }
    let person = unit.get_person();
    if person.is_null() { return call_original!(this, unit, None); }
    if il2str(unit.get_person().get_aid()).is_some_and(|v| v.contains("Person_チキ")) { return call_original!(this, unit, None); }
    if unit.is_hero() && unit.is_engaging_2() && to_opt(unit.get_god_unit()).is_some_and(|v| v.is_hero()) {
        if person.hash() != PersonPool::LUEUR && !person.get_unit_icon_id().is_null(){
            let unit_icon = person.get_unit_icon_id().to_rust_string();
            let icon = format!("{}_{}", unit_icon, unit_icon);
            if this.try_set(icon, unit_icon) { return; }
        }
    }
    if let Some(pos) = PersonPool::M022_EMBLEMS.iter().position(|&p| p == unit.get_person().hash()){
        if let Some(replacement) = EmblemPool::get_replacement_god_3(pos as i32 + 1, false) {
            unit_icon_set_god_icon(this, replacement, replacement.get_female() != 0, true, None);
            return;
        }
    }
    if let Some(god) = EmblemPool::get_paralogue_custom_emblem(person) {
        this.set_icon_5(god, false);
        return;
    }
    if !unit.is_engaging_2() {
        if AppearanceManager::is_playable_rand_app(unit.get_person().hash()) {
            if let Some(icon_path) = AppearanceManager::get_appearance_person(unit.get_person(), Some(unit.get_dress_gender().value == 2))
                .and_then(|p| p.get_unit_icon(false))
            {
                let spilt = icon_path.split('_').collect::<Vec<&str>>();
                if spilt[2] != "NoWeapon" {
                    let no_weapon = format!("{}_{}_NoWeapon", spilt[0], spilt[1]);
                    if !GameIcon::tyr_get_unit_icon_index(no_weapon.as_str()).is_null(){
                        this.try_set(no_weapon, spilt[0]);
                        return;
                    }
                }
                if !GameIcon::tyr_get_unit_icon_index(icon_path.as_str()).is_null() {
                    this.try_set(icon_path.as_str(), spilt[0]);
                    return;
                }
            }
        }
        if let Some(idx) = PersonPool::get_appearance_index(person) {
            if (idx < 41 && PersonPool::is_enemy(person)) || (idx >= 41 && idx < 70 && DVCFlags::RandomBossesNPCs.get_value()) {
                if let Some(icon) = AppearanceManager::get_appearance_person(unit.get_person(), Some(unit.get_dress_gender().value == 2))
                    .and_then(|p| p.get_unit_icon(false))
                {
                    this.try_set(icon.as_str(), icon.split('_').next().unwrap());
                    return;
                }
            }
        }
    }
    else {
        let god_unit = unit.get_god_unit();
        if !god_unit.is_null() {
            let data = god_unit.m_data().get_unit_icon_id();
            let key = format!("{}E_{}_NoWeapon", person.get_unit_icon_id(), data);
            if GameIcon::tyr_get_unit_icon_index(key.as_str()).is_null() {
                let key = format!("{}_{}_NoWeapon", data, data);
                this.try_set(key.as_str(), data);
                return;
            }
        }
    }
    call_original!(this, unit, None);
    if this.m_icon_name().to_rust_string().contains("000Dummy") && !person.get_unit_icon_id().is_null() {
        let unit_icon = person.get_unit_icon_id();
        let default = format!("{}_{}", unit_icon, unit_icon);
        this.try_set(default.as_str(), unit_icon);
    }
}
#[skyline::hook(offset=0x01f827f0)]
pub fn unit_icon_set_god_icon(this: UnitIcon, god_data: GodData, is_female: bool, is_dark: bool, _: OptionalMethod) {
    if god_data.is_null() { return call_original!(this, god_data, is_female, is_dark, None); }
    if DVCFlags::GodNames.get_value() {
        if let Some(app) = AppearanceManager::get_appearance_emblem(god_data.hash(), is_dark).and_then(|v| v.get_unit_icon(is_dark)){
            if !GameIcon::tyr_get_unit_icon_index(app.as_str()).is_null() {
                this.try_set(app.as_str(), app.split('_').next().unwrap());
                return;
            }
        }
        call_original!(this, god_data, is_female, is_dark, None);
    }
    else {
        let replacement = EmblemPool::get_enemy_replacement(god_data).unwrap_or(god_data);
        let dark = EmblemPool::is_enemy_dark(god_data) | is_dark;
        let custom = EmblemPool::is_custom(god_data);
        let icon = replacement.get_unit_icon_id().to_rust_string();
        if custom && dark {
            let darkness_key = format!("997Darkness_{}_NoWeapon", icon);
            if !GameIcon::tyr_get_unit_icon_index(darkness_key.as_str()).is_null() { this.try_set(darkness_key.as_str(), "997Darkness"); }
            else { this.try_set("997Darkness_711ShadowEmblem_NoWeapon", "997Darkness"); }
        }
        else { call_original!(this, replacement, is_female, is_dark, None); }
    }
}