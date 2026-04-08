use std::sync::OnceLock;
use super::*;
use engage::gamedata::{Gamedata, GodData, JobData, PersonData};
use engage::gamevariable::GameVariableManager;
use engage::language::*;
use engage::mess::Mess;
use crate::{DVCVariables, VERSION};
use crate::enums::PIDS;
use crate::menus::ingame::draconic_vibe_name;
use crate::randomizer::data::GameData;
use crate::randomizer::{EMBLEM_GIDS, MPIDS, RINGS};

pub static CONFIG_TEXT: OnceLock<DVCConfigText> = OnceLock::new();
pub static NONE: &'static str = "---";
pub const RANK: [&'static str; 4] = ["S", "A", "B", "C"];
pub struct DVCConfigText {
    pub var: Vec<DVCVariableText>,
    pub flag: Vec<DVCFlagText>,
    pub menu: Vec<DVCMenuText>,
    pub com: Vec<DVCCommandText>,
}
impl DVCConfigText {
    pub fn on_off(is_on: bool) -> &'static mut Il2CppString {
        Mess::get(format!("MID_CONFIG_COMBATANIME_{}", if is_on { "ON" } else { "OFF" }))
    }
    pub fn normal_random(random: bool) -> &'static mut Il2CppString {
        let mid = if random { "MID_SYS_Grow_Random" } else { "MID_CONFIG_COMBATCAMERA_HORIZON_NOMAL" };
        Mess::get(mid)
    }
    pub fn include_exclude(include: bool) -> &'static mut Il2CppString {
        match Language::get_lang() {
            LanguageLangs::JPJapanese => if include { "盛り込む" } else { "除外する" },
            LanguageLangs::EUFrench | LanguageLangs::USFrench => if include { "Inclure" } else { "Exclure" },
            LanguageLangs::USSpanish | LanguageLangs::EUSpanish=> if include { "Incluir" } else { "Excluir" },
            LanguageLangs::EUGerman => if include { "Einschließen" } else { "Ausschließen"},
            LanguageLangs::EUItalian => if include { "Includi" } else { "Escludi" },
            LanguageLangs::CNTraditional | LanguageLangs::CNSimplified => if include {"包含" } else { "排除" },
            LanguageLangs::KRKorean => if include { "포함"} else { "제외"},
            _ => if include { "Include" } else { "Exclude" },
        }.into()
    }
    pub fn change_text(title: &str, to: &Il2CppString) -> String {
        match Language::get_lang() {
            LanguageLangs::JPJapanese=> { format!("{} 設定を {}\nに変更しますか?", title, to) }
            LanguageLangs::EUFrench | LanguageLangs::USFrench => { format!("Modifier le paramètre « {} »\nen « {} » ?", title, to) }
            LanguageLangs::USSpanish | LanguageLangs::EUSpanish => { format!("¿Cambiar la configuración\n“{}” a “{}”?", title, to) }
            LanguageLangs::EUGerman => { format!("Die Einstellung „{}“\nin „{}“ ändern?", title, to) }
            LanguageLangs::EUItalian => { format!("Cambia l'impostazione '{}'\nin '{}'?", title, to) }
            LanguageLangs::KRKorean => { format!("'{}' 설정을 '{}'로 변경합니다?", title, to) }
            LanguageLangs::CNSimplified => { format!("将“{}”设置更改为“{}”？", title, to ) }
            LanguageLangs::CNTraditional  => { format!("將“{}”設定更改為“{}”？", title, to)}
            _ => { format!("Change '{}'\nto '{}'?", title, to) }
        }
    }
    pub fn none() -> &'static mut Il2CppString{
        match Language::get_lang() {
            LanguageLangs::JPJapanese=> { "なし" }
            LanguageLangs::EUFrench | LanguageLangs::USFrench  => { "Aucun" }
            LanguageLangs::USSpanish | LanguageLangs::EUSpanish => { "Nada" }
            LanguageLangs::EUGerman => { "Keine" }
            LanguageLangs::EUItalian => { "Nessuno" }
            LanguageLangs::KRKorean => { "없음" }
            LanguageLangs::CNSimplified  => { "无" }
            LanguageLangs::CNTraditional => { "無 "}
            _ => { "None" }
        }.into()
    }
    pub fn init() -> Self {
        let mut var: Vec<DVCVariableText> = Vec::new();
        let mut flag: Vec<DVCFlagText> = Vec::new();
        let mut menu: Vec<DVCMenuText> = Vec::new();
        let mut com: Vec<DVCCommandText> = Vec::new();
        let mut section = 0;
        let mut var_str = vec![];
        include_str!("../../../data/config/en.txt").lines()
            .for_each(|line| {
                if line == "[Command]" { section = 0; }
                else if line == "[SubMenu]" { section = 1; }
                else if line == "[Variables]" { section = 2; }
                else if line == "[Flags]" { section = 3; }
                else {
                    if section == 2 { var_str.push(line); } else if line.contains("|") {
                        let mut spilt = line.split('|');
                        if let Some(title) = spilt.next() {
                            match section {
                                0 => {
                                    if let Some(command) = spilt.next() {
                                        if let Some(help) = spilt.next() {
                                            let help_alt = spilt.next();
                                            let command = DVCCommandText { title, command, help, help_alt };
                                            com.push(command);
                                        }
                                    }
                                }
                                1 => {
                                    if let Some(help) = spilt.next() {
                                        let command = spilt.next().filter(|s| s.len() > 1);
                                        let header = spilt.next().filter(|s| s.len() > 1);
                                        let header_help = spilt.next().filter(|s| s.len() > 1);
                                        let m = DVCMenuText { title, help, header, command, header_help };
                                        menu.push(m);
                                    }
                                }
                                3 => {
                                    if let Some(help) = spilt.next() {
                                        let flag_command =
                                            if let Some(b) = spilt.next() {
                                                if b == "fo" { 1 }
                                                else if b == "rn" { 2 }
                                                else if b == "nr" { 3 }
                                                else if b == "ed" { 4 }
                                                else if b == "de" { 5 }
                                                else if b == "ie" { 6 }
                                                else if b == "ei" { 7 }
                                                else { 0 }
                                            } else { 0 };
                                        flag.push(DVCFlagText { title, help, flag_command });
                                    } else {
                                        let m = DVCFlagText { title, help: NONE, flag_command: -1 };
                                        flag.push(m);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    else {
                        match section {
                            2 => { var_str.push(line); }
                            3 => {
                                let m = DVCFlagText { title: line, help: NONE, flag_command: -1 };
                                flag.push(m);
                            }
                            _ => {}
                        }
                    }
                }
            });
        var_str.windows(2).enumerate().for_each(|(i, s)|{
            if i % 2 == 0 {
                let mut spilt_line_1 = s[0].split('|');
                let mut spilt_line_2 = s[1].split('|');
                if let Some(title) = spilt_line_1.next() {
                    if title.starts_with("Var") {
                        var.push(DVCVariableText{title, commands: vec![], help: vec![]});
                    }
                    else {
                        let mut commands: Vec<String> = vec![];
                        let help: Vec<String> = spilt_line_2.map(|v| v.to_string()).collect();
                        while let Some(s) = spilt_line_1.next() {
                            if s == "r" { commands.push("Random".to_string()); }
                            else if s == "d" { commands.push("Default".to_string()); }
                            else if s == "o" { commands.push("Off".to_string()); }
                            else if s == "O" { commands.push("On".to_string()); }
                            else if s == "u" { commands.push("Unit".to_string()); }
                            else if s == "rv" { commands.push("Reverse".to_string()); }
                            else if s == "s" { commands.push("Shuffled".to_string()); }
                            else if s == "fr" { commands.push("Full Random".to_string()); }
                            else if s == "n" { commands.push("None".to_string()); }
                            else if s == "c" { commands.push("Chaos".to_string()); }
                            else { commands.push(s.to_string()); }
                        }
                        var.push(DVCVariableText{title, commands, help});
                    }
                }
            }
        });
        Self { var, flag, menu, com, }
    }
    pub fn set_text(&self, item: &mut DVCConfigMenuItem) {
        let command_index = item.dvc_value;
        let kind = item.menu_item_kind.clone();
        match kind {
            DVCMenuItemKind::SingleJob => {
                item.title = "Opps all".into();
                item.help_text = "Playable units will be restricted to this class line".into();
                match item.dvc_value {
                    1 => { item.command_text = format!("{} (DLC)", Mess::get("MJID_ShadowLordR")).into(); }
                    _ => {
                        if let Some(job) = JobData::try_get_hash(item.dvc_value) {
                            item.command_text = format!("{}: {}", job.parent.index, Mess::get_name(job.jid)).into()
                        }
                        else { item.command_text = "???".into(); }
                    }
                };
                if !DVCVariables::is_main_menu() {
                    if !DVCFlags::SingleJobEnabled.get_value() || DVCVariables::SingleJob.get_value() != item.dvc_value {
                        item.is_command = true;
                    }
                }
            }
            DVCMenuItemKind::Command(command) => {
                item.is_arrow = false;
                item.is_command = true;
                match command {
                    DVCCommand::SetSeed => {
                        let v = DVCVariables::Seed.get_value() as u32;
                        if DVCVariables::is_main_menu() || v == 0 {
                            item.command_text = "Input".into();
                            item.help_text = format!("Set randomizer seed. {} Random Seed", Mess::create_sprite_tag_str(2, "Plus")).into();
                            if v == 0 { item.title = "Seed: Unset".into(); }
                            else { item.title = format!("Seed: {}", v).into(); }
                        }
                        else {
                            item.command_text = "Change".into();
                            item.help_text = format!("Change save file seed. {} Random Seed", Mess::create_sprite_tag_str(2, "Plus")).into();
                            if v == 0 { item.title = "Save File Seed: Unset".into(); }
                            else { item.title = format!("Save File Seed: {}", v).into(); }
                        }
                    }
                    DVCCommand::ReRandJob => {
                        item.command_text = "Randomize".into();
                        item.help_text = "Re-Randomize unit classes".into();
                        item.title = "Re-Roll Player Unit Class".into();
                    }
                }
            }
            DVCMenuItemKind::Menu(menu) => {
                let index = menu as usize;
                if let Some(m) = self.menu.get(index) {
                    item.is_command = true;
                    item.is_arrow = false;
                    item.title = m.title.into();
                    if let Some(com_text) = m.command { item.command_text = com_text.into(); }
                    else { item.command_text = "View".into(); }
                    item.help_text = m.help.into();
                }
            }
            DVCMenuItemKind::Variable(var) => {
                let index = var as usize;
                if let Some(m) = self.var.get(index) {
                    item.title = m.title.into();
                    if command_index == 3 && m.commands.len() == 3 {
                        item.command_text = format!("{}/{}", m.commands[1], m.commands[2]).into();
                    }
                    else if let Some(command) = m.commands.get(command_index as usize).or_else(|| m.commands.last()){
                        item.command_text = command.as_str().into();
                    }
                    else { item.command_text = "---".into(); }
                    if let Some(help) = m.help.get(command_index as usize).or_else(|| m.help.last()){
                        item.help_text = help.as_str().into();
                    }
                    else { item.help_text = "Nothing is here.".into(); }
                }
            }
            DVCMenuItemKind::Gauge(var) => {
                let mut index = var as usize;
                let s_index = if index > 35 { 35 } else { index };
                if let Some(m) = self.var.get(s_index){
                    let help_index =
                        if m.help.len() > 2 {
                            if command_index == 10 { 1 }
                            else if command_index > 10 { 2 }
                            else { 0 }
                        }
                        else if command_index == 0 { 0 }
                        else { 1 };

                    if let Some(help) = m.help.get(help_index) {
                        if help.contains("$$"){
                            item.help_text = help.replace("$$", command_index.to_string().as_str()).into();
                        }
                        else { item.help_text = help.as_str().into(); }

                    }
                    else { item.help_text = "Nothing is here.".into(); }
                    if m.title.contains("$R") && index >= 35 {
                        let r = RANK.get(index - 35).unwrap_or(&"?");
                        item.title = m.title.replace("$R", r).into();
                    }
                    else { item.title = m.title.into(); }
                    item.gauge_ratio = command_index as f32 / 100.0;
                }
            }
            DVCMenuItemKind::Flag(flag) => {
                let index = flag as usize;
                if let Some(m) = self.flag.get(index) {
                    item.title = m.title.into();
                    let v = command_index != 0;
                    let com = m.flag_command / 2;
                    let reverse = m.flag_command & 1 == 1;
                    let xor = v ^ reverse;
                    item.command_text =
                    match com {
                        -1 => { Mess::get_item_none() }
                        1 => { Self::normal_random(xor) }
                        2 => { if xor { "Enable" } else { "Disable" }.into() }
                        3 => { Self::include_exclude(xor) }
                        _ => { Self::on_off(xor) }
                    };
                    item.help_text = m.help.into();
                }
            }
            DVCMenuItemKind::Order(order) => {
                let index = item.index as usize;
                if order != RecruitmentOrder::Emblem { item.title = Mess::get_name(MPIDS[index]); }
                else { item.title = Mess::get(format!("MPID_{}", RINGS[index])); }
                if !DVCVariables::is_main_menu() {
                    item.command_text =
                        match order {
                        RecruitmentOrder::Unit|RecruitmentOrder::UnitCustom => {
                            let key = format!("G_R_{}", PIDS[index]);
                            if GameVariableManager::exist(key.as_str()){
                                Mess::get_name(GameVariableManager::get_string(key.as_str())) }
                            else {Mess::get_name(MPIDS[index]) }
                        }
                        RecruitmentOrder::Emblem => {
                            let key = format!("G_R_{}", EMBLEM_GIDS[index]);
                            if GameVariableManager::exist(key.as_str()) {
                                Mess::get(GodData::get(GameVariableManager::get_string(key.as_str()).to_string()).unwrap().mid)
                            }
                            else { Mess::get(format!("MPID_{}", RINGS[index])) }
                        }
                    }
                }
                else {
                    match order {
                        RecruitmentOrder::Unit => {
                            if DVCVariables::is_main_menu() {
                                let new_index = item.padding as usize;
                                item.command_text =
                                    if new_index < 41 { Mess::get(MPIDS[new_index]) }
                                    else { Mess::get("MID_MATCH_Random") };
                            }
                        }
                        RecruitmentOrder::UnitCustom => {
                            let playables = &GameData::get().playables;
                            if is_required(index as i32) { item.is_arrow = false; }
                            item.command_text =
                                if item.padding < playables.len() as u8 {
                                    GameData::get().playables.get(item.padding as usize)
                                        .and_then(|p| PersonData::try_get_hash(p.hash))
                                        .map(|m| Mess::get_name(m.pid))
                                        .unwrap_or(format!("Person #{}", item.padding).into())
                                }
                                else { Mess::get("MID_MATCH_Random") };
                        }
                        RecruitmentOrder::Emblem => {
                            let new_index = item.padding as usize;
                            item.command_text =
                                if new_index < 19 { Mess::get(format!("MPID_{}", RINGS[new_index])) }
                                else { Mess::get("MID_MATCH_Random") }
                        }
                    }
                }
            }
        }
    }
    pub fn apply_menu_header(kind: DVCMenu) {
        let text = CONFIG_TEXT.get_or_init(||Self::init());
        if let Some(data) = text.menu.get(kind as usize){
            if let Some(header) = data.header {
                if let Some(help) = data.header_help {
                    TitleBar::open_header(header, help, "");
                }
                else {
                    TitleBar::open_header(header, Mess::get("MID_MENU_CONFIG_HELP").to_string(), "");
                }
            }
            else { TitleBar::open_header(draconic_vibe_name(), VERSION, ""); }
        }
        else { TitleBar::open_header(draconic_vibe_name(), VERSION, ""); }
        if let Some(key_help) = TitleBar::get_instance().current_title.as_ref().map(|v| v.key_help) {
            key_help.help_object[10].set_active(false);
            key_help.help_object[11].set_active(false);
        }
        match kind {
            DVCMenu::CustomUnitOrder2|DVCMenu::CustomUnitOrder|DVCMenu::CustomEmblemOrder => {
                if let Some(key_help) = TitleBar::get_instance().current_title.as_ref().map(|v| v.key_help) {
                    key_help.help_object[10].set_active(true);
                    key_help.help_object[11].set_active(true);
                    key_help.set_text(&key_help.help_object[10], "Randomize".into());
                    key_help.set_text(&key_help.help_object[11], "Original".into());
                }
            }
            _ => {}
        }
    }
}
pub struct DVCMenuText {
    pub title: &'static str,
    pub help: &'static str,
    pub command: Option<&'static str>,
    pub header: Option<&'static str>,
    pub header_help: Option<&'static str>,
}
pub struct DVCVariableText {
    pub title: &'static str,
    pub commands: Vec<String>,
    pub help: Vec<String>,
}

pub struct DVCCommandText {
    pub title: &'static str,
    pub command: &'static str,
    pub help: &'static str,
    pub help_alt: Option<&'static str>,
}

pub struct DVCFlagText {
    pub title: &'static str,
    pub help: &'static str,
    pub flag_command: i32,
}
