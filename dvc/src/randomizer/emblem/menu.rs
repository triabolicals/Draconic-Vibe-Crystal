use unity::prelude::*;
use super::*;
pub use engage::{
    dialog::yesno::TwoChoiceDialogMethods,
};
pub use engrave::*;
use engage::gamedata::GodData;
use engage::gamedata::item::ItemData;
use engage::gamedata::skill::SkillData;
use engage::menu::{BasicMenu, BasicMenuContent, BasicMenuItem, BasicMenuItemAttribute, MenuItem};
use engage::menu::menu_item::skill_inheritance::{SkillInheritanceMenuItem, SkillInheritanceMenuItemContent};
use unity::prelude::*;
use unity::system::List;
use crate::config::{DVCFlags, DVCVariables};

#[unity::class("App", "RingListSkillMenu")]
pub struct RingListSkillMenu {
    pub menu: &'static mut BasicMenu<BasicMenuItem>,
    pub menu_content: &'static mut BasicMenuContent,
    //..
}

#[unity::class("App", "SkillInheritanceSequence")]
pub struct SkillInheritanceSequence {
    pub proc: ProcInstFields,
    pub unit: &'static Unit,
}
pub fn skill_inheritance_menu_item_content_build(this: &mut SkillInheritanceMenuItemContent, item: &mut SkillInheritanceMenuItem, _optional_method: OptionalMethod) {
    if let Some(build_method) = this.klass.get_method_from_name("Build", 1).ok() {
        let build = unsafe { std::mem::transmute::<_, fn(&mut SkillInheritanceMenuItemContent, &mut SkillInheritanceMenuItem, &MethodInfo)>(build_method.method_ptr) };
        build(this, item, build_method);
    }
    if let Some(unit) = unsafe { get_skill_inheritance_sequence_select_unit(None) }{
        reset_skill_inherit_menu_item_cost(item, unit, DVCFlags::RandomSP.get_value());
        let bond_level_met = item.is_enough_level();
        if let Some(color) = engage::game::GameColor::get() {
            if item.sort_id < 10000 {
                let value = unsafe { to_string_with_comma(item.skill_cost, None )};
                this.text_cost.set_text(value, true);

                if (unit.skill_point as i32) < item.skill_cost { this.text_cost.set_color(color.insufficient_value); }
                else { this.text_cost.set_color(color.default_character); }
                if bond_level_met {
                    this.text_name.set_color(color.default_character);
                }
                else {
                    this.text_level.set_color(color.insufficient_value);
                    this.text_name.set_color(color.disable_character);
                }
            }
            else {
                this.text_name.set_color(color.disable_character);
            }
        }
    }
}
/*
#[skyline::from_offset(0x1d5e340)]
fn ring_skill_menu_item_skill_ctor(this: &mut BasicMenuItem, menu: *const u8, level: i32, skill_data: &SkillData, got_skill: bool, optional_method: OptionalMethod);
 */

#[skyline::from_offset(0x1c81ad0)]
fn to_string_with_comma(value: i32, optional_method: OptionalMethod) -> &'static Il2CppString;
fn change_skill_inheritance_menu_item(menu_item: &mut SkillInheritanceMenuItem, skill: &'static mut SkillData) {
    if menu_item.original_skill_index == 0 {
        menu_item.original_skill_index = menu_item.skill.as_ref().map(|m| m.parent.index).unwrap_or(0);
        /*
        if let Some(name) = skill.name.map(|v| Mess::get(v)) {
            println!("Skill has been updated with: {}", name);
        }

         */
        menu_item.skill = Some(skill);
    }
}
fn reset_skill_inherit_menu_item_cost(menu_item: &mut SkillInheritanceMenuItem, unit: &Unit, random_sp: bool) {
    let is_inherit = menu_item.skill.as_ref().map(|s| {
        let mut inherit = unit.equip_skill_pool.iter().any(|e| e.get_index() == s.parent.index);
        let mut high = s.high_skill.as_ref();
        if !inherit && high.is_some() {
            while let Some(h) = high {
                if unit.equip_skill_pool.iter().any(|e| e.get_index() == h.parent.index) {
                    inherit = true;
                    break;
                }
                high = h.high_skill.as_ref();
            }
        }
        inherit
    }).unwrap_or(false);
    if is_inherit { if menu_item.sort_id < 10000{ menu_item.sort_id += 10000; } }
    else { if menu_item.sort_id >= 10000 { menu_item.sort_id -= 10000; } }

    if let Some(skill) = menu_item.skill.as_ref() {
        if random_sp {
            let skill_cost = skill.pad4;
            let mut sk = skill.low_skill;
            while let Some(low_skill) = sk.as_ref() {
                let lower_cost = low_skill.pad4;
                if lower_cost > 0 {
                    if unit.equip_skill_pool.iter().any(|e| e.get_index() == low_skill.parent.index) {
                        menu_item.skill_cost = skill_cost - lower_cost;
                        /*
                        if let Some(name) = menu_item.skill.as_ref().and_then(|n| n.name).map(|s| Mess::get(s)) {
                            println!("Inherited MenuItem: {} with SortID: {}, Cost: {}", name, menu_item.sort_id, menu_item.skill_cost);
                        }

                         */
                        return;
                    }
                }
                sk = low_skill.low_skill;
            }
            menu_item.skill_cost = skill_cost;
        }
        else if let Some(original_skill) = SkillData::try_index_get(menu_item.original_skill_index) {
            if let Some(skill) = menu_item.skill.as_mut() {
                if skill.parent.index == original_skill.parent.index { return; }
                let skill_cost = if skill.inheritance_cost == 0 { original_skill.inheritance_cost as i32 } else { skill.inheritance_cost as i32 };
                let mut sk = skill.low_skill;
                let mut original_sk = original_skill.low_skill;
                while let Some((low, original_low)) = sk.as_ref().zip(original_sk.as_ref()) {
                    let lower_cost = if low.inheritance_cost == 0 { original_low.inheritance_cost } else { low.inheritance_cost } as i32;
                    if lower_cost > 0 {
                        if unit.equip_skill_pool.iter().any(|e| e.get_index() == low.parent.index){

                            menu_item.skill_cost = skill_cost - lower_cost;
                            /*
                            if let Some(name) = menu_item.skill.as_ref().and_then(|n| n.name).map(|s| Mess::get(s)) {
                                println!("Inherited MenuItem: {} with SortID: {}, Cost: {}", name, menu_item.sort_id, menu_item.skill_cost);
                            }
                            */
                            return;
                        }
                    }
                    sk = low.low_skill;
                    original_sk = original_low.low_skill;
                }
                menu_item.skill_cost = skill_cost;
            }
        }
    }
    /*
    if let Some(name) = menu_item.skill.as_ref().and_then(|n| n.name).map(|s| Mess::get(s)) {
        println!("MenuItem: {} with SortID: {}, Cost: {}", name, menu_item.sort_id, menu_item.skill_cost);
    }

     */
}
#[unity::hook("App", "SkillInheritanceMenu", "CreateMenuItemList")]
pub fn skill_inheritance_menu_create_menu_item_list(god: &GodData, method_info: OptionalMethod) -> &'static mut List<SkillInheritanceMenuItem> {
    let menu: &'static mut List<SkillInheritanceMenuItem> = call_original!(god, method_info);
    let mut ran_data = get_rand_data_read();
    let mode = DVCVariables::EmblemInherit.get_value();
    let random_sp = DVCFlags::RandomSP.get_value();
    if let Some(unit) = unsafe { get_skill_inheritance_sequence_select_unit(None) }{
        // if crate::DeploymentConfig::get().debug { unit.skill_point = 9999; }
        if mode == 3 {
            let playable_index = GameData::get().playables.iter().position(|x| x.hash == unit.person.parent.hash).unwrap_or(0);
            menu.iter_mut().for_each(|x| {
                if let Some(skill) = x.skill.as_mut() {
                    if let Some(new_skill) = ran_data.engage_skills.get_unit_inherit(skill, playable_index as i32) {
                        change_skill_inheritance_menu_item(x, new_skill);
                        reset_skill_inherit_menu_item_cost(x, unit, random_sp);
                    }
                }
            });
        }
        else {
            menu.iter_mut().for_each(|x| {
                if let Some(skill) = x.skill.as_mut() {
                    if let Some(new_skill) = ran_data.engage_skills.get_inherit(skill) {
                        change_skill_inheritance_menu_item(x, new_skill);
                        reset_skill_inherit_menu_item_cost(x, unit, random_sp);
                    }
                }
            });
        }
    }
    menu
}

#[unity::hook("App", "RingListSkillMenu", "CreateGodGrowthMenuItem")]
pub fn ring_list_skill_menu_create_menu_items(
    god: &GodData,
    menu: *const u8,
    from_lv: i32,
    to_lv: i32,
    max_bond: i32,
    out: &mut &mut List<BasicMenuItem>,
    ring_select: bool,
    method_info: OptionalMethod
){
    call_original!(god, menu, from_lv, to_lv, max_bond, out, ring_select, method_info);
    /*
    let playable_index =
        get_singleton_proc_instance::<ArenaOrderSequence>()
            .map(|arena| arena.training_unit.person.parent.hash)
            .or_else(||{ SortieSelectionUnitManager::get_instance().and_then(|sortie| sortie.unit).map(|v| v.person.parent.hash) })
            .or_else(||{ GodPool::try_get(god, false).and_then(|v| v.parent_unit).map(|unit| unit.person.parent.hash) })
            .and_then(|hash|GameData::get().playables.iter().position(|x| hash == x.hash)).unwrap_or(0);
    */
    let get_pos = GameData::get_playable_god_list().iter().position(|x| x.parent.hash == god.main_data.parent.hash);
    let inherit_mode = DVCVariables::EmblemInherit.get_value();
    let mut ran_data = get_rand_data_write();
    out.iter_mut().enumerate().for_each(|(index, item)|{
        let klass = item.get_class();
        let name = klass.get_name();
        if DVCFlags::EngageWeapons.get_value() && name.contains("EngageItem") {
            if let Some(get_item) = klass.get_methods().iter().find(|m| m.get_name() == Some(String::from("get_Item"))) {
                let get_item_fn = unsafe { std::mem::transmute::<_, fn(&BasicMenuItem, &MethodInfo) -> Option<&'static ItemData>>(get_item.method_ptr) };
                if let Some(new_item) = get_item_fn(item, get_item).map(|x| ran_data.engage_weapons.get_replacement(x.parent.hash)){
                    if let Some(method) = klass.get_methods().iter().find(|m| m.get_name() == Some(String::from("set_Item"))) {
                        let set_item_fn = unsafe { std::mem::transmute::<_, fn(&BasicMenuItem, &ItemData, &MethodInfo) -> Option<&'static ItemData>>(method.method_ptr) };
                        set_item_fn(item, new_item, method);
                    }
                }
            }
        }
        else if name.contains("Skill") && !name.contains("Extra") && inherit_mode > 0 {
            let get_method = klass.get_methods().iter().find(|m| m.get_name() == Some(String::from("get_Skill"))).unwrap();
            let set_method = klass.get_methods().iter().find(|m| m.get_name() == Some(String::from("set_Skill"))).unwrap();
            let get_fn = unsafe { std::mem::transmute::<_, fn(&BasicMenuItem, &MethodInfo) -> Option<&'static SkillData>>(get_method.method_ptr) };
            let set_fn = unsafe { std::mem::transmute::<_, fn(&BasicMenuItem, &SkillData, &MethodInfo)>(set_method.method_ptr) };
            if let Some(skill) = get_fn(item, get_method) {
                if skill.inheritance_cost != 0 && inherit_mode != 3 {
                    if let Some(new_item) = ran_data.engage_skills.get_inherit(skill) {
                        set_fn(item, new_item, set_method);
                      //  println!("Skill Inherit {}: {}", item.klass.get_name(), index);
                    }
                }
                else if let Some(new_item) = ran_data.engage_skills.get_sync_replacement_skill(skill.parent.index) {
                    set_fn(item, new_item, set_method);
                    // println!("Skill Sync {}: {}", item.klass.get_name(), index);
                }
            }
        }
        else if name.contains("WeaponTalent") && get_pos.is_some() && DVCVariables::EmblemWepProf.get_value() == 1 {
            let apt = ran_data.emblem_aptitude_randomizer.apts[get_pos.unwrap()];
            if let Some(get) = klass.get_methods().iter().find(|m| m.get_name() == Some(String::from("get_ItemKindTableIndex"))) {
                let get_fn = unsafe { std::mem::transmute::<_, fn(&BasicMenuItem, &MethodInfo) -> i32>(get.method_ptr) };
                let old_apt = get_fn(item, get) - 1;
                if let Some(new_apt) = apt.get(old_apt as usize) {
                    if let Some(method) = klass.get_methods().iter().find(|m| m.get_name() == Some(String::from("set_ItemKindTableIndex"))) {
                        let set = unsafe { std::mem::transmute::<_, fn(&BasicMenuItem, i32, &MethodInfo)>(method.method_ptr) };
                        // println!("{}: {}", item.klass.get_name(), index);
                        set(item, *new_apt as i32, method);
                    }
                }
            }
        }
    });
    /*
    if let Some((grow_data, klass)) = GodGrowthData::try_get_from_god_data(god)
        .zip(get_nested_class(Il2CppClass::from_name("App", "RingListSkillMenu").ok().unwrap(),  "MenuItem").ok())
    {
        let skill_item = get_nested_class(klass, "Skill").unwrap();
        for x in from_lv..to_lv {
            let level = grow_data[ x as usize].level;
            let mut skills = vec![];
            if inherit_mode > 0 || DVCVariables::EmblemSyncSkill.get_value() > 0 {
                if let Some(inherits) = grow_data[x as usize].inheritance_skills {
                    inherits.iter().for_each(|inherit_sid| {
                        if let Some(new_skill) = SkillData::get(inherit_sid)
                            .and_then(|original|
                                if inherit_mode == 3 { ran_data.engage_skills.get_unit_inherit(original, playable_index as i32) } else { ran_data.engage_skills.get_inherit(original) })
                            .filter(|skill| skill.flag & 1 == 0)
                        {
                            skills.push(new_skill.parent.hash);
                            let item = skill_item.instantiate_as::<BasicMenuItem>().ok().unwrap();
                            unsafe { ring_skill_menu_item_skill_ctor(item, menu, level, new_skill, level <= max_bond, None); }
                            out.add(item);
                        }
                    });
                }
            }
            if DVCVariables::EmblemSyncSkill.get_value() > 0 {
                if let Some(syncho) = grow_data[ x as usize].synchro_skills {
                    syncho.iter().for_each(|inherit_sid| {
                        if let Some(new_skill) = SkillData::get(inherit_sid)
                            .and_then(|original| ran_data.engage_skills.get_sync_replacement_skill(original.parent.index))
                            .filter(|skill| skill.flag & 1 == 0 && !skills.contains(&skill.parent.hash))
                        {
                            let item = skill_item.instantiate_as::<BasicMenuItem>().ok().unwrap();
                            unsafe { ring_skill_menu_item_skill_ctor(item, menu, level, new_skill, level <= max_bond, None); }
                            out.add(item);
                        }
                    });

                }
            }
        }
    }
    */
}

pub fn weapon_talent_build_attr(_this: &BasicMenuItem, _optional_method: OptionalMethod) -> BasicMenuItemAttribute {
    if DVCVariables::EmblemWepProf.get_value() == 2 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

#[skyline::from_offset(0x024a1de0)]
pub fn get_skill_inheritance_sequence_select_unit(optional_method: OptionalMethod) -> Option<&'static mut Unit>;
