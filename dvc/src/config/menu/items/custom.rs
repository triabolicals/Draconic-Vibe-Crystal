use engage::gamedata::{GodData, PersonData};
use engage::gamevariable::GameVariableManager;
use engage::menu::menu_item::config::ConfigBasicMenuItem;
use engage::menu::menus::config::ConfigMenu;
use engage::proc::Bindable;
use crate::CONFIG;
use crate::enums::{EMBLEM_GIDS, MPIDS, PIDS, RINGS};
use crate::randomizer::data::GameData;
use crate::randomizer::DeploymentConfig;
use super::*;

pub static CUSTOM_RECRUITMENT_ITEM: OnceLock<&'static Il2CppClass> = OnceLock::new();
pub static mut CUSTOM_RECRUITMENT_ORDER: [u8; 42] = [255; 42];

#[unity::class("", "ConfigBasicMenuItem")]
pub struct DVCCustomRecruitmentMenuItem {
	pub menu: &'static mut engage::menu::BasicMenu<DVCCustomRecruitmentMenuItem>, // 0
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
	pub mode: u8,
	pub padding: u8,
	pub gauge_ratio: f32,
}
impl ConfigMenuItem for DVCCustomRecruitmentMenuItem {}
impl DVCCustomRecruitmentMenuItem {
	pub fn create_class() -> &'static Il2CppClass {
		let klass1 = Il2CppClass::from_name("", "VolumeVoiceMenuItem").unwrap().clone();
		let klass2 = ConfigBasicMenuItem::class();
		let vtable_1 = klass1.get_vtable_mut();
		let vtable_2 = klass2.get_vtable();
		for x in 0..28 { vtable_1[x] = vtable_2[x]; }
		vtable_1[8].method_ptr = Self::build_attribute as _;
		vtable_1[12].method_ptr = Self::on_select as _;
		vtable_1[18].method_ptr = Self::a_call as _;
		vtable_1[24].method_ptr = Self::plus_call as _;
		vtable_1[25].method_ptr = Self::minus_call as _;
		vtable_1[26].method_ptr = Self::custom_call as _;
		vtable_1[27].method_ptr = Self::init_content as _;
		klass1
	}
	pub fn bind<B: Bindable>(parent: &mut B, mode: u8) {
		ConfigMenu::create_bind(parent);
		let mut child = parent.get_child();
		let child2 = child.as_mut();
		let config_menu = child2.unwrap().cast_mut::<ConfigMenu<DVCCustomRecruitmentMenuItem>>();
		config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
		config_menu.full_menu_item_list.clear();
		let n_items = crate::utils::get_total_unit_emblems(mode == 1);
		if mode == 1 { TitleBar::open_header(draconic_vibe_name(), "Custom Emblem Recruitment Order", ""); }
		else if mode == 2 {
			let max = GameData::get().playables.len() as u8;
			for x in 0..n_items {
				unsafe {
					if is_required(x) { CUSTOM_RECRUITMENT_ORDER[x as usize] = x as u8;}
					else if CUSTOM_RECRUITMENT_ORDER[x as usize] > max { CUSTOM_RECRUITMENT_ORDER[x as usize] = max; }
				}
			}
			unsafe { CUSTOM_RECRUITMENT_ORDER[41] = max; }
			TitleBar::open_header(draconic_vibe_name(), "Custom Unit Recruitment Order with Custom Units", "");
		}
		else { TitleBar::open_header(draconic_vibe_name(), "Custom Unit Recruitment Order", ""); }
		for x in 0..n_items { config_menu.add_item(Self::new(x,mode)); }
		if GameUserData::get_sequence() == 0 {
			if let Some(key_help) = TitleBar::get_instance().current_title.as_ref().map(|v| v.key_help) {
				key_help.help_object[10].set_active(true);
				key_help.help_object[11].set_active(true);
				key_help.set_text(&key_help.help_object[10], "Randomize".into());
				key_help.set_text(&key_help.help_object[11], "Original".into());
			}
		}
	}
	pub fn add_items(config_menu: &mut ConfigMenu<DVCCustomRecruitmentMenuItem>, mode: u8) {
		config_menu.full_menu_item_list.clear();
		let n_items = crate::utils::get_total_unit_emblems(mode == 1);
		for x in 0..n_items { config_menu.add_item(Self::new(x,mode)); }
	}
	pub fn new(index: i32, mode: u8) -> &'static mut DVCCustomRecruitmentMenuItem {
		let item = CUSTOM_RECRUITMENT_ITEM.get_or_init(||Self::create_class()).instantiate_as::<DVCCustomRecruitmentMenuItem>().unwrap();
		unsafe { config_basic_menu_item_ctor(item, None); }
		item.config_method = 0;
		item.is_command = false;
		item.mode = mode;
		item.index = index;
		if mode == 1 {
			if let Some(v) = GodData::get(EMBLEM_GIDS[index as usize]).map(|g| Mess::get(g.mid)) { item.title = v; }
			item.padding = crate::DeploymentConfig::get().emblem[index as usize];
			if GameUserData::get_sequence() == 0 { item.help_text = "Assign an emblem to swap recruitment position.".into(); }
			else { item.help_text = "View the set emblem recruitment order.".into(); }
		}
		else {
			if let Some(v) = PersonData::get(PIDS[index as usize]).map(|v| v.get_name()) { item.title = v; }
			if mode == 0 {
				if index < 32 { item.padding = crate::DeploymentConfig::get().unit1[index as usize]; }
				else { item.padding = crate::DeploymentConfig::get().unit2[index as usize - 32 ]; }
				item.help_text = "Assign an playable unit to swap recruitment positions.".into();
			}
			else {
				if is_required(index) {
					unsafe { CUSTOM_RECRUITMENT_ORDER[index as usize] = index as u8; }
					item.attribute = 2;
					item.padding = index as u8;
					item.help_text = "This character's recruitment position cannot be changed.".into();
				}
				else {
					item.padding = unsafe { CUSTOM_RECRUITMENT_ORDER[index as usize] };
					item.help_text = "Assign an playable unit to swap recruitment positions.".into();
				}
			}
			if GameUserData::get_sequence() != 0 { item.help_text = "View the set unit recruitment order.".into(); }
		}
		Self::set_text(item);
		item
	}
	pub fn on_select(this: &mut DVCCustomRecruitmentMenuItem, _optional_method: OptionalMethod) {
		this.is_arrow = GameUserData::get_sequence() == 0;
		if this.mode == 2 && is_required(this.index) { this.is_arrow = false; }
		Self::update_text(this);
		Self::config_select(this);
	}
	pub fn a_call(_this: &mut DVCCustomRecruitmentMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }

	pub fn build_attribute(_this: &mut DVCCustomRecruitmentMenuItem, _optional_method: OptionalMethod) -> BasicMenuItemAttribute {
		BasicMenuItemAttribute::Enable
	}
	pub fn init_content(_this: &mut DVCCustomRecruitmentMenuItem, _optional_method: OptionalMethod) {}
	pub fn custom_call(this: &mut DVCCustomRecruitmentMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult{
		if GameUserData::get_sequence() != 0 { return BasicMenuResult::new(); }
		let pad = get_instance::<Pad>();
		if (!pad.old_buttons.left() && !pad.old_buttons.right() ) && (pad.npad_state.buttons.right() || pad.npad_state.buttons.left()) {
			let increase = pad.npad_state.buttons.right();
			this.padding =
				match this.mode {
					1 => {crate::DeploymentConfig::get().get_next_emblem(this.index, increase) }
					2 => {
						let old_value = this.padding;
						let max = unsafe { CUSTOM_RECRUITMENT_ORDER[41] };
						let mut avail = vec![];
						for x in 1..max {
							if is_required(x as i32) { continue; }
							if unsafe { !CUSTOM_RECRUITMENT_ORDER.contains(&x) } { avail.push(x); }
						}
						if is_required(this.index) { this.index as u8 }
						else {
							if increase {
								if old_value >= max { avail.first() }
								else { avail.iter().find(|v_new| **v_new > old_value) }
									.map(|v| *v).unwrap_or(max)
							}
							else {
								if old_value < 1 { avail.last() }
								else { avail.iter().rfind(|v_new| **v_new < old_value) }
									.map(|v| *v).unwrap_or(max)
							}
						}
					}
					_ => {crate::DeploymentConfig::get().get_next_unit(this.index, increase) }
				};
			if this.mode == 2 { unsafe { CUSTOM_RECRUITMENT_ORDER[this.index as usize] = this.padding; } }
			Self::update_text(this);
			BasicMenuResult::se_cursor()
		}
		else { BasicMenuResult::new() }
	}
	pub fn plus_call(this: &mut DVCCustomRecruitmentMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
		if GameUserData::get_sequence() != 0 { BasicMenuResult::new() }
		else if this.mode != 2 {
			this.padding = 50;
			DeploymentConfig::get().set_custom_index(this.index, 50, this.mode == 1);
			Self::update_text(this);
			BasicMenuResult::se_cursor()
		}
		else {
			if is_required(this.index) {
				unsafe { CUSTOM_RECRUITMENT_ORDER[this.index as usize] = this.index as u8; }
				this.padding = this.index as u8;
				BasicMenuResult::se_miss()
			}
			else {
				unsafe {
					CUSTOM_RECRUITMENT_ORDER[this.index as usize] = CUSTOM_RECRUITMENT_ORDER[41];
					this.padding = CUSTOM_RECRUITMENT_ORDER[41];
				}
				Self::update_text(this);
				BasicMenuResult::se_cursor()
			}
		}
	}
	pub fn minus_call(this: &mut DVCCustomRecruitmentMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
		let index = this.index as u8;
		let emblem = this.mode == 1;
		if GameUserData::get_sequence() != 0 { BasicMenuResult::new() }
		else if this.mode != 2 {
			this.padding = index;

			DeploymentConfig::get().set_custom_index(this.index, this.padding, emblem);
			this.menu.full_menu_item_list.iter_mut().for_each(|item|{
				if item.index != index as i32 && item.padding == index {
					item.padding = 50;
					DeploymentConfig::get().set_custom_index(item.index, 50, emblem);
					Self::update_text(item);
				}
			});
			Self::update_text(this);
			BasicMenuResult::se_cursor()
		}
		else {
			unsafe {
				CUSTOM_RECRUITMENT_ORDER[this.index as usize] = this.index as u8;
				this.padding = index;
				this.menu.full_menu_item_list.iter_mut().for_each(|item|{
					if item.index != index as i32 && item.padding == index {
						item.padding = CUSTOM_RECRUITMENT_ORDER[41];
						CUSTOM_RECRUITMENT_ORDER[item.index as usize] = CUSTOM_RECRUITMENT_ORDER[41];
						Self::update_text(item);
					}
				})
			}
			Self::update_text(this);
			BasicMenuResult::se_cursor()
		}
	}
	pub fn update_text(this: &mut DVCCustomRecruitmentMenuItem) {
		Self::set_text(this);
		Self::update_config_text(this);
	}
	pub fn set_text(this: &mut DVCCustomRecruitmentMenuItem) {
		let is_emblem = this.mode == 1;
		if GameUserData::get_sequence() != 0 {
			let index = this.index;
			if is_emblem {
				if GameVariableManager::exist(format!("G_R_{}", EMBLEM_GIDS[index as usize])) {
					this.command_text = Mess::get(GodData::get(GameVariableManager::get_string(format!("G_R_{}", EMBLEM_GIDS[index as usize])).to_string()).unwrap().mid);
				}
				else { this.command_text = Mess::get(format!("MPID_{}", RINGS[index as usize])); }
			}
			else {
				if GameVariableManager::exist(format!("G_R_{}", PIDS[index as usize])) {
					this.command_text = Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", PIDS[index as usize])).to_string());
				}
				else { this.command_text = Mess::get_name(MPIDS[index as usize]); }
			}
		}
		else {
			this.command_text =
				if is_emblem {
					let new_index = this.padding as usize;
					if new_index < 19 { Mess::get(format!("MPID_{}", RINGS[new_index])) }
					else { Mess::get("MID_MATCH_Random") }
				}
				else if this.mode == 0 {
					let new_index = this.padding as usize;
					if new_index < 41 { Mess::get(MPIDS[new_index]) }
					else { Mess::get("MID_MATCH_Random") }
				}
				else {
					let playables = &GameData::get().playables;
					if this.padding < playables.len() as u8 {
						GameData::get().playables.get(this.padding as usize)
							.and_then(|p| PersonData::try_get_hash(p.hash))
							.map(|m| Mess::get_name(m.pid))
							.unwrap_or(format!("Person #{}", this.padding).into())
					}
					else { Mess::get("MID_MATCH_Random") }
				}
		}
	}
}

pub fn is_required(person_index: i32) -> bool { person_index < 30 && ((1 << person_index) & 142753809 != 0) }