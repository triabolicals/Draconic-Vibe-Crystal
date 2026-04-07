use std::collections::HashMap;
use std::fmt::Display;
use engage::mess::Mess;
use unity::prelude::Il2CppString;
use crate::language::Localization;
pub const KEY: [&str; 14] = ["A", "B", "X", "Y", "L", "R", "ZL", "ZR", "Plus", "Plus", "Up", "Down", "Right", "Left"];
#[repr(i32)]
pub enum DVCCommandText {
	On = 0,
	Off = 1,
	Normal = 2,
	Random = 3,
	Personal = 4,
	Class = 5,
	Sync = 6,
	Engage = 7,
	List = 8,
	Emblems = 9,
	View = 10,
	SelfText = 11,
	SP = 12,
	Appearance = 13,
	Outfits = 14,
	ALL = 15,
	EmblemEnergy = 16,
	Fixed = 17,
	Skill = 18,
	GrowMode = 19,
	Emblem = 20,
	Body = 21,
	Voice = 22,
	Somniel = 23,
	Battle = 24,
	Cancel = 25,
	Confirm = 26,
	Load = 27,
	Save = 28,
	Mode = 29,
	Settings = 30,
	Select = 31,
	Quit = 32,
	A = 200,
	B = 201,
	X = 202,
	Y = 203,
	L = 204,
	R = 205,
	ZL = 206,
	ZR = 207,
	Plus = 208,
	Minus = 209,
	Up = 210,
	Down = 211,
	Left = 212,
	Right = 213,
	LeftRight = 214,
}
pub const MIDS: [&str; 33] = [
	"MID_CONFIG_ROD_DANCE_OFF", "MID_CONFIG_ROD_DANCE_ON", "MID_CONFIG_GAMESPEED_NOMAL", "MID_MATCH_Random",
	"MID_SORTIE_SKILL_CATEGORY_PERSON", "MID_SYS_Class", "MID_SORTIE_SKILL_CATEGORY_GOD", "MID_SORTIE_SKILL_CATEGORY_ENGAGE",
	"MID_MENU_UNIT_LIST", "MID_MENU_REFINE_SHOP_REFINE_GODSYMBOL", "MID_KEYHELP_MENU_UNIT_SELECT", "MID_SYS_Me", "MID_SYS_SP",
	"MID_GAMESTART_PLAYER_GENDER_SELECT_TITLE", "MID_Hub_amiibo_Accessory_Trade", "MID_ITEMMENU_ITEM_ALL",  "MTID_Engage", "MID_SYS_Grow_Fixed",
	"MID_SYS_Skill", "MID_GAMESTART_GROWMODE_SELECT_TITLE", "MID_H_INFO_Param_Correction_God", "MID_MENU_ACCESSORY_SHOP_PART_BODY", "MID_MAINMENU_LANGUAGE_VOICE",
	"MID_SAVEDATA_SEQ_HUB", "MID_TUT_CATEGORY_TITLE_Battle", "MID_MATCH_Map_UpRoad_No", "MID_Decision", "MID_SAVEDATA_LOAD_YES", "MID_SAVEDATA_SAVE_TITLE",
	"MID_ProfileCard_Card_GameMode", "MID_MENU_CONFIG", "MID_KEYHELP_MENU_SELECT", "MID_MENU_RESET",
];

impl DVCCommandText {
	pub fn count() -> usize { MIDS.len() }
	pub fn get_with_sys_sprite(self, sys: &str) -> &'static Il2CppString {
		format!("{}{}", Mess::create_sprite_tag_str(2, sys), self.get()).into()
	}
	pub fn get_with_value<T: Display>(self, value: T) -> &'static Il2CppString {
		format!("{}: {}", self.get(), value).into()
	}
	pub fn get(self) -> &'static Il2CppString {
		let index = self as usize;
		if index < MIDS.len() { Mess::get(MIDS[index]) }
		else if index >= 200 && index < 214 {
			Mess::create_sprite_tag_str(2, KEY[index-200])
		}
		else if index  == 214 {
				format!("{}{}", Mess::create_sprite_tag_str(2, "Left"), Mess::create_sprite_tag_str(2, "Right")).into() 
		}
		else { "".into() }
	}
	pub fn get_from_index(index: i32) -> &'static mut Il2CppString {
		if index < MIDS.len() as i32 { Mess::get(MIDS[index as usize]) }
		else { format!("C {}", index).into() }
	}
}
pub struct DVCLocalizerInner {
	pub command: HashMap<i32, String>,
	pub config: HashMap<i32, String>,
	pub outfit: HashMap<i32, String>,
	pub current_language: Localization,
}

impl DVCLocalizerInner {
	pub fn init() -> DVCLocalizerInner {
		let current_language = Localization::get();
		let mut title = HashMap::new();
		let mut command = HashMap::new();
		let mut outfit = HashMap::new();
		Self::set_en_text(&mut title, &mut command, &mut outfit);
		Self::set_lang(current_language, &mut title, &mut command);
		Self { current_language, config: title, command, outfit}
	}
	pub fn set_en_text(
		title: &mut HashMap<i32, String>,
		command: &mut HashMap<i32, String>,
		outfit: &mut HashMap<i32, String>
	) {

		title.clear();
		command.clear();
		*outfit = include_str!("../outfit/en.txt").lines().flat_map(|l| {
			if l.contains("\t") {
				l.split_once("\t").map(|x| (x.0.parse::<i32>().unwrap(), x.1.to_string()))
			} else if let Some(spilt) = l.split_once(" ") {
				let mut s = spilt.1.to_string();
				if s.contains("\\n") { Some((spilt.0.parse::<i32>().unwrap(), s.replace("\\n", "\n"))) }
				else { Some((spilt.0.parse::<i32>().unwrap(), s)) }
			} else { None }
		}).collect();
		*command =
			include_str!("../command/en.txt").lines().flat_map(|l| {
				if l.contains("\t") { l.split_once("\t").map(|x| (x.0.parse::<i32>().unwrap(), x.1.to_string())) } else if let Some(spilt) = l.split_once(" ") {
					Some((spilt.0.parse::<i32>().unwrap(), spilt.1.to_string()))
				} else { None }
			}).collect()
	}
	pub fn set_lang(lang: Localization, title: &mut HashMap<i32, String>, command: &mut HashMap<i32, String>) {
		let lang = lang.get_lang_code();
		match lang {
			"ja" => {
				//parse_and_replace_from_file(title, include_str!("../config/ja.txt"));
				parse_and_replace_from_file(command, include_str!("../command/ja.txt"));
			}
			"fr" => {
				//parse_and_replace_from_file(title, include_str!("../config/fr.txt"));
				parse_and_replace_from_file(command, include_str!("../command/fr.txt"));
			}
			"es" => {
				//parse_and_replace_from_file(title, include_str!("../config/es.txt"));
				parse_and_replace_from_file(command, include_str!("../command/es.txt"));
			}
			"de" => {
				//parse_and_replace_from_file(title, include_str!("../config/de.txt"));
				parse_and_replace_from_file(command, include_str!("../command/de.txt"));
			}
			"it" => {
				//parse_and_replace_from_file(title, include_str!("../config/it.txt"));
				parse_and_replace_from_file(command, include_str!("../command/it.txt"));
			}
			"tw" => {
				//parse_and_replace_from_file(title, include_str!("../config/tw.txt"));
				parse_and_replace_from_file(command, include_str!("../command/tw.txt"));
			}
			"cn" => {
				//parse_and_replace_from_file(title, include_str!("../config/cn.txt"));
				parse_and_replace_from_file(command, include_str!("../command/cn.txt"));
			}
			"kr" => {
				// parse_and_replace_from_file(title, include_str!("../config/kr.txt"));
				parse_and_replace_from_file(command, include_str!("../command/kr.txt"));
			}
			_ => {}
		}
	}
	pub fn update_language(&mut self) {
		let new_language = Localization::get();
		if new_language != self.current_language {
			if self.current_language != Localization::USEng && self.current_language != Localization::EUEnglish {
				Self::set_en_text(&mut self.config, &mut self.command, &mut self.outfit);
			}
			self.current_language = new_language;
			Self::set_lang(new_language, &mut self.config, &mut self.command);
		}
	}
}
fn parse_and_replace_from_file(map: &mut HashMap<i32, String>, file: &str) {
	file.lines().flat_map(|l| {
		if l.contains("\t") { l.split_once("\t").map(|x| (x.0.parse::<i32>().unwrap(), x.1.to_string())) } else if let Some(spilt) = l.split_once(" ") {
			Some((spilt.0.parse::<i32>().unwrap(), spilt.1.to_string()))
		} else { None }
	}).for_each(|(index, title)| {
		if let Some(value) = map.get_mut(&index) { *value = title; }
		else { map.insert(index, title); }
	});
}