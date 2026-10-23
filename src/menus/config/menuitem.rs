use engage::{
    root::configbasicmenuitem::*,
    app::{
        jobdata::*,
        GameMessage, Mess, 
        IStructBase, IStructData_1Methods,
    },
    nn::hid::NpadButton,
    root::{ConfigBasicMenuItem, IConfigBasicMenuItemMethods},
    system::Action,
};
mod order;
mod submenu;
mod command;
mod output;
#[cfg(feature = "debug")] mod debug;
mod job;

use super::*;
pub use {submenu::DVCMenu, order::RecruitmentOrder, command::DVCCommand, job::DVCJobItem};
use crate::dvc::DVC;

const KINDS: [&str; 8] = ["Sword", "Lance", "Axe", "Bow", "Dagger", "Magic", "Fist", "Special"];

use crate::gamedata::{GameData, ENCHANTER, LUEUR_CLASS, MAGE_CANNON, MONSTER_CLASS};

#[unity::inject(namespace="DVC", name="DVCBasicMenuItem", parent=ConfigBasicMenuItem)]
pub struct DVCBasicMenuItem {
    pub value: i32,
    pub value2: i32,
    pub menu_item_type: i32,
    pub menu_type: i32,
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum DVCMenuItemKind {
    Menu(DVCMenu),
    Variable(DVCVariable),
    Gauge(DVCVariable),
    Flag(DVCFlags),
    Command(DVCCommand),
    Order(RecruitmentOrder),
    Job(DVCJobItem),
}

impl DVCMenuItemKind {
    pub fn to_i32(self) -> i32 {
        match self {
            DVCMenuItemKind::Variable(variable) => { variable as i32 }
            DVCMenuItemKind::Gauge(variable) => { variable as i32 | (1 << 8) }
            DVCMenuItemKind::Flag(flag) => { flag as i32 | (2 << 8) }
            DVCMenuItemKind::Menu(menu) => { menu as i32 | (3 << 8) }
            DVCMenuItemKind::Command(command) => { command as i32 | (4 << 8) }
            DVCMenuItemKind::Order(order) => { order as i32 | (5 << 8) }
            DVCMenuItemKind::Job(mode) => { mode as i32 | (6 << 8) }
            #[cfg(feature = "debug")] DVCMenuItemKind::Debug(debug) => { debug as i32 | (7 << 8) }
        }
    }
    pub fn from_i32(value: i32) -> Option<DVCMenuItemKind> {
        let v = value & 255;
        let kind = value >> 8;
        match kind {
            0 => DVCVariable::from(v).map(|var| Self::Variable(var)),
            1 => DVCVariable::from(v).map(|var| Self::Gauge(var)),
            2 => DVCFlags::from(v).map(|flags| Self::Flag(flags)),
            3 => DVCMenu::from(v).map(|menu| Self::Menu(menu)),
            4 => DVCCommand::from(v).map(|cmd| Self::Command(cmd)),
            5 => RecruitmentOrder::from(v).map(|order| Self::Order(order)),
            6 => DVCJobItem::from(v).map(|job| Self::Job(job)),
            _ => None,
        }
    }
}

impl DVCBasicMenuItem {
    pub fn get_menu_item_type(self) -> DVCMenuItemKind { DVCMenuItemKind::from_i32(self.menu_item_type()).unwrap() }
    pub fn update_text1(self) { 
        CONFIG_TEXT.get_or_init(|| DVCConfigText::init()).set_text(self);
        self.update_text();
    }
}
#[unity::injected_methods]
impl DVCBasicMenuItem {
    #[override_virtual(name="BuildAttribute")]
    pub fn build_attribute(self) -> BasicMenuItem_Attribute {
        match self.get_menu_item_type() {
            DVCMenuItemKind::Command(command) =>command.build_attribute(),
            DVCMenuItemKind::Menu(menu) => menu.build_attribute(),
            DVCMenuItemKind::Variable(var) => var.build_attribute(),
            DVCMenuItemKind::Gauge(var) => var.build_attribute(),
            DVCMenuItemKind::Flag(flag) => flag.build_attribute(),
            DVCMenuItemKind::Order(order) => order.build_attribute(self),
            DVCMenuItemKind::Job(job) => job.build_attribute(),
        }
    }
    #[override_virtual(name="OnSelect")]
    pub fn on_select1(self) {
        unsafe { BasicMenuItem::on_select(self) };
        match self.get_menu_item_type() {
            DVCMenuItemKind::Command(_) => {
                self.set_m_is_arrow(false);
                self.set_m_is_command_icon(true);
            }
            DVCMenuItemKind::Order(_) => { self.set_m_is_arrow(DVCVariables::is_main_menu()); },
            DVCMenuItemKind::Job(DVCJobItem::UnitJob) => { self.set_m_is_arrow(DVCVariables::is_main_menu()); }
            _ => { self.set_m_is_arrow(true); }
        }
        let attribute = self.get_attribute().value;
        if attribute & 2 != 0 { self.set_m_is_arrow(false); }
        self.update_text1();
    }
    #[override_virtual(name="ACall")]
    pub fn a_call(self) -> BasicMenu_Result {
        let kind = self.get_menu_item_type();
        let menu = self.get_dvc_menu();
        match kind {
            DVCMenuItemKind::Command(command) => { command.a_call(self) }
            DVCMenuItemKind::Menu(next_menu) => {
                menu.rebuild_menu(next_menu);
                BasicMenu_Result::se_cursor()
            }
            DVCMenuItemKind::Job(job) => { job.a_call(self) }
            DVCMenuItemKind::Variable(var) => {
                if var.can_a_call() && self.m_is_command_icon() {
                    var.set_value(self.value());
                    self.set_m_is_command_icon(false);
                    self.update_text1();
                    BasicMenu_Result::se_decide()
                }
                else { BasicMenu_Result::pass() }
            }
            _ => { BasicMenu_Result::pass() }
        }
    }
    #[override_virtual(name="PlusCall")]
    pub fn plus_call(self) -> BasicMenu_Result {
        match self.get_menu_item_type() {
            DVCMenuItemKind::Order(order) => {
                if DVCVariables::is_main_menu() { order.plus_call(self) } else { BasicMenu_Result::pass() }
            }
            DVCMenuItemKind::Variable(DVCVariable::InteractSetting) => {
                if !DVCVariables::is_main_menu() {
                    if let Some(text) = break_dialog_text(false) {
                        GameMessage::create_key_wait(self.get_dvc_menu(), text.as_str());
                        BasicMenu_Result::se_cursor()
                    }
                    else { BasicMenu_Result::se_miss() }
                }
                else { BasicMenu_Result::pass() }
            }
            DVCMenuItemKind::Command(command) => { command.plus_call(self) }
            DVCMenuItemKind::Job(job) => { job.plus_call(self) }
            _ => { BasicMenu_Result::pass() }
        }
    }
    #[override_virtual(name="MinusCall")]
    pub fn minus_call(self) -> BasicMenu_Result {
        match self.get_menu_item_type() {
            DVCMenuItemKind::Order(order) => { if DVCVariables::is_main_menu() { order.minus_call(self) } else { BasicMenu_Result::pass() } }
            DVCMenuItemKind::Variable(DVCVariable::InteractSetting) => {
                if !DVCVariables::is_main_menu() {
                    if let Some(text) = break_dialog_text(true) {
                        GameMessage::create_key_wait(self.get_dvc_menu(), text.as_str());
                        BasicMenu_Result::se_cursor()
                    }
                    else { BasicMenu_Result::se_miss() }
                }
                else { BasicMenu_Result::pass() }
            }
            DVCMenuItemKind::Job(job) => { job.minus_call(self) }
            _ => { BasicMenu_Result::pass() }
        }
    }
    #[override_virtual(name="CustomCall")]
    pub fn custom_call(self) -> BasicMenu_Result {
        let kind = self.get_menu_item_type();
        let menu = self.get_dvc_menu();
        match kind {
            DVCMenuItemKind::Variable(var) => {
                if let Some(increase) = Self::get_change(true) {
                    let current = self.value();
                    if var == DVCVariable::ClassMode {
                        if DVCVariables::is_main_menu() {
                            let v = DVCVariable::ClassMode.increment(current, increase);
                            menu.set_current_value(var, v);
                            self.set_value(v);
                        }
                        else if current == 6 {
                            self.set_m_is_arrow(false);
                            return BasicMenu_Result::pass();
                        }
                        else {
                            let new = (current + if increase { 1 } else { 5 }) % 6;
                            menu.set_current_value(var, new);
                            self.set_value(new);
                            let initial = DVCVariable::ClassMode.get_setting_value();
                            self.set_m_is_command_icon(new != initial);
                        }
                    }
                    else if var.is_adjustable() {
                        let v = var.increment(self.value(), increase);
                        self.set_value(v);
                        menu.set_current_value(var, v);
                    }
                    else { self.set_m_is_arrow(false); }
                    self.update_text1();
                    if var.rebuild_menu_custom_call() { menu.rebuild() }
                    BasicMenu_Result::se_cursor()
                }
                else { BasicMenu_Result::pass() }
            }
            DVCMenuItemKind::Gauge(gauge) => {
                if let Some(increase) = Self::get_change(true){
                    let v = self.value();
                    let v2 = gauge.increment(v, increase);
                    self.set_m_gauge_ratio((v2 as f32) / 100.0);
                    self.set_value(v2);
                    self.update_text1();
                    BasicMenu_Result::se_cursor()
                }
                else { BasicMenu_Result::pass() }
            }
            DVCMenuItemKind::Flag(flag) => {
                if Self::get_change(true).is_some(){
                    if flag.is_adjustable() {
                        let current = self.value() != 0;
                        self.set_value(if current { 0 } else { 1 });
                        flag.set_value(!current);
                        self.update_text1();
                        if flag.rebuild_menu_custom_call() { menu.rebuild(); }
                        BasicMenu_Result::se_cursor()
                    }
                    else {
                        self.set_m_is_arrow(false);
                        BasicMenu_Result::se_miss()
                    }
                }
                else { BasicMenu_Result::pass() }
            }
            DVCMenuItemKind::Order(order) => { 
                if DVCVariables::is_main_menu() { order.custom_call(self) } 
                else { BasicMenu_Result::pass() }
            }
            DVCMenuItemKind::Job(job) => {
                if job == DVCJobItem::UnitJob && !DVCVariables::is_main_menu() { BasicMenu_Result::pass() } 
                else { job.custom_call(self) }
            }
            _ => BasicMenu_Result::pass(),
        }
    }
    #[override_virtual(name="XCall")]
    pub fn x_call(self) -> BasicMenu_Result {
        match self.get_menu_item_type() {
            DVCMenuItemKind::Job(job) => { job.x_call(self) }
            _ => BasicMenu_Result::pass(),
        }
    }
}
impl DVCBasicMenuItem {
    pub fn get_change(trigger: bool) -> Option<bool> {
        let left;
        let right;
        if trigger {
            left = Pad::is_trigger(NpadButton::left());
            right = Pad::is_trigger(NpadButton::right());
        }
        else {
            left = Pad::is_button(NpadButton::left());
            right = Pad::is_button(NpadButton::right());
        }
        if left && !right { Some(false) }
        else if right && !left { Some(true) }
        else { None }
    }
    pub fn get_dvc_menu(self) -> DVCSettingMenu { unsafe { IBasicMenuItemMethods::get_menu(self).cast() } }
    pub fn from_type(ty: DVCMenuItemKind, idx: i32) -> Self {
        let item = Self::instantiate().unwrap();
        IConfigBasicMenuItemMethods::ctor(item);
        IBasicMenuItemMethods::set_item_index(item, idx);
        item.set_menu_item_type(ty.to_i32());
        item.set_m_config_method(ConfigBasicMenuItem_ConfigMethodKind::switch());
        match ty {
            DVCMenuItemKind::Order(order) => {
                item.set_value2(idx);
                order.set_initial_value(item);
            }
            DVCMenuItemKind::Gauge(gauge) => {
                item.set_m_config_method(ConfigBasicMenuItem_ConfigMethodKind::gauge());
                item.set_value(gauge.get_setting_value());
                item.set_m_gauge_ratio(gauge.get_setting_value() as f32 / 100.0);
            }
            DVCMenuItemKind::Flag(flag) => { item.set_value(flag.get_setting_value() as i32); }
            DVCMenuItemKind::Variable(var) => { item.set_value(var.get_setting_value()); }
            DVCMenuItemKind::Job(job) => {
                let v =
                    match job {
                        DVCJobItem::SingleClass => DVCVariable::SingleJob.get_setting_value(),
                        DVCJobItem::GenderM => DVCVariable::GenderClassM.get_setting_value(),
                        DVCJobItem::GenderF => DVCVariable::GenderClassF.get_setting_value(),
                        DVCJobItem::UnitJob => { DVC::get().unit_job[idx as usize] },
                    };
                item.set_value(v);
                item.set_m_is_command_icon(false);
            }
            _ => {}
        }
        CONFIG_TEXT.get_or_init(|| DVCConfigText::init()).set_text(item);
        item
    }
    pub fn as_basic_menu_item(self) -> BasicMenuItem { unsafe { self.cast() } }
}
#[skyline::from_offset(0x2466570)]
fn basic_menu_item_on_select(this: DVCBasicMenuItem, optional_method: OptionalMethod);

fn break_dialog_text(disadvantage: bool) -> Option<String> {
    let interaction = GameData::get().get_interaction();
    let mut str = String::new();
    [1, 2, 3, 4, 5, 6, 8, 9].into_iter().enumerate().for_each(|(index, kind)| {
        let flag = if disadvantage { interaction[kind] >> 10 } else { interaction[kind] };
        let mut kind_str = String::new();
        if flag & 1023 == 1023 { kind_str += "All"; }
        else {
            for i in 0..5 {
                if flag & (2 << i) != 0 {
                    if !kind_str.is_empty() { kind_str += "/"; }
                    kind_str +=  Mess::get(format!("MID_H_INFO_WLV_{}", KINDS[i])).to_rust_string().as_str();
                }
            }
            if flag & 64 != 0 {
                if !kind_str.is_empty() { kind_str += "/"; }
                kind_str += Mess::get(format!("MID_H_INFO_WLV_{}", KINDS[5])).to_rust_string().as_str();
            }
            if flag & 256 != 0 {
                if !kind_str.is_empty() { kind_str += "/"; }
                kind_str += Mess::get(format!("MID_H_INFO_WLV_{}", KINDS[6])).to_rust_string().as_str();
            }
            if flag & 512 != 0 {
                if !kind_str.is_empty() { kind_str += "/"; }
                kind_str += Mess::get(format!("MID_H_INFO_WLV_{}", KINDS[7])).to_rust_string().as_str();
            }
        }
        if !kind_str.is_empty() {
            if !str.is_empty() { str += "\n"; }
            str += format!("{} {} {}",
                Mess::get(format!("MID_H_INFO_WLV_{}", KINDS[index])),
                if disadvantage { "broken by:" } else { "breaks:" },
                kind_str
            ).as_str();
        }
    });
    if !str.is_empty() { Some(str) } else { None }
}