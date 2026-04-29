use super::*;
use std::sync::{OnceLock};
use engage::{
    mess::Mess, gameuserdata::GameUserData,
    gamedata::{Gamedata, JobData},
    menu::{BasicMenuItemAttribute, BasicMenuResult},
};
use unity::{prelude::*, engine::Color};
use crate::{
    DVCVariables, DVCConfig, enums::{MPIDS, RINGS},
    randomizer::{data::GameData, job::single::get_next_class},
};
pub static DVC_CONFIG_MENU_ITEM_CLASS: OnceLock<&'static Il2CppClass> = OnceLock::new();

#[unity::class("", "ConfigBasicMenuItem")]
pub struct DVCConfigMenuItem {
    pub menu: &'static mut BasicMenu<DVCConfigMenuItem>, // 0
    pub menu_content: *const u8,    // ConfigMenuContent    // 0x8
    pub name: &'static Il2CppString,    // 0x10
    pub index: i32, // 0x18
    pub full_index: i32,    //0x1c
    pub attribute: i32, //0x20
    pub cursor_color: Color,    //0x24
    pub active_text_color: Color,   //0x34
    pub inactive_text_color: Color, //0x44
    pub config_method: i32, //0x54
    pub title: &'static mut Il2CppString,   //0x58
    pub command_text: &'static mut Il2CppString,    //0x60
    pub help_text: &'static mut Il2CppString,   //0x68
    pub is_arrow: bool,
    pub is_command: bool,
    pub dvc_command: bool,
    pub padding: u8,
    pub gauge_ratio: f32,
    pub menu_info: u64,
    pub menu_id: i32,
    pub dvc_value: i32,
    pub menu_item_kind: DVCMenuItemKind,
    pub menu_kind: DVCMenu,
}

impl DVCConfigMenuItem {
    #[unity::class_method(2,  ConfigBasicMenuItem)] pub fn on_select_(&self); // Offset: 0x2536FD0 Flags: 0
    #[unity::class_method(3,  ConfigBasicMenuItem)] pub fn on_deselect_(&self); // Offset: 0x25370F0 Flags: 0
    // #[unity::class_method(5,  ConfigBasicMenuItem)] pub fn init_color(&self); // Offset: 0x2536EB0 Flags: 0
    #[unity::class_method(6,  ConfigBasicMenuItem)] pub fn set_title_text(&self, text: &Il2CppString); // Offset: 0x2537130 Flags: 0
    #[unity::class_method(7,  ConfigBasicMenuItem)] pub fn update_text(&self); // Offset: 0x2537000 Flags: 0
    #[unity::class_method(12, ConfigBasicMenuItem)] pub fn ctor(&self); // Offset: 0x25379A0 Flags: 0

    #[unity::class_method(34, BasicMenuItem)] pub fn on_select_base(&self); // Offset: 0x2466570 Flags: 0
    #[unity::class_method(35, BasicMenuItem)] pub fn on_deselect_base(&self); // Offset: 0x24666D0 Flags: 0

    pub fn new_kind(kind: DVCMenuItemKind) -> &'static mut Self {
        let item = DVC_CONFIG_MENU_ITEM_CLASS.get_or_init(|| Self::create_class())
            .instantiate_as::<DVCConfigMenuItem>().unwrap();

        item.ctor();
        item.config_method = 0;
        match kind {
            DVCMenuItemKind::Variable(var) => { item.dvc_value = var.get_value(); }
            DVCMenuItemKind::Gauge(var) => {
                let v = var.get_value();
                item.config_method = 1;
                item.gauge_ratio = v as f32 / 100.0;
                item.dvc_value = v;
            }
            DVCMenuItemKind::Flag(flag) => {
                let v = flag.get_value() as i32;
                /*
                if !flag.need_confirm_to_change() && v != 0 {
                    item.is_command = true;
                }
                */
                item.dvc_value = v;
            }
            DVCMenuItemKind::Command(_) => { item.is_command = true; }
            DVCMenuItemKind::SingleJob => {
                let current = DVCVariables::SingleJob.get_value();
                if current == 1 || JobData::try_get_hash(current).is_some() { item.dvc_value = current; }
                else { item.dvc_value = get_next_class(0, true); }
            }
            _ => {}
        }
        item.menu_item_kind = kind;
        item.update_config_text();
        item
    }
    pub fn new_recruitment_item(order: RecruitmentOrder, index: i32) -> &'static mut Self {
        let item = DVC_CONFIG_MENU_ITEM_CLASS.get_or_init(|| Self::create_class())
            .instantiate_as::<DVCConfigMenuItem>().unwrap();
        item.ctor();
        item.config_method = 0;
        item.index = index;
        if GameUserData::get_sequence() != 0 { item.help_text = "View the set unit recruitment order.".into(); } else {
            match order {
                RecruitmentOrder::Unit => {
                    if index < 32 { item.padding = DVCConfig::get().unit1[index as usize]; }
                    else { item.padding = DVCConfig::get().unit2[index as usize - 32]; }
                    item.help_text = "Assign an playable unit to swap recruitment positions.".into();
                    item.title = Mess::get(MPIDS[index as usize]);
                }
                RecruitmentOrder::UnitCustom => {
                    item.padding = unsafe { CUSTOM_RECRUITMENT_ORDER[index as usize] };
                    item.title = Mess::get(MPIDS[index as usize]);
                    if is_required(index) { item.help_text = "This character's recruitment position cannot be changed.".into(); }
                    else { item.help_text = "Assign an playable unit to swap recruitment positions.".into(); }
                }
                RecruitmentOrder::Emblem => {
                    item.padding = DVCConfig::get().emblem[index as usize];
                    if GameUserData::get_sequence() == 0 { item.help_text = "Assign an emblem to swap recruitment position.".into(); }
                    item.title = Mess::get(format!("MGID_{}", RINGS[index as usize]));
                }
            }
        }
        item.menu_item_kind = DVCMenuItemKind::Order(order);
        item.update_config_text();
        item
    }
    pub fn create_class() -> &'static Il2CppClass {
        let klass1 = Il2CppClass::from_name("", "VolumeVoiceMenuItem").unwrap().clone();
        let klass2 = ConfigBasicMenuItem::class();
        klass1._2.actual_size = size_of::<DVCConfigMenuItem>() as u32;
        klass1._2.instance_size = size_of::<DVCConfigMenuItem>() as u32;
        let vtable_1 = klass1.get_vtable_mut();
        let vtable_2 = klass2.get_vtable();
        for x in 0..28 { vtable_1[x] = vtable_2[x]; }
        vtable_1[8].method_ptr = Self::build_attribute as _;
        vtable_1[12].method_ptr = Self::on_select as _;
        vtable_1[18].method_ptr = Self::a_call as _;
        vtable_1[19].method_ptr = Self::b_call as _;
        vtable_1[24].method_ptr = Self::plus_call as _;
        vtable_1[25].method_ptr = Self::minus_call as _;
        vtable_1[26].method_ptr = Self::custom_call as _;
        klass1
    }
    pub fn on_select(this: &mut DVCConfigMenuItem, _optional_method: OptionalMethod) {
        this.on_select_base();
        match &this.menu_item_kind {
            DVCMenuItemKind::Command(_) => {
                this.is_arrow = false;
                this.is_command = true;
            }
            DVCMenuItemKind::Order(_) => { this.is_arrow = DVCVariables::is_main_menu(); }
            _ => {
                if this.menu_kind == DVCMenu::ReadOnly { this.is_arrow = false; }
                else { this.is_arrow = true;}
            }
        }
        this.update_config_text();
    }
    pub fn a_call(this: &mut DVCConfigMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
        let kind = this.menu_item_kind.clone();
        kind.a_call(this)
    }
    pub fn custom_call(this: &mut DVCConfigMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
        let kind = this.menu_item_kind.clone();
        kind.custom_call(this)
    }
    pub fn build_attribute(this: &mut DVCConfigMenuItem, _optional_method: OptionalMethod) -> BasicMenuItemAttribute {
        let kind = this.menu_item_kind.clone();
        match kind {
            DVCMenuItemKind::SingleJob => {
                if DVCVariables::ClassMode.get_value() == 2 { BasicMenuItemAttribute::Enable }
                else { BasicMenuItemAttribute::Hide }
            }
            DVCMenuItemKind::Menu(menu) => menu.build_attribute(this),
            DVCMenuItemKind::Variable(var) => var.build_attribute(),
            DVCMenuItemKind::Gauge(var) => var.build_attribute(),
            DVCMenuItemKind::Flag(flag) => flag.build_menu_item(),
            DVCMenuItemKind::Command(cmd) => cmd.build_attribute(this),
            _ => { BasicMenuItemAttribute::Enable }
        }
    }
    pub fn plus_call(this: &mut DVCConfigMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
        let kind = this.menu_item_kind.clone();
        match kind {
            DVCMenuItemKind::Command(cmd) => cmd.plus_call(this),
            DVCMenuItemKind::Order(order) => order.plus_call(this),
            _ => BasicMenuResult::new()
        }
    }
    pub fn minus_call(this: &mut DVCConfigMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
        let kind = this.menu_item_kind.clone();
        match kind {
            DVCMenuItemKind::Order(order) => order.minus_call(this),
            _ => BasicMenuResult::new()
        }
    }
    pub fn update_config_text(&mut self) {
        CONFIG_TEXT.get_or_init(|| DVCConfigText::init()).set_text(self);
        self.update_text();
    }
    pub fn b_call(this: &mut DVCConfigMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
        DVCMenu::save_select(this);
        if let Some(previous) = this.menu_kind.get_previous(){
            previous.rebuild_menu(this, false);
            BasicMenuResult::new().with_se_cancel(true)
        }
        else {
            if !DVCVariables::is_main_menu() {
                if unsafe { MENU_SELECT[19] } != DVCVariables::BondRingSkillRate.get_value(){ GameData::get().update_bond_ring(); }
                if DVCVariables::ClassMode.get_value() == 2 && !DVCFlags::SingleJobEnabled.get_value() {
                    DVCVariables::ClassMode.set_value( unsafe { MENU_SELECT[20] } );
                }
            }
            DVCMenu::reset_select();
            if let Some(parent) = this.menu.get_parent() {
                if let Some(method) = parent.klass.get_virtual_method("OpenAnime") {
                    let call = unsafe { std::mem::transmute::<_, fn(&ProcInst, &MethodInfo)> (method.method_ptr) };
                    call(parent, method.method_info);
                }
            }
            BasicMenuResult::close_cancel()
        }
    }
}

