use crate::message::original::MessageList;
use crate::message::swap_kinds::MessSwapType;

pub struct TalkSwapData {
    pub filename: String,
    pub g_person_swap: Vec<PersonTalkSwapData>,
    pub demo_list: Vec<TalkDemo>,
}

impl TalkSwapData {
    pub fn new() -> Self {
        Self {
            filename: String::new(),
            g_person_swap: vec![],
            demo_list: vec![],
        }
    }
    pub fn clear(&mut self) {
        self.g_person_swap.clear();
        self.demo_list.clear();
        self.filename = "".to_string();
    }
    pub fn try_get_demo(&self, mid: &String) -> Option<&TalkDemo> {
        let trimed = mid.trim_start_matches("MID_");
        let (demo_name, _) = trimed.split_once("_#")?;
        self.demo_list.iter().find(|d| d.demo == demo_name)
    }
    pub fn try_get_line(&self, mid: &String) -> Option<&TalkLine> {
        let trimed = mid.trim_start_matches("MID_");
        let (demo_name, talk_name) = trimed.split_once("_#")?;
        self.demo_list.iter().find(|d| d.demo == demo_name)
            .and_then(|d| d.lines.iter().find(|l| l.mid == talk_name || l.mid.contains(talk_name)))

    }
    pub fn load(&mut self, filename: &String) {
        self.clear();
        let mut found_section = false;
        let mut talk_demo = TalkDemo {
            demo: String::new(),
            lines: vec![],
            demo_person_swaps: vec![]
        };
        include_str!("../../data/talk_swap").lines().filter(|x| !x.is_empty()).for_each(|line| {
            if line.contains("Section") { found_section = line.contains(filename); } 
            else if found_section {
                if line.starts_with("emblem") || line.starts_with("char") {
                    if let Some(data) = PersonTalkSwapData::from_line(line) {
                        if talk_demo.demo.is_empty() { self.g_person_swap.push(data); } 
                        else { talk_demo.demo_person_swaps.push(data); }
                    }
                } 
                else {
                    let num = line.split_whitespace().count();
                    if num == 1 {
                        if !talk_demo.lines.is_empty() {
                            self.demo_list.push(talk_demo.clone());
                            talk_demo.lines.clear();
                        }
                        talk_demo.demo = line.to_string();
                        talk_demo.demo_person_swaps.clear();
                    } else if talk_demo.demo.len() > 0 {
                        if let Some(data) = TalkLine::new(line) {
                            talk_demo.lines.push(data);
                        } 
                        else { println!("Cannot Parse: {}", line); }
                    }
                }
            }
        });
    }
}

#[derive(Clone)]
pub struct PersonTalkSwapData {
    pub person_index: i32,
    pub emblem: bool,
    pub gender_swap_idx: i32,
    pub gender: i32,
    pub mid: Vec<String>,
    pub exclude_mid: Vec<String>,
}
impl PersonTalkSwapData {
    pub fn from_line(line: &str) -> Option<Self> {
        if line.is_empty() || (!line.starts_with("char") && !line.starts_with("emblem")) { return None; }
        let mut spilt = line.split_whitespace();
        let emblem = spilt.next()?.contains("emblem");   // char
        let person_index = spilt.next().and_then(|s| s.parse::<i32>().ok())?;
        let gender_swap_idx = spilt.next().and_then(|s| s.parse::<i32>().ok())?;
        let gender = spilt.next().and_then(|s| s.parse::<i32>().ok())?;
        let mut mid = Vec::<String>::new();
        let mut exclude_mid = Vec::<String>::new();
        while let Some(mid_) = spilt.next() {
            if mid_.starts_with("-") { exclude_mid.push(mid_.trim_start_matches("-").to_string()); }
            else { mid.push(mid_.to_string()); }
        }
        Some(Self { person_index, emblem, gender_swap_idx, mid, exclude_mid, gender })
    }
    pub fn check_label(&self, label: &String) -> bool {
        !self.exclude_mid.iter().any(|exclude| label.contains(exclude))
            && ( self.mid.is_empty() || self.mid.iter().any(|mid| mid.contains(label)) )
    }
}
#[derive(Clone)]
pub struct TalkDemo {
    pub demo: String,
    pub lines: Vec<TalkLine>,
    pub demo_person_swaps: Vec<PersonTalkSwapData>,
}
impl TalkDemo {
    pub fn new(demo_name: &str) -> Self {
        Self {
            demo: demo_name.to_string(),
            demo_person_swaps: vec![],
            lines: vec![],
        }
    }
}

#[derive(Clone)]
pub struct TalkLine {
    pub mid: String,
    pub commands: Vec<MessSwapType>,
}

impl TalkLine {
    pub fn new(line: &str) -> Option<Self> {
        let mut spilt = line.split_whitespace();
        let mut commands = vec![];
        let mid = spilt.next()?.to_string();
        while let Some(swap) = MessSwapType::from_iter(&mut spilt){ commands.push(swap); }
        if commands.len() == 0 { None }
        else { Some(Self { mid, commands }) }
    }
    pub fn execute(&self, message: &mut Vec<u16>, data: &MessageList) -> bool {
        let mut changed = false;
        self.commands.iter().for_each(|x| {
            match x {
                MessSwapType::HeroAlias(_) => {}
                MessSwapType::HeroJob => {}
                MessSwapType::UnitName(idx) => {
                    let idx = (*idx) as usize;
                    if idx < data.person_list.len() {
                        if let Some((pos, len, _)) = data.person_list.get(idx).and_then(|m| m.find_position(&message, false)) {
                            message.splice(pos..pos + len, [14, 6, 100+idx as u16, 0]);
                            changed = true;
                        }
                    }
                    else {
                        data.person_list.iter().enumerate().for_each(|(i, x)|{ 
                            if let Some((pos, len, _)) = x.find_position(&message, false){ 
                                message.splice(pos..pos+len, [14, 6, 100+i as u16, 0]);
                                changed = true;
                            } 
                        }); 
                    }
                }
                MessSwapType::EmblemName(idx) => {
                    let idx = (*idx) as usize;
                    if idx < data.emblem_list.len() {
                        if let Some((pos, len, _)) = data.emblem_list.get(idx).and_then(|m| m.find_position(&message, false)) {
                            message.splice(pos..pos+len, [14, 6, 200+idx as u16, 0]);
                            changed = true;
                        }
                    }
                    else {
                        data.emblem_list.iter().enumerate().for_each(|(i, x)|{
                            if let Some((pos, len, _)) = x.find_position(&message, false){
                                message.splice(pos..pos+len, [14, 6, 200+i as u16, 0]);
                                changed = true;
                            }
                        })
                    }
                }
                MessSwapType::RingName(idx) => {
                    let idx = (*idx) as usize;
                    if idx < 20 {
                        if let Some((pos, len, _)) = data.emblem_alias.get(idx).and_then(|m| m.find_position(&message, false)){
                            message.splice(pos..pos+len, [14, 6, 300+idx as u16, 0]);
                            changed = true;
                        }
                    }
                    else {
                        for xx in 0..20 {
                            if let Some((pos, len, _)) = data.emblem_alias[xx].find_position(&message, false) {
                                message.splice(pos..pos+len, [14, 6, 300+xx as u16, 0]);
                                changed = true;
                            }
                        }
                    }
                }
                MessSwapType::EmblemAlias(idx) => {
                    let idx = (*idx) as usize;
                    if idx < 20 {
                        if let Some((pos, len, _)) = data.emblem_alias.get(idx+20).and_then(|m| m.find_position(&message, false)){
                            message.splice(pos..pos+len, [14, 6, 320+idx as u16, 0]);
                            changed = true;
                        }
                    }
                    else {
                        for xx in 0..20 {
                            if let Some((pos, len, _)) = data.emblem_alias[xx+20].find_position(&message, false) {
                                message.splice(pos..pos+len, [14, 6, 320+xx as u16, 0]);
                                changed = true;
                            }
                        }
                    }
                }
                MessSwapType::EmblemInvocation(idx) => {
                    let idx = (*idx) as usize;
                    if idx < 20 {
                        if let Some((pos, len, _)) = data.emblem_alias.get(idx+40).and_then(|m| m.find_position(&message, false)){
                            message.splice(pos..pos+len, [14, 6, 340+idx as u16, 0]);
                            changed = true;
                        }
                    }
                    else {
                        for xx in 0..20 {
                            if let Some((pos, len, _)) = data.emblem_alias[xx+40].find_position(&message, false) {
                                message.splice(pos..pos+len, [14, 6, 340+xx as u16, 0]);
                                changed = true;
                            }
                        }
                    }
                }
                MessSwapType::UnitGenderTextSwap { person_idx: _, txt_idx } => {
                    if let Some((pos, len, upper)) = data.gender.get(*txt_idx as usize).and_then(|m| m.find_position(message)) {
                        message.splice(pos..pos+len, x.create_tag_arguments( upper, 0));
                        changed = true;
                    }
                }
                MessSwapType::EmblemGenderTextSwap { emblem_idx: _ , txt_idx  } => {
                    if let Some((pos, len, upper)) = data.gender.get(*txt_idx as usize).and_then(|m| m.find_position(message)) {
                        message.splice(pos..pos+len, x.create_tag_arguments( upper, 0));
                        changed = true;
                    }
                }
                _ => {}
            }
        });
        changed
    }
}