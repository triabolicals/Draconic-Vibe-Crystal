use super::*;
use engage::{
    app::{
        ringselectmenu::*,
        ringlistskillmenu::*,
        skillinheritancemenu::*, skillinheritancemenuitemcontent::*, skillinheritancemenuitem::*,
        gamecolor::*, basicmenuitem::*,
        ISingletonScriptableObject_1Methods,
        Ut
    },
    List_1Ext,
    Mess,
    prelude::List_1,
    tm_pro::ITMP_TextMethods,
};
use crate::{dvc::sync::EmblemSkillRandomizer, utils::override_vtable};

pub fn ring_skill_inheritance_vtable_edits() {
    override_vtable("App", "SkillInheritanceMenuItemContent", "Build", skill_inheritance_menu_item_content_build as _);
    override_vtable("App", "RingListSkillMenu.MenuItem_WeaponTalent", "BuildAttribute", weapon_talent_build_attribute as _);
    override_vtable("App", "RingSelectMenu.RingMenuItem", "BuildAttribute", ring_select_menu_item_build_attr as _);
}

#[unity::hook("App", "RingListSkillMenu", "CreateGodGrowthMenuItem")]
pub fn ring_list_skill_menu_create_menu_items(
    god: GodData,
    menu: *const u8,
    from_lv: i32,
    to_lv: i32,
    max_bond: i32,
    out: &mut List_1<BasicMenuItem>,
    ring_select: bool,
    method_info: OptionalMethod
){
    call_original!(god, menu, from_lv, to_lv, max_bond, out, ring_select, method_info);
    if !DVCFlags::RandomEnable.get_value() { return; }
    let data = RandomizedGameData::get_read();
    let inherit = DVCVariable::EmblemInherit.get_value();
    out.iter().for_each(|item|{
        if let Some(engage_item) = item.try_cast::<RingListSkillMenu_MenuItem_EngageItem>() {
            if let Some(item) = utils::to_opt(engage_item.get_item()) {
                let new_item = data.engage_weapons.get_replacement(item.hash());
                engage_item.set_item(new_item);
            }
        }
        else if let Some(emblem_skill) = item.try_cast::<RingListSkillMenu_MenuItem_Skill>() {
            if let Some(skill) = utils::to_opt(emblem_skill.get_skill()){
                let index = skill.index();
                let new_sync = data.engage_skills.get_sync_replacement_skill(index);
                if index != new_sync.index() { emblem_skill.set_skill(new_sync); }
                else {
                    let new_engage = data.engage_skills.get_engage_skill_index(index);
                    if new_engage != index { emblem_skill.set_skill(SkillData::try_get_2(new_engage)); }
                    else {
                        if inherit != 0 && inherit != 3 {
                            if let Some(new_inherit) = data.engage_skills.get_inherit(skill) {
                                emblem_skill.set_skill(new_inherit);
                            }
                        }
                    }
                }
            }
        }
    });
}

fn weapon_talent_build_attribute(_: BasicMenuItem, _: OptionalMethod) -> BasicMenuItem_Attribute {
    if DVCVariable::EmblemWepProf.get_value() == 2 { BasicMenuItem_Attribute::hide() }
    else { BasicMenuItem_Attribute::enable() }
}

#[unity::hook("App", "SkillInheritanceMenu", "CreateMenuItemList")]
pub fn skill_inheritance_menu_create_menu_item_list(god: GodData, method_info: OptionalMethod) -> List_1<SkillInheritanceMenuItem> {
    let menu: List_1<SkillInheritanceMenuItem> = call_original!(god, method_info);
    if !DVCFlags::RandomEnable.get_value() { return menu; }
    let ran_data = RandomizedGameData::get_read();
    let mode = DVCVariable::EmblemInherit.get_value();
    let random_sp = DVCFlags::RandomSP.get_value();
    menu.iter().for_each(|x| { if let Some(skill) = to_opt(x.get_skill()) { set_original_skill_hash(x, skill); } });
    if let Some(unit) = to_opt(SkillInheritanceMenu::get_m_unit()){
        if mode == 3 {
            let data = GameData::get();
            let unit_inherit_map = EmblemSkillRandomizer::generate_unit_inherit(&data, unit);
            menu.iter().for_each(|x| {
                if let Some(skill) = to_opt(get_original_skill(x)){
                    let index = skill.index();
                    if let Some(new_skill) = unit_inherit_map.get(&index).map(|&i| SkillData::try_get_2(i)){
                        x.set_skill(new_skill);
                        reset_skill_inherit_menu_item_cost(x, unit, random_sp, &ran_data.engage_skills);
                    }
                }
            });
        }
        else {
            menu.iter().for_each(|x| {
                if let Some(skill) = to_opt(get_original_skill(x)){
                    if let Some(new_skill) = ran_data.engage_skills.get_inherit(skill){
                        x.set_skill(new_skill);
                        reset_skill_inherit_menu_item_cost(x, unit, random_sp, &ran_data.engage_skills);
                    }
                }
            });
        }
    }
    menu
}
pub fn skill_inheritance_menu_item_content_build(this: SkillInheritanceMenuItemContent, item: SkillInheritanceMenuItem, _: OptionalMethod) {
    unsafe { SkillInheritanceMenuItemContent::build(this, item.into()) };
    if !DVCFlags::RandomEnable.get_value() { return; }
    if !item.is_null() {
        let ran_data = RandomizedGameData::get_read();
        let unit = SkillInheritanceMenu::get_m_unit();
        reset_skill_inherit_menu_item_cost(item, unit, DVCFlags::RandomSP.get_value(), &ran_data.engage_skills);
        this.m_text_cost().set_text_2(Ut::to_string_with_comma(item.get_skill_cost()), true);
        let bond_level_met = item.is_enough_level();
        let game_color = GameColor::get_instance();
        if item.get_sort_id() < 10000 {
            if unit.get_skill_point() < item.get_skill_cost() { this.m_text_cost().set_color(game_color.数値不足()); }
            if bond_level_met { this.m_text_name().set_color(game_color.デフォルトカラー()); }
            else {
                this.m_text_level().set_color(game_color.数値不足());
                this.m_text_name().set_color(game_color.選択不可カラー());
            }
        }
        else { this.m_text_name().set_color(game_color.デフォルトカラー()); }
    }
}
fn get_original_skill(menu_item: SkillInheritanceMenuItem) -> SkillData { SkillData::try_get_from_hash(get_skill_hash(menu_item)) }
fn set_original_skill_hash(menu_item: SkillInheritanceMenuItem, skill: SkillData) { if get_skill_hash(menu_item) == 0 { set_skill_hash(menu_item, skill.hash()); } }
fn reset_skill_inherit_menu_item_cost(menu_item: SkillInheritanceMenuItem, unit: Unit, random_sp: bool, skill_pool: &EmblemSkillRandomizer) {
    let skill = menu_item.get_skill();
    let inherited = unit.is_exist_in_equip_skill_pool_2(skill);
    let mut sort_value = menu_item.get_sort_id() % 10000;
    if inherited { sort_value += 10000; }
    menu_item.set_sort_id(sort_value);
    menu_item.set_name(Mess::get(skill.get_name()));
    if random_sp {
        let mut cost = skill_pool.get_random_inherit_cost(skill);
        let mut low = skill.low_skill();
        menu_item.set_skill_cost(cost);
        loop {
            if !low.is_null() {
                if unit.is_exist_in_equip_skill_pool_2(low) {
                    cost -= skill_pool.get_random_inherit_cost(low);
                    menu_item.set_skill_cost(cost);
                    return;
                }
                else { low = low.low_skill(); }
            }
            else {
                menu_item.set_skill_cost(cost);
                return;
            }
        }
    }
    else {
        let original_skill = get_original_skill(menu_item);
        if original_skill.hash() == skill.hash() { return; }
        let mut cost = skill.get_inheritance_cost() as i32;
        if cost == 0 {
            if skill.get_private_flag().value & 16 != 0 { cost = 5000; } 
            else { cost = original_skill.get_inheritance_cost() as i32; }
        }
        menu_item.set_skill_cost(cost);
        let mut skill_low = skill.low_skill();
        if !skill_low.is_null() {
            let mut original_low = original_skill.low_skill();
            loop {
                if !skill_low.is_null() {
                    if unit.is_exist_in_equip_skill_pool_2(skill_low) {
                        if skill_low.get_inheritance_cost() == 0 {
                            if !original_low.is_null() { cost -= original_low.get_inheritance_cost() as i32; } else { cost = cost / 2; }
                        } else { cost -= skill_low.get_inheritance_cost() as i32; }
                        if cost < 50 { cost = 500 + (skill_low.get_priority() as i32 + 1) * 100; }
                        menu_item.set_skill_cost(cost);
                        break;
                    } else {
                        skill_low = skill_low.low_skill();
                        if !original_low.is_null() { original_low = original_low.low_skill(); }
                    }
                } else {
                    if cost < 50 { cost = 100; }
                    menu_item.set_skill_cost(cost);
                    break;
                }
            }
        }
    }
}
// BuildAttribute Edit for disabling Hero / Marth Replacement in Chapter 22
fn ring_select_menu_item_build_attr(ring: RingSelectMenu_RingMenuItem, _: OptionalMethod) -> BasicMenuItem_Attribute {
    let original_result = unsafe { RingSelectMenu_RingMenuItem::build_attribute(ring) };
    if let Some(menu) = ring.m_menu().try_cast::<RingSelectMenu>() {
        let unit = menu.get_unit();
        let god = ring.get_god();
        if DVCVariables::get_chapter_index() == 22 {
            if !unit.is_null() && unit.is_hero() { BasicMenuItem_Attribute::disable() }
            else if !god.is_null() {
                if DVCVariable::EmblemRecruitment.get_value() == 0 { original_result }
                else if EmblemPool::get_replacement_index(god.m_data().get_main_data(), true).is_some_and(|v| v == 0) { BasicMenuItem_Attribute::disable() }
                else { BasicMenuItem_Attribute::enable() }
            } else { original_result }
        }
        else { original_result }
    } else { original_result }
}
fn set_skill_hash(menu_item: SkillInheritanceMenuItem, hash: i32) { unity::field_set_value_at_offset(menu_item, 0x64, hash); }
fn get_skill_hash(menu_item: SkillInheritanceMenuItem) -> i32 { unity::field_get_value_at_offset(menu_item, 0x64) }