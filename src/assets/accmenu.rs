use unity::prelude::*;
use unity::system::List;
use engage::{
    gamedata::{accessory::*, unit::*, Gamedata},
    gameuserdata::GameUserData,
    menu::*,
    mess::Mess, 
    proc::Bindable, 
    sortie::SortieSelectionUnitManager
};
use engage::gamevariable::GameVariableManager;

static mut MENU_SELECT: i32 = 0;
static mut ACCESSORY_OPTIONS: Vec<i32> = vec![];
static mut CURRENT_ACCESSORIES: [i32; 16] = [0; 16];

const ACCESSORY_KIND_NAMES: [&str; 8] = ["Somniel", "Head", "Face", "Back", "Sommie", "Battle", "Hair", "Style"];
use engage::pad::Pad;
use engage::proc::ProcInst;
use engage::random::Random;
use engage::util::get_instance;
use unity::engine::Vector2;
use crate::CONFIG;
use super::{get_unit_outfit_mode, set_unit_outfit_mode, unit_get_accessory_list, ACCESSORY_COUNT};

pub fn unit_item_y_call(this: &BasicMenuItem, _optional_method: OptionalMethod) -> i32 {
    unsafe { MENU_SELECT = 0; }
    unit_accessory_sub_menu_create_bind(this);
    0x80
}

#[unity::class("App", "MapUnitCommandMenu")]
pub struct MapUnitCommandMenu{}

const ACCESSORY_SLOT_OFFSET: i32 = 3;
pub extern "C" fn map_unit_command_accessory(proc: &mut ProcInst) {
    let unit_command_menu = proc.cast_mut::<BasicMenu<BasicMenuItem>>();
    let x =  unit_command_menu.full_menu_item_list.iter_mut().enumerate()
        .find(|x| x.1.get_class().get_name() == "ItemMenuItem")
        .map(|(x, c)| (x, c.get_class_mut().clone()));
    if let Some((index, class)) = x {
        unit_command_menu.full_menu_item_list.insert(
                index as i32 + 1,
            MapCommandAccessory::set_new_menu_item(class)
        );
    }
}

pub trait UnitAccessoryMenuItem {
    fn get_name(this: &BasicMenuItem, method_info: OptionalMethod) -> &'static Il2CppString;
    fn map_command_help(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString { "".into() }
    fn r_call(_this: &BasicMenuItem, _method_info:  OptionalMethod) -> i32 { 0 }
    fn l_call(_this: &BasicMenuItem, _method_info:  OptionalMethod) -> i32 { 0 }
    fn a_call(_this: &BasicMenuItem, _method_info:  OptionalMethod) -> i32 { 0 }
    fn minus_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 { 0 }
    fn plus_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 { 0 }
    fn y_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 { 0 }
    fn x_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 { 0 }
    fn b_call(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        if let Some(parent) = this.menu.proc.parent.as_mut() {
            let menu = parent.cast_mut::<BasicMenu<BasicMenuItem>>();
            menu.open_anime();
        }
        0x201
    }
    fn custom_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 { 0 }
    fn build_attr(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 { 1 }
    fn set_new_menu_item(class: &mut Il2CppClass) -> &mut BasicMenuItem {
        let new_menu_item = il2cpp::instantiate_class::<BasicMenuItem>(class).unwrap();
        let method = class.get_methods().iter().find(|method| method.get_name() == Some(String::from(".ctor"))).unwrap();
        let ctor = unsafe {
            std::mem::transmute::<_, extern "C" fn(&BasicMenuItem, &MethodInfo) -> ()> (method.method_ptr,)
        };
        ctor(new_menu_item, method);
        new_menu_item.get_class_mut().get_virtual_method_mut("GetName")
            .map(|method| method.method_ptr = Self::get_name as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("LCall")
            .map(|method| method.method_ptr = Self::l_call as _);

        new_menu_item.get_class_mut().get_virtual_method_mut("RCall")
            .map(|method| method.method_ptr = Self::r_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("YCall")
            .map(|method| method.method_ptr = Self::y_call as _);

        new_menu_item.get_class_mut().get_virtual_method_mut("XCall")
            .map(|method| method.method_ptr = Self::x_call as _);

        new_menu_item.get_class_mut().get_virtual_method_mut("MinusCall")
            .map(|method| method.method_ptr = Self::minus_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("PlusCall")
            .map(|method| method.method_ptr = Self::plus_call as _);

        new_menu_item.get_class_mut().get_virtual_method_mut("ACall")
            .map(|method| method.method_ptr = Self::a_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("BCall")
            .map(|method| method.method_ptr = Self::b_call as _);

        new_menu_item.get_class_mut().get_virtual_method_mut("CustomCall")
            .map(|method| method.method_ptr = Self::custom_call as _);

        new_menu_item.get_class_mut().get_virtual_method_mut("GetCommandHelp")
            .map(|method| method.method_ptr = Self::map_command_help as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("GetMapAttribute")
            .map(|method| method.method_ptr = Self::build_attr as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("BuildAttribute")
            .map(|method| method.method_ptr = Self::build_attr as _);
        new_menu_item
    }
}

pub struct MapCommandAccessory;
impl UnitAccessoryMenuItem for MapCommandAccessory {
    fn get_name(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        Mess::get("MID_Hub_amiibo_Accessory_Trade")
    }
    fn map_command_help(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        Mess::get("MID_Hub_Mascot_Accessories_Choice")
    }
    fn a_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        unit_accessory_sub_menu_create_bind(this);
        0x81
    }
    fn minus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        if CONFIG.lock().unwrap().debug { class_change(this, false, None) }
        else { 0 }
    }
    fn plus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        if CONFIG.lock().unwrap().debug { class_change(this, true, None) }
        else { 0 }
    }
    fn b_call(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        unsafe { map_command_b_call(this, None) }
    }
    fn build_attr(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        if unit.status.value & 35184372088832 != 0 { 4 }
        else {
            if GameVariableManager::exist(format!("G_O{}", unit.person.pid).as_str()) { 1 }
            else { 4 }
        }
    }
}

fn unit_accessory_sub_menu_create_bind(menu: &BasicMenuItem){
    let is_sortie = GameUserData::get_sequence() != 3;
    let list = menu.menu.full_menu_item_list.get_class();
    let new_list = il2cpp::instantiate_class::<List<BasicMenuItem>>(list).unwrap();
    let count;
    if *ACCESSORY_COUNT.get().unwrap() >= 6  {
        new_list.items = Il2CppArray::new(10).unwrap();
        count = 7;
    }
    else {
        new_list.items = Il2CppArray::new(6).unwrap();
        count = 3;
    }
    let cock = get_base_menu_item_class();
    new_list.add(UnitOutfitMode::set_new_menu_item(cock.clone()));
    new_list.add(UnitHairColor::set_new_menu_item(cock.clone()));
    new_list.add(UnitEngageOutfit::set_new_menu_item(cock.clone()));

    for _x in 0..count {
        new_list.add(UnitAccessorySelect::set_new_menu_item(cock.clone()));
    }
    unsafe {
        let unit = get_unit();
        for x in 0..unit.accessory_list.unit_accessory_array.len() {
            if x >= 16 { break }
            CURRENT_ACCESSORIES[x] = unit.accessory_list.unit_accessory_array[x].index;
        }
    }
    let unit = get_unit();
    if unit.person.parent.index == 1 || GameVariableManager::exist(format!("G_A{}", unit.person.name.unwrap())) {
        new_list.add(RandomAppearance::set_new_menu_item(cock.clone()));
    }
    else {
        new_list.add(RandomAppearanceAlt::set_new_menu_item(cock.clone()));
    }

    let content =
        if is_sortie{ unsafe { create_basic_menu_content(None) } }
        else { unsafe { map_command_menu_content(None)}};
    let l = new_list.len() as i32;
    let new_menu =
    if is_sortie { BasicMenu::new(new_list, content) }
    else {
        let ctor_method = menu.menu.get_class().get_methods().iter().find(|method| method.get_name() == Some(String::from(".ctor"))).unwrap();

        let ctor = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<BasicMenuItem>, _, &BasicMenuContent, &MethodInfo)>(ctor_method.method_ptr) };
        let map = MapUnitCommandMenu::instantiate_as::<BasicMenu<BasicMenuItem>>().unwrap();
        ctor(map, new_list, content, ctor_method);
        map
    };
    let descs = new_menu.create_default_desc();
    new_menu.bind_parent_menu();
    new_menu.create_bind(menu.menu, descs, "");
    if is_sortie { new_menu.set_transform_as_sub_menu(menu.menu, menu);  }
    new_menu.set_show_row_num(l);
}

pub fn reload_unit_info(unit: &Unit) -> i32 {
    unsafe {
        help_set_unit(0, None, false, false, false, None, None);
        help_set_unit(1, None, false, false, false, None, None);
        help_set_unit(0, Some(unit), false, false, false, None, None);
    }
    let sequence = GameUserData::get_sequence();
    if sequence == 3 || sequence == 2 { unit.reload_actor(); }
    0x80
}


pub fn get_base_menu_item_class() -> &'static mut Il2CppClass {
    let menu = if GameUserData::get_sequence() != 3 {
        Il2CppClass::from_name("App", "UnitSelectSubMenu")
            .unwrap().get_nested_types().iter().find(|x| x.get_name() == "BaseMenuItem")
    }
    else {
        Il2CppClass::from_name("App", "MapUnitCommandMenu")
            .unwrap().get_nested_types().iter().find(|x| x.get_name() == "ItemMenuItem")
    }.unwrap();
    Il2CppClass::from_il2cpptype(menu.get_type()).unwrap()
}

pub fn unit_menu_item_y_call(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    unsafe { MENU_SELECT = 0; }
    unit_accessory_sub_menu_create_bind(this);
    0x80
}
pub struct RandomAppearanceAlt;
impl UnitAccessoryMenuItem for RandomAppearanceAlt {
    fn get_name(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        let unit = get_unit();
        if get_unit_outfit_mode(unit) & 64 != 0 { "Random Colors: On".into() } else { "Random Colors: Off".into() }
    }
    fn r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        set_unit_outfit_mode(unit, mode^64);
        this.rebuild_text();
        reload_unit_info(unit)
    }
    fn l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        Self::r_call(this, None)
    }
    fn a_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 { Self::r_call(_this, _method_info) }
}

pub struct RandomAppearance;
impl UnitAccessoryMenuItem for RandomAppearance {
    fn get_name(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        let unit = get_unit();
        let label = if GameUserData::get_sequence() != 3 { "Random: " } else { "" };
        let mode = get_unit_outfit_mode(unit) & 96;
        let ty =
            if mode == 32 { "Appearance" }
            else if mode == 64 { "Colors" }
            else if mode == 96 { "Both" }
            else { "None" };
        format!("{}{}", label, ty).into()
    }
    fn map_command_help(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        "Random Appearance / Colors Setting".into()
    }
    fn r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        let new_mode = mode & !96;
        if mode & 96 == 0 {
            set_unit_outfit_mode(unit, new_mode|32);
        }
        else if mode & 96 == 32 { set_unit_outfit_mode(unit, new_mode|64); }
        else if mode & 96 == 64 {
            set_unit_outfit_mode(unit, new_mode|96);

        }
        else if mode & 96 == 96 { set_unit_outfit_mode(unit, new_mode); }
        this.menu.full_menu_item_list.iter().for_each(|x| x.rebuild_text());
        this.rebuild_text();
        reload_unit_info(unit)
    }
    fn l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        let new_mode = mode & !96;
        if mode & 96 == 0 {
            set_unit_outfit_mode(unit, new_mode|96);
        }
        else if mode & 96 == 32 {
            set_unit_outfit_mode(unit, new_mode);
        }
        else if mode & 96 == 64 {
            set_unit_outfit_mode(unit, new_mode|32);
        }
        else if mode & 96 == 96 { set_unit_outfit_mode(unit, new_mode|64); }
        this.menu.full_menu_item_list.iter().for_each(|x| x.rebuild_text());
        reload_unit_info(unit)
    }
    fn a_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 { Self::r_call(_this, _method_info) }
}
pub struct UnitOutfitMode;
impl UnitAccessoryMenuItem for UnitOutfitMode {
    fn get_name(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit) & 3;
        if GameUserData::get_sequence() == 3 {
            match mode {
                1 => "Mode: Somniel",
                2 => "Mode: Battle",
                _ => "Mode: Default",
            }
        }
        else {
            match mode {
                1 => "Outfit Mode: Somniel",
                2 => "Outfit Mode: Battle",
                _ => "Outfit Mode: Default",
            }
        }.into()

    }
    fn r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        let res = mode & !3;
        if mode & 3 == 0 { set_unit_outfit_mode(unit, 1|res); }
        else if mode & 3 == 1 { set_unit_outfit_mode(unit, 2|res); }
        else if mode & 3 == 2 { set_unit_outfit_mode(unit, res); }
        this.menu.menu_item_list.iter().for_each(|item| item.rebuild_text());
        reload_unit_info(unit)
    }
    fn l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);

        let res = mode & !3;
        if mode & 3 == 0 { set_unit_outfit_mode(unit, 2|res); }
        else if mode & 3 == 1 { set_unit_outfit_mode(unit, res); }
        else if mode & 3 == 2 { set_unit_outfit_mode(unit, 1|res); }
        this.menu.menu_item_list.iter().for_each(|item| item.rebuild_text());
        reload_unit_info(unit)
    }
    fn a_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        Self::r_call(this, None)
    }
}
pub struct UnitAccessorySelect;
impl UnitAccessoryMenuItem for UnitAccessorySelect {
    fn get_name(this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        let accessory_index = accessory_slot_index(this.index);

        let unit = get_unit();
        let slot = &unit.accessory_list.unit_accessory_array[accessory_index as usize];
        if get_unit_outfit_mode(unit) & 32 != 0 && accessory_index == 0 {
            return if slot.index == 0 { "Default".into() } else { format!("Accessory #{}", slot.index).into() }
        }
        let s = if accessory_index >= 0 && accessory_index < 8 {
            ACCESSORY_KIND_NAMES[accessory_index as usize]
        } else { "" };
        if slot.index == 0 {
            return
            if GameUserData::get_sequence() == 3 { Mess::get_item_none() }
            else { format!("{}: {}", s, Mess::get_item_none()).into() };
        }
        if let Some(data) = AccessoryData::try_index_get(slot.index) {
            if GameUserData::get_sequence() == 3 { Mess::get(data.name) }
            else if s.len() > 2 { format!("{}: {}", s, Mess::get(data.name)).into() }
            else { format!("{}: {}", data.parent.index, Mess::get(data.name)).into() }
        }
        else { Mess::get_item_none() }
    }

    fn map_command_help(this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        let kind = accessory_slot_index(this.index);
        if get_unit_outfit_mode(get_unit()) & 32 != 0 && kind == 0 {
            return "Random Accessory slot.".into();
        }
        if kind >= 0 && kind < 8 {
            format!("{} Accessory Slot", ACCESSORY_KIND_NAMES[kind as usize]).into()
        }
        else { "Unit Accessory Slot".into() }
    }

    fn r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let kind = accessory_slot_index(this.index);
        let unit = get_unit();
        if super::accessory::next_unit_accessory(unit, kind, true) {
            this.rebuild_text();
            reload_unit_info(unit)
        }
        else { 0x800 }
    }
    fn l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let kind = accessory_slot_index(this.index);
        let unit = get_unit();
        if super::accessory::next_unit_accessory(unit, kind, false) {
            this.rebuild_text();
            reload_unit_info(unit)
        }
        else { 0x800 }
    }
    fn a_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();

        let is_sortie = GameUserData::get_sequence() != 3;
        if get_unit_outfit_mode(unit) & 32 != 0 && accessory_slot_index(this.index) == 0 {
            return 0x800;
        }
        let length;
        unsafe {
            MENU_SELECT = accessory_slot_index(this.index);
                ACCESSORY_OPTIONS = AccessoryData::get_list().unwrap().iter().filter(|x|{
                    x.kind == accessory_slot_index(this.index) && x.can_equip(unit)
                        && crate::assets::accessory::accessory_gender_check(x, unit)
                }).map(|x| x.parent.index).collect();
                if !ACCESSORY_OPTIONS.contains(&0) { ACCESSORY_OPTIONS.push(0); }
                CURRENT_ACCESSORIES[this.index as usize] = unit.accessory_list.unit_accessory_array[this.index as usize].index;
            length = ACCESSORY_OPTIONS.len();
        }
        let pos = unsafe {
            ACCESSORY_OPTIONS.iter()
                .position(|x| *x ==  unit.accessory_list.unit_accessory_array[MENU_SELECT as usize].index).unwrap_or(length - 1)
        };

        let list = this.menu.full_menu_item_list.get_class();
        let new_list = il2cpp::instantiate_class::<List<BasicMenuItem>>(list).unwrap();
        new_list.items = unsafe { Il2CppArray::new(ACCESSORY_OPTIONS.len()).unwrap() };
        let cock = get_base_menu_item_class();
        for _x in 0.. unsafe { ACCESSORY_OPTIONS.len() } {
            let menu = UnitAccessory::set_new_menu_item(cock.clone());
            menu.get_class_mut().get_virtual_method_mut("OnSelect").map(|m|
                m.method_ptr = unit_accessory_on_selected as _);
            new_list.add(menu);
        }
        let content =
            if is_sortie{ unsafe { create_basic_menu_content(None) } }
            else { unsafe { map_command_menu_content(None)}};
        let new_menu = BasicMenu::new(new_list, content);
        let descs = new_menu.create_default_desc();
        new_menu.create_bind(this.menu, descs, "");
        new_menu.bind_parent_menu();

        if is_sortie { new_menu.set_transform_as_sub_menu(this.menu, this);  }

        if is_sortie {
            if let Some(parent) = this.menu.proc.parent.as_ref() {
                let unit_select_menu = unsafe { std::mem::transmute::<_, &&BasicMenu<BasicMenuItem>>(parent) };
                if is_sortie { new_menu.set_transform_as_sub_menu(*unit_select_menu, this); }
            }
        }
        if is_sortie {
            let len = if new_list.len() < 15 { new_list.len() as i32 } else { 15 };
            new_menu.set_show_row_num(len);
        }
        else {
            let len = if new_list.len() < 12 { new_list.len() as i32 } else { 12 };
            new_menu.set_show_row_num(len);
        }
        set_scroll_select_index(new_menu, pos as i32, pos as i32);
        0x81
    }
    fn minus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let kind = accessory_slot_index(this.index);
        let unit = get_unit();
        let accessory = unsafe { unit_get_accessory_list(unit, None)};
        if accessory.unit_accessory_array[kind as usize].index != 0 {
            accessory.unit_accessory_array[kind as usize].index = 0;
            reload_unit_info(unit)
        }
        else { 0x800 }
    }
    fn y_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let kind = accessory_slot_index(this.index);
        if super::accessory::random_unit_accessory(unit, kind, false) {
            this.rebuild_text();
            reload_unit_info(unit)
        }
        else { 0x800 }
    }
}
pub struct UnitAccessory;
impl UnitAccessoryMenuItem for UnitAccessory {
    fn get_name(this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        unsafe {
            if let Some(data) = ACCESSORY_OPTIONS.get(this.index as usize)
                .and_then(|&index| AccessoryData::try_index_get(index))
            {
                if data.parent.index == 0 { Mess::get_item_none() }
                else if GameUserData::get_sequence() == 3 { Mess::get(data.name) }
                else { format!("{}: {}", data.parent.index, Mess::get(data.name)).into() }
            }
            else { Mess::get_item_none()  }
        }
    }
    fn a_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        unsafe {
            let kind = MENU_SELECT;
            let accessory = unit_get_accessory_list(unit, None);
            if CURRENT_ACCESSORIES[kind as usize] != accessory.unit_accessory_array[kind as usize].index {
                CURRENT_ACCESSORIES[kind as usize] = accessory.unit_accessory_array[kind as usize].index;
                0x100
            }
            else { 0x800 }
        }
    }

    fn minus_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let kind = unsafe { MENU_SELECT };
        let accessory = unsafe { unit_get_accessory_list(unit, None)};
        if accessory.unit_accessory_array[kind as usize].index == unsafe { CURRENT_ACCESSORIES[kind as usize] } { 0x800 }
        else {
            accessory.unit_accessory_array[kind as usize].index = unsafe { CURRENT_ACCESSORIES[kind as usize] };
            reload_unit_info(unit)
        }

    }
    fn b_call(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let kind = unsafe { MENU_SELECT };
        if unsafe { CURRENT_ACCESSORIES[kind as usize] } != unit.accessory_list.unit_accessory_array[kind as usize].index {
            unit.accessory_list.unit_accessory_array[kind as usize].index = unsafe { CURRENT_ACCESSORIES[kind as usize] };
            reload_unit_info(unit);
        }
        this.menu.get_class().get_virtual_method("CloseAnime").map(|method| {
            let close_anime = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<_>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime(this.menu, method.method_info);
        });
        if let Some(s) = this.menu.proc.parent.as_mut() {
            let s = s.cast_mut::<BasicMenu<BasicMenuItem>>();
            s.menu_item_list.iter().for_each(|i| {i.rebuild_text(); });
            s.open_anime();
        }
        0x201
    }
}

pub struct UnitEngageOutfit;
impl UnitAccessoryMenuItem for UnitEngageOutfit {
    fn get_name(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        let unit = get_unit();
        let mode = (get_unit_outfit_mode(unit) & 12) >> 2;
        if GameUserData::get_sequence() != 3 {
            if mode == 0 { "Engage Outfit: On" }
            else if mode == 1 { "Engage Outfit: Off" }
            else { "Engage Outfit: Emblem" }.into()
        }
        else {
            if mode == 0 { "Engage: On" }
            else if mode == 1 { "Engage: Off" }
            else { "Engage: Emblem" }.into()
        }
    }
    fn r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        let res = mode & !12;
        if ((mode >> 2) & 3) == 0 { set_unit_outfit_mode(unit, res|4); }
        else if ((mode >> 2) & 3)  == 1 { set_unit_outfit_mode(unit, 8 | res); }
        else if ((mode >> 2) & 3) == 2 { set_unit_outfit_mode(unit, res); }
        this.rebuild_text();
        if unit.status.value & 0x800000 != 0 { reload_unit_info(unit) } else { 0x80 }
    }
    fn l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);

        let res = mode & !12;
        if ((mode >> 2) & 3) == 0 { set_unit_outfit_mode(unit, res|8); }
        else if ((mode >> 2) & 3) == 1 { set_unit_outfit_mode(unit, res); }
        else if ((mode >> 2) & 3) == 2 { set_unit_outfit_mode(unit, 4|res); }
        this.rebuild_text();
        if unit.status.value & 0x800000 != 0 { reload_unit_info(unit) } else { 0x80 }
    }
    fn a_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        Self::r_call(this, None)
    }
    fn minus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        set_unit_outfit_mode(unit, mode & !12);
        this.rebuild_text();
        if unit.status.value & 0x800000 != 0 { reload_unit_info(unit) } else { 0x80 }
    }
}
pub struct UnitHairColor;
impl UnitAccessoryMenuItem for UnitHairColor {
    fn get_name(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit) & 16 != 0;
        if GameUserData::get_sequence() != 3 {
            if mode { "Custom Hair Color: On" }
            else { "Custom Hair Color: Off" }
        }
        else {
            if mode { "Hair Color: On" }
            else { "Hair Color: Off" }
        }.into()
    }
    fn r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        Self::l_call(this, None)
    }
    fn l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        set_unit_outfit_mode(unit, mode^16);
        this.rebuild_text();
        reload_unit_info(unit)
    }
    fn a_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit) & 16 != 0;
        if mode { unit_hair_color_a_call(this, None) }
        else { 0 }
    }
}
pub struct HairColor;
impl UnitAccessoryMenuItem for HairColor {
    fn get_name(this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        let shift = if this.index < 3 { (this.index + 1)* 8  } else { 8 };
        let color =
            match this.index {
                1 => "G",
                2 => "B",
                3 => "A",
                _ => "R",
            };
        let unit = get_unit();
        let value = ((get_unit_outfit_mode(unit) >> shift) & 0xFF) as u8;
        format!("{}: {}", color, value).into()
    }
    fn r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let shift = if this.index < 3 { (this.index + 1) * 8 } else { 8 };
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        let value = (mode >> shift) & 0xFF;
        let new_value = if value == 255 { 0 } else { value + 1 };
        let new_mode_value = ((new_value) << shift) | (mode & !(255 << shift));
        set_unit_outfit_mode(unit, new_mode_value);
        this.rebuild_text();
        0x80
    }
    fn l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let shift = if this.index < 3 { (this.index + 1) * 8 } else { 8 };
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        let value = (mode >> shift) & 0xFF;
        let new_value = if value == 0 { 255 } else { value - 1 };
        let new_mode_value = (new_value << shift) | (mode & !(255 << shift));
        set_unit_outfit_mode(unit, new_mode_value);
        this.rebuild_text();
        0x80
    }
    fn a_call(_this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let unit = get_unit();
        reload_unit_info(unit)
    }
    fn minus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        Self::l_call(this, None)
    }
    fn plus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        Self::r_call(this, None)
    }
    fn y_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let shift = if this.index < 3 { (this.index + 1) * 8 } else { 8 };
        let unit = get_unit();
        let mode = get_unit_outfit_mode(unit);
        let new_value = Random::get_system().get_value(255);
        let new_mode_value = (new_value << shift) | (mode & !(255 << shift));
        set_unit_outfit_mode(unit, new_mode_value);
        this.rebuild_text();
        reload_unit_info(unit)
    }
    fn b_call(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        if let Some(parent) = this.menu.proc.parent.as_ref() {
            let unit_select_menu = unsafe { std::mem::transmute::<_, &&BasicMenu<BasicMenuItem>>(parent) };
            let s =  unsafe { select_index_from_unit(unit_select_menu, None) };
            let current_unit_menu_item = unsafe { get_selected_item(unit_select_menu, s, None) };
            unit_accessory_sub_menu_create_bind( current_unit_menu_item);
            if let Some(child) = parent.child.as_ref() {
                let accessory_menu = unsafe { std::mem::transmute::<_, &&BasicMenu<BasicMenuItem>>(child) };
                set_scroll_select_index(accessory_menu, 1, 1);
            }
        }
        this.menu.get_class().get_virtual_method("CloseAnime").map(|method| {
            let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<_>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        });
        0x201
    }
    fn custom_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
        let pad_instance = get_instance::<Pad>();
        if pad_instance.npad_state.buttons.left() {
            Self::l_call(this, None);
            0
        }
        else if pad_instance.npad_state.buttons.right() {
            Self::r_call(this, None);
            0
        }
        else { 0 }
    }
}
pub fn unit_hair_color_a_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let is_sortie = GameUserData::get_sequence() != 3;
    let list = this.menu.full_menu_item_list.get_class();
    let new_list = il2cpp::instantiate_class::<List<BasicMenuItem>>(list).unwrap();
    new_list.items = Il2CppArray::new(3).unwrap();
    let cock = get_base_menu_item_class();
    for _x in 0..3{
        let menu = HairColor::set_new_menu_item(cock.clone());
        new_list.add(menu);
    }
    let content =
        if is_sortie{ unsafe { create_basic_menu_content(None) } }
        else { unsafe { map_command_menu_content(None)}};
    let new_menu = BasicMenu::new(new_list, content);
    let descs = new_menu.create_default_desc();
    if let Some(parent) = this.menu.proc.parent.as_ref() {
        this.menu.get_class().get_virtual_method("CloseAnime").map(|method| {
            let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<_>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        });
        let unit_select_menu = unsafe { std::mem::transmute::<_, &&BasicMenu<BasicMenuItem>>(parent) };
        new_menu.create_bind(*unit_select_menu, descs, "");
        new_menu.bind_parent_menu();
        if is_sortie { new_menu.set_transform_as_sub_menu(*unit_select_menu, this);  }
        new_menu.set_show_row_num(3);
    }
    0x81
}
fn get_unit() -> &'static mut Unit {
    if GameUserData::get_sequence() != 3 { SortieSelectionUnitManager::get_unit() }
    else { engage::mapmind::MapMind::get_unit() }
}

fn accessory_slot_index(menu_index: i32) -> i32 {
    if menu_index < 4 + ACCESSORY_SLOT_OFFSET  {
        menu_index - ACCESSORY_SLOT_OFFSET
    }
    else {
        menu_index - ACCESSORY_SLOT_OFFSET + 1
    }
}
fn set_scroll_select_index(menu: &BasicMenu<BasicMenuItem>, index: i32, scroll: i32) {
    let select = BasicMenuSelect::instantiate().unwrap();
    select.index = index;
    select.scroll = scroll;
    unsafe { restore_select(menu, select, None); }
}

fn unit_accessory_on_selected(basic_menu_item: &mut BasicMenuItem, optional_method: OptionalMethod) {
    let unit = get_unit();
    unsafe { basic_menuon_selected(basic_menu_item, optional_method); }
    if GameUserData::get_sequence() != 3 {
        let w = unsafe { get_w(basic_menu_item.menu.menu_content, None) };
        //let h = get_h(basic_menu_item.menu.menu_content, None);
        //println!("Index: {}, WH: {}/{}, Cursor_x/w: {}/{}", basic_menu_item.index, w, h, basic_menu_item.menu.menu_content.cursor.pos_x, basic_menu_item.menu.menu_content.cursor.pos_y);
        // let v = &mut basic_menu_item.menu.menu_content.cursor;
        if basic_menu_item.menu.menu_content.cursor.pos_x  > 0.0 {
            basic_menu_item.menu.menu_content.cursor.pos_x = 0.333 * w + 333.0;
        }
    }
    let s = basic_menu_item.index;
    let kind = unsafe { MENU_SELECT };
    if let Some(acc_index) = unsafe { ACCESSORY_OPTIONS.get(s as usize) } {
        if *acc_index != unit.accessory_list.unit_accessory_array[kind as usize].index {
            unit.accessory_list.unit_accessory_array[kind as usize].index = *acc_index;
            reload_unit_info(unit);
        }
    }
}
#[skyline::from_offset(0x2454810)]
fn get_w(content: &BasicMenuContent, method_info: OptionalMethod) -> f32;

#[skyline::from_offset(0x0245e660)]
fn get_h(content: &BasicMenuContent, method_info: OptionalMethod) -> f32;
pub fn class_change(_this: &BasicMenuItem, increase: bool, _method_info: OptionalMethod) -> i32 {
    let unit = get_unit();
    let mut job_index = unit.get_job().parent.index;
    let job_count = engage::gamedata::JobData::get_count();
    for _x in 0..job_count {
        if increase {
            job_index += 1;
            if job_index >= job_count { job_index = 1;}
        }
        else {
            job_index -= 1;
            if job_index == 0 { job_index = job_count - 1; }
        }
        if let Some(job) = engage::gamedata::JobData::try_index_get(job_index)
            .filter(|j| j.flag.value & 3 != 0)
        {
            unit.class_change(job);
            unit.update_weapon_mask();
            let sequence = GameUserData::get_sequence();
            if sequence == 3 || sequence == 2 { unit.reload_actor(); }
            crate::randomizer::person::unit::fixed_unit_weapon_mask(unit);
            crate::randomizer::person::unit::adjust_unit_items(unit);
            return reload_unit_info(unit);
        }
    }
    0x800
}

#[skyline::from_offset(0x01f86a50)]
fn help_set_unit(side: i32, unit: Option<&Unit>, relax: bool, reverse_rotation: bool, is_delay_load: bool, action: OptionalMethod, method_info: OptionalMethod);

#[skyline::from_offset(0x02454ae0)]
fn get_selected_item(basic_menu: &BasicMenu<BasicMenuItem>, index: i32, optional_method: OptionalMethod) -> &'static BasicMenuItem;

#[skyline::from_offset(0x024622f0)]
fn create_basic_menu_content(method_info: OptionalMethod) -> &'static BasicMenuContent; 

#[skyline::from_offset(0x245d0a0)]
fn restore_select(basic_menu: &BasicMenu<BasicMenuItem>, select: &BasicMenuSelect, method_info: OptionalMethod);

#[skyline::from_offset(0x0245ce90)]
fn set_select_index(basic_menu: &BasicMenu<BasicMenuItem>, scroll_index: i32, method_info: OptionalMethod);

#[skyline::from_offset(0x0245e330)]
fn transform_as_sub_menu(this: &BasicMenu<BasicMenuItem>, parent: &BasicMenu<BasicMenuItem>, parent_item: &BasicMenuItem, method_info: OptionalMethod);

#[skyline::from_offset(0x0202b7a0)]
fn map_command_menu_content(method_info: OptionalMethod) -> &'static BasicMenuContent;

#[skyline::from_offset(0x0245ce70)]
fn select_index_from_unit(basic_menu: &BasicMenu<BasicMenuItem>, method_info: OptionalMethod) -> i32;

pub fn unit_access_map_command_help(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString { "".into() }

#[skyline::from_offset(0x2466570)]
fn basic_menuon_selected(basic_menu_item: &BasicMenuItem, method: OptionalMethod);

#[skyline::from_offset(0x01e48e70)]
fn map_command_b_call(basic_menu_item: &BasicMenuItem, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x02464770)]
fn get_pos(basic_menu: *const u8, method: OptionalMethod) -> Vector2<f32>;