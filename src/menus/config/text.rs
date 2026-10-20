use std::sync::OnceLock;
use engage::{
    app::{Mess_IconCategory, IJobDataMethods, IStructBase, Language},
    Mess,
};
use crate::{get_chapter_from_idx, VERSION};
use super::*;

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
    pub fn on_off(is_on: bool) -> Il2CppString {
        Mess::get(format!("MID_CONFIG_COMBATANIME_{}", if is_on { "ON" } else { "OFF" }))
    }
    pub fn normal_random(random: bool) -> Il2CppString {
        let mid = if random { "MID_SYS_Grow_Random" } else { "MID_CONFIG_COMBATCAMERA_HORIZON_NOMAL" };
        Mess::get(mid)
    }
    pub fn include_exclude(include: bool) -> Il2CppString {
        let lang = Language::get_lang().value;
        match lang {
            0 => if include { "盛り込む" } else { "除外する" },
            2|5 => if include { "Inclure" } else { "Exclure" },
            3|6 => if include { "Incluir" } else { "Excluir" },
            7 => if include { "Einschließen" } else { "Ausschließen"},
            8 => if include { "Includi" } else { "Escludi" },
            9|10 => if include {"包含" } else { "排除" },
            11=> if include { "포함"} else { "제외"},
            _ => if include { "Include" } else { "Exclude" },
        }.into()
    }
    pub fn change_text(title: &str, to: Il2CppString) -> String {
        let lang = Language::get_lang().value;
        match lang {
            0 => { format!("{} 設定を {}\nに変更しますか?", title, to) }
            2 | 4 => { format!("Modifier le paramètre « {} »\nen « {} » ?", title, to) }
            3 | 6 => { format!("¿Cambiar la configuración\n“{}” a “{}”?", title, to) }
            7 => { format!("Die Einstellung „{}“\nin „{}“ ändern?", title, to) }
            8 => { format!("Cambia l'impostazione '{}'\nin '{}'?", title, to) }
            11=> { format!("'{}' 설정을 '{}'로 변경합니다?", title, to) }
            9 => { format!("将“{}”设置更改为“{}”？", title, to ) }
            10  => { format!("將“{}”設定更改為“{}”？", title, to)}
            _ => { format!("Change '{}'\nto '{}'?", title, to) }
        }
    }
    pub fn none() -> Il2CppString{
        let lang = Language::get_lang().value;
        match lang {
            1 => { "なし" }
            2 | 4 => { "Aucun" }
            3 | 6 => { "Nada" }
            7 => { "Keine" }
            8 => { "Nessuno" }
            11 => { "없음" }
            9  => { "无" }
            10 => { "無 "}
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
        include_str!("../../data/config/en.txt").lines()
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
                let spilt_line_2 = s[1].split('|');
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
    pub fn set_text(&self, item: DVCBasicMenuItem) {
        let command_index = item.value();
        let kind = item.get_menu_item_type();
        match kind {
            DVCMenuItemKind::Job(job) => {
                let command_index = item.value();
                println!("Value: {}", command_index);
                let mut help =
                    if job == DVCJobItem::UnitJob {
                        if DVCVariables::is_main_menu() { "Assign class lines." } else { "View assigned class lines." }
                    }
                    else { "Restrict units to this class line." }.to_string();
                match job {
                    DVCJobItem::SingleClass => { item.set_title_text("Opps all"); }
                    DVCJobItem::GenderM => { item.set_title_text("Opps all [Male]"); }
                    DVCJobItem::GenderF => { item.set_title_text("Opps all [Female]"); }
                    DVCJobItem::UnitJob => { item.set_title_text(PersonPool::get_playable_person_name(item.m_index())); }
                }
                help += format!(" {} {}",  engage::app::Mess::create_sprite_tag(Mess_IconCategory::system(), "Plus"), Self::normal_random(true)).as_str();
                match command_index {
                    -1 => item.set_m_command_text(Self::normal_random(true)),
                    0 => {
                        let command_text = if job != DVCJobItem::UnitJob { "-----" } else { "Default" };
                        item.set_m_command_text(command_text.into());
                    }
                    1000 => { item.set_m_command_text( format!("{} (DLC)", Mess::get("MJID_ShadowLordR")).into()); }
                    _ => {
                        if let Some(job) = utils::to_opt(JobData::try_get_from_hash(command_index)){
                            help += format!(" / {} Description", engage::app::Mess::create_sprite_tag(Mess_IconCategory::system(), "X")).as_str();
                            item.set_m_command_text(format!("{}: {}", job.index(), Mess::get(job.get_name())).into());
                        }
                        else {  item.set_m_command_text( "???".into()); };
                    }
                };
                item.set_m_help_text(help.into());
            }
            #[cfg(feature = "debug")]
            DVCMenuItemKind::Debug(debug) => {
                match debug {
                    DVCDebug::ClearMap => {
                        item.set_m_command_text("Clear".into());
                        item.set_m_title_text("Complete Chapter".into());
                        item.set_m_help_text("Sets the Win (勝利) variable to true.".into());
                    }
                    DVCDebug::AverageLevel => {
                        item.set_m_command_text(item.value().to_string().into());
                        item.set_m_title_text("Set Player Average Level".into());
                        item.set_m_help_text("Sets the average player level for autoscaling tests".into());
                    }
                    DVCDebug::OpenWorldMap => {
                        item.set_m_command_text("Open".into());
                        item.set_m_title_text("Unlock World Map".into());
                        item.set_m_help_text("Open all nodes on the world map.".into());
                    }
                    DVCDebug::ChangeChapterState => {
                        let index = item.value();
                        let name = get_chapter_from_idx(index);
                        let key = GameVariableManager::get_bool(format!("G_Cleared_{}", name));
                        item.set_m_command_text(format!("{} [{}]", name, if key { "C"} else { "U" }).into());
                        item.set_m_help_text("Toggle Chapter state between cleared / uncleared.".into());
                        item.set_m_title_text("Toggle Chapter State".into());
                    }
                }
            }
            DVCMenuItemKind::Command(command) => {
                item.set_m_is_arrow( false);
                item.set_m_is_command_icon( true);
                match command {
                    /*
                    DVCCommand::Credits => {
                        item.set_m_command_text("Debug".into());
                        item.set_m_title_text("Debug Button".into());
                        item.set_m_help_text("For testing purposes only. Behavior is unexpected".into());
                    }
                     */
                    DVCCommand::Menu => {
                        item.set_m_command_text("View".into());
                        item.set_m_help_text("View Draconic Vibe Crystal settings".into());
                        item.set_m_title_text(format!("Draconic Vibe Crystal v{}", VERSION).into());
                    }
                    #[cfg(feature = "debug")]
                    DVCCommand::Output => {
                        item.set_m_command_text("Create".into());
                        item.set_m_title_text("DVC Output File".into());
                        item.set_m_help_text("Save file containing DVC Randomization results.".into());
                    }
                    DVCCommand::SetSeed => {
                        let v = if DVCVariables::is_main_menu() { DVCConfig::get().seed } else { DVCVariable::Seed.get_setting_value() as u32 };
                        if DVCVariables::is_main_menu() || v == 0 {
                            item.set_m_command_text( "Set Seed".into());
                            item.set_m_help_text(format!("Set randomizer seed. {} Random Seed", engage::app::Mess::create_sprite_tag(Mess_IconCategory::system(), "Plus")).into());
                            if v == 0 { item.set_title_text("Seed: Unset"); }
                            else { item.set_title_text(format!("Seed: {}", v)); }
                        }
                        else {
                            item.set_m_command_text( "Change Seed".into());
                            item.set_m_help_text( format!("Change save file seed. {} Random Seed", engage::app::Mess::create_sprite_tag(Mess_IconCategory::system(), "Plus")).into());
                            if v == 0 { item.set_title_text("Save File Seed: Unset"); }
                            else { item.set_title_text(format!("Save File Seed: {}", v)); }
                        }
                    }
                    DVCCommand::ReRandJob => {
                        if !DVCVariables::is_main_menu() {
                            item.set_m_command_text( "Randomize".into());
                            if DVCVariables::get_chapter_index() < 4 { item.set_m_help_text("Re-Randomize player unit classes".into()); }
                            else { item.set_m_help_text("Re-Randomize unrecruited player unit classes".into()); }
                            item.set_title_text("Re-Roll Player Unit Class");
                        }
                    }
                }
            }
            DVCMenuItemKind::Menu(menu) => {
                let index = menu as usize;
                if let Some(m) = self.menu.get(index) {
                    item.set_m_is_command_icon( true);
                    item.set_m_is_arrow(false);
                    item.set_title_text(m.title);
                    if let Some(com_text) = m.command {  item.set_m_command_text( com_text.into()); }
                    else { item.set_m_command_text( "View".into()); }
                    item.set_m_help_text(m.help.into());
                }
            }
            DVCMenuItemKind::Variable(var) => {
                let index = var as usize;
                if let Some(m) = self.var.get(index) {
                    item.set_title_text(m.title);
                    if var == DVCVariable::EnemyCapabilityOffset { item.set_m_command_text( command_index.to_string().into()) }
                    else {
                        if command_index == 3 && m.commands.len() == 3 { item.set_m_command_text( format!("{}/{}", m.commands[1], m.commands[2]).into()); }
                        else if let Some(command) = m.commands.get(command_index as usize).or_else(|| m.commands.last()){ item.set_m_command_text(  command.as_str().into()); }
                        else { item.set_m_command_text( command_index.to_string().into()); }
                    }
                    if let Some(help) = m.help.get(command_index as usize).or_else(|| m.help.last()){
                        item.set_m_help_text(help.as_str().into());
                    }
                    else { item.set_m_help_text("Nothing is here.".into()); }
                }
            }
            DVCMenuItemKind::Gauge(var) => {
                let index = var as usize;
                let s_index = if index > 35 { 35 } else { index };
                let v = item.value();
                if let Some(m) = self.var.get(s_index){
                    let help_index =
                        if m.help.len() > 2 { if v == 10 { 1 } else if v > 10 { 2 } else { 0 } }
                        else if v == 0 { 0 }
                        else { 1 };

                    if let Some(help) = m.help.get(help_index) {
                        if help.contains("$$"){ item.set_m_help_text(help.replace("$$", v.to_string().as_str()).into()); }
                        else { item.set_m_help_text(help.as_str().into()); }
                    }
                    else { item.set_m_help_text("Nothing is here.".into()); }
                    if m.title.contains("$R") && index >= 35 {
                        let r = RANK.get(index - 35).unwrap_or(&"?");
                        item.set_title_text(m.title.replace("$R", r));
                    }
                    else { item.set_title_text(m.title) }
                    item.set_m_gauge_ratio(v as f32 / 100.0);
                }
            }
            DVCMenuItemKind::Flag(flag) => {
                let index = flag as usize;
                if let Some(m) = self.flag.get(index) {
                    item.set_title_text(m.title);
                    let v = command_index != 0;
                    let com = m.flag_command / 2;
                    let reverse = m.flag_command & 1 == 1;
                    let xor = v ^ reverse;
                    item.set_m_command_text( 
                        match com {
                            -1 => { "------".into() }
                            1 => { Self::normal_random(xor) }
                            2 => { if xor { "Enable" } else { "Disable" }.into() }
                            3 => { Self::include_exclude(xor) }
                            _ => { Self::on_off(xor) }
                        }
                    );
                    item.set_m_help_text(m.help.into());
                }
            }
            DVCMenuItemKind::Order(order) => { order.set_text(item); }
        }
    }
    pub fn apply_menu_header(kind: DVCMenu) {
        let text = CONFIG_TEXT.get_or_init(||Self::init());
        if let Some(data) = text.menu.get(kind as usize){
            if let Some(header) = data.header {
                if let Some(help) = data.header_help { engage::TitleBar::open_header(header, help, ""); }
                else { engage::TitleBar::open_header(header, Mess::get("MID_MENU_CONFIG_HELP").to_string(), ""); }
                return;
            }
        }
        let mut version_str = String::new();
        version_str += VERSION;
        #[cfg(feature = "debug")]{ version_str += " Debug"; }
        engage::TitleBar::open_header(draconic_vibe_name(), VERSION, "");
        /*
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
         */
    }
}
pub fn draconic_vibe_name() -> &'static str {
    let lang = Language::get_lang().value;
    match lang {
        0 => "竜の振動水晶",
        2|4  => "Cristal de Vibration Draconique",
        3|5 => "Gema de Vibración Dracónica",
        7 => "Drachenvibrationskristall",
        8 => "Gemma di Vibrazione del Drago",
        9 => "龍之氣息水晶",
        10 => "龙之气息水晶",
        11 => "용의 분위기석",
        _ => "Draconic Vibe Crystal",
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
