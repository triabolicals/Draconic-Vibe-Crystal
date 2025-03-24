use unity::prelude::*;
use unity::system::List;
use engage::{
    gamedata::{accessory::*, unit::*, Gamedata},
    gameuserdata::GameUserData, 
    gamevariable::GameVariableManager, 
    menu::*,
    mess::Mess, 
    proc::Bindable, 
    sortie::SortieSelectionUnitManager
};
use super::ACCESSORY_COUNT;

#[unity::class("App", "UnitMenuItem")]
pub struct UnitMenuItem {
    pub menu: u64,
    pub junk: [u8; 0x4c],
    pub unit: &'static mut Unit,
}

fn unit_accessory_sub_menu_create_bind(menu: &mut BasicMenuItem){
    let list = menu.menu.full_menu_item_list.get_class();
    let new_list = il2cpp::instantiate_class::<List<BasicMenuItem>>(list).unwrap();
    let count;
    if *ACCESSORY_COUNT.get().unwrap() >= 6  {
        new_list.items = Il2CppArray::new(10).unwrap();
        count = 7;
    }
    else {
        new_list.items = Il2CppArray::new(4).unwrap();
        count = 4;
    }
    for _x in 0..count {
        let cock = get_base_menu_item_class().clone();
        let new_menu_item = il2cpp::instantiate_class::<BasicMenuItem>(cock).unwrap();
        new_menu_item.get_class_mut().get_virtual_method_mut("GetName").map(|method| method.method_ptr = unit_access_sub_menu_name as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("LCall").map(|method| method.method_ptr = unit_access_sub_menu_l_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("RCall").map(|method| method.method_ptr = unit_access_sub_menu_r_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("PlusCall").map(|method| method.method_ptr = unit_access_sub_menu_plus_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("YCall").map(|method| method.method_ptr = unit_access_sub_menu_y_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("MinusCall").map(|method| method.method_ptr = unit_access_sub_menu_minus_call as _);
        new_list.add(new_menu_item);
    }
    let content = unsafe { create_basic_menu_content(None) };
    let new_menu = BasicMenu::new(new_list, content);
    let descs = new_menu.create_default_desc();
    new_menu.bind_parent_menu();
    new_menu.create_bind(menu.menu, descs, "UnitAccessorySubMenu");
    new_menu.set_transform_as_sub_menu(menu.menu, menu);
    new_menu.set_show_row_num(count);
}

pub fn reload_unit_info(unit: &Unit) -> i32 {
    unsafe {
        help_set_unit(0, None, false, false, false, None, None);
        help_set_unit(1, None, false, false, false, None, None);
        help_set_unit(0, Some(unit), false, false, false, None, None);
    }
    let sequence = GameUserData::get_sequence();
    if sequence == 3 || sequence == 2 { unit.reload_actor(); }
    return 0x80;
}

#[skyline::from_offset(0x01f61b10)]
pub fn accessory_count(lol: u64, method_info: OptionalMethod) -> i32; 

fn get_base_menu_item_class() -> &'static mut Il2CppClass {
    let menu = Il2CppClass::from_name("App", "UnitSelectSubMenu").unwrap().get_nested_types().iter().find(|x| x.get_name() == "BaseMenuItem").unwrap();
    Il2CppClass::from_il2cpptype(menu.get_type()).unwrap()
}

pub fn unit_menu_item_y_call(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    unit_accessory_sub_menu_create_bind(this);
    return 0x80;
}

pub fn unit_access_sub_menu_name(this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    let accessory_index = if this.index < 4 { this.index } else { this.index + 1 };
    let unit = SortieSelectionUnitManager::get_unit();
    let mode = super::get_unit_outfit_mode(unit);
    let slot = &unit.accessory_list.unit_accessory_array[accessory_index as usize];
    if slot.index == 0 { return "--------".into(); }
    if let Some(acc) = AccessoryData::try_index_get(slot.index) { 
        if ( mode == 1 && accessory_index != 5) || (mode == 2 && accessory_index != 0 ) { return format!("[{}]: {}", Mess::get("MID_SORTIE_SKILL_CATEGORY_EQUIPED"), Mess::get(acc.name)).into(); }
        else {
            return Mess::get(acc.name);
        }
    }
    else { return "--------".into(); }
}

pub fn unit_access_sub_menu_r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32{
    let kind = if this.index < 4 { this.index }
        else { this.index + 1 };

    let unit = SortieSelectionUnitManager::get_unit();
    if super::accessory::next_unit_accessory(unit, kind, true) {
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    else { return 0x800; }
}

pub fn unit_access_sub_menu_l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let kind = if this.index < 4 { this.index }
        else { this.index + 1 };

    let unit = SortieSelectionUnitManager::get_unit();
    if super::accessory::next_unit_accessory(unit, kind, false) {
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    else { return 0x800; }
}

pub fn unit_access_sub_menu_plus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let unit = SortieSelectionUnitManager::get_unit();
    let mode = super::get_unit_outfit_mode(unit);
    let key = format!("G_O{}", unit.person.pid);
    match mode {
        0 => { GameVariableManager::set_number(key.as_str(), 1); }
        1 => {
            if unsafe { accessory_count(0, None) >= 6 } { GameVariableManager::set_number(key.as_str(), 2); }
            else { GameVariableManager::set_number(key.as_str(), 0); }
        }
        _ => { GameVariableManager::set_number(key.as_str(), 0); }
    }
    this.menu.full_menu_item_list.iter().for_each(|item| item.rebuild_text());

    return reload_unit_info(unit);
}

pub fn unit_access_sub_menu_minus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let unit = SortieSelectionUnitManager::get_unit();
    let accessory_list = &mut unsafe { super::unit_get_accessory_list(unit, None) }.unit_accessory_array;
    let kind = if this.index < 4 { this.index } else { this.index + 1 };
    if accessory_list[kind as usize].index != 0 {
        accessory_list[kind as usize].index = 0;
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    else {  return 0;  }
}

pub fn unit_access_sub_menu_y_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let unit = SortieSelectionUnitManager::get_unit();
    let kind = if this.index < 4 { this.index } else { this.index + 1 };
    if super::accessory::random_unit_accessory(unit, kind, false) {
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    return 0;
}

#[skyline::from_offset(0x01f86a50)]
fn help_set_unit(side: i32, unit: Option<&Unit>, relax: bool, reverse_rotation: bool, is_delay_load: bool, action: OptionalMethod, method_info: OptionalMethod);

#[skyline::from_offset(0x024622f0)]
fn create_basic_menu_content(method_info: OptionalMethod) -> &'static BasicMenuContent; 

#[skyline::from_offset(0x0245e330)]
fn transform_as_sub_menu(this: &BasicMenu<BasicMenuItem>, parent: &BasicMenu<BasicMenuItem>, parent_item: &BasicMenuItem, method_info: OptionalMethod);
