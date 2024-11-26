use std::sync::Mutex;
use unity::prelude::*;
pub use engage::{
    random::Random,
    gamevariable::*, 
    gameuserdata::*,
    mess::*,
    tmpro::*,
};
use unity::il2cpp::object::Array;
use crate::enums::*;
use crate::randomizer::emblem::EMBLEM_ORDER;
use crate::utils::*;
use engage::gamedata::{Gamedata, PersonData, GodData};

pub static NAMES: Mutex<Names> = Mutex::new(Names{ 
    is_custom: false, 
    original_names: Vec::new(), 
    original_emblem_names: Vec::new(), 
    original_emblem_rings: Vec::new(), 
    original_emblem_nickname: Vec::new(),
    other_names: Vec::new(),
});

pub struct Names {
    pub original_names: Vec<String>,
    pub original_emblem_names: Vec<String>,
    pub original_emblem_rings: Vec<String>,
    pub original_emblem_nickname: Vec<String>,
    pub other_names: Vec<(i32, String)>,
    pub is_custom: bool,
}

pub fn fill_name_array() {
    let mut names = NAMES.lock().unwrap();
    names.original_names.clear();
    MPIDS.iter().for_each(|mpid|{
        names.original_names.push(Mess::get(mpid).to_string());
    });
    for x in 0..41 {
        if MPIDS[x] != PersonData::get(PIDS[x]).unwrap().get_name().unwrap().to_string() {
            names.is_custom = true;
            break;
        }
    }
    names.original_emblem_names.clear();
    names.original_emblem_rings.clear();
    names.original_emblem_nickname.clear();
    names.other_names.clear();

    RINGS.iter().for_each(|mid|{
        let mid1 = format!("MGID_{}", mid);
        names.original_emblem_names.push(Mess::get(mid1).to_string());
        let ring_h = format!("MGID_Ring_{}", mid);
        let mess = Mess::get(ring_h);
        names.original_emblem_rings.push(mess.to_string());
        let nickname = Mess::get(format!("MGEID_{}", mid)).to_string();
        names.original_emblem_nickname.push(nickname);
    });
    // 
    let other_pids = ["PID_ルミエル", "PID_ソンブル", "PID_イヴ", "PID_モリオン", "PID_ハイアシンス", "PID_スフォリア", "PID_セピア", "PID_グリ", "PID_マロン"];
    let mpids = ["MPID_Lumiere", "MPID_Sombre", "MPID_Eve", "MPID_Morion", "MPID_Hyacinth", "MPID_Sfoglia", "MPID_Sepia", "MPID_Gris", "MPID_Marron"];
    for x in 0..9 {
        let index = PersonData::get(other_pids[x]).unwrap().parent.index;
        let name = Mess::get(mpids[x]).to_string();
        names.other_names.push( (index, name) );
     }
}




static TEXT_REPLACE: Mutex<TextReplacer> = Mutex::new(
    TextReplacer{ mid: String::new(), replace: Vec::new(), current_position: 0, is_enabled: false, is_start: false, first_char: 0, diff: 0});

const VEYRE: [&str; 7] = ["PID_ヴェイル_フード", "PID_ヴェイル_包帯", "PID_ヴェイル_フード_顔出し", "PID_ヴェイル_白_悪", "PID_ヴェイル_黒_悪", "PID_ヴェイル_黒_善", "PID_ヴェイル_黒_善_角折れ" ];


#[unity::class("App", "EventDemoSequence")]
pub struct EventDemoSequence {
    proc: [u8; 0x78],
    pub demo_name: &'static Il2CppString,
    pub mess_file_name: &'static Il2CppString,
}
#[unity::class("App", "Talk3D.TalkSequence")]
pub struct TalkSequence {
    junk: [u8; 0xb8],
    pub mid: Option<&'static Il2CppString>,

}

#[skyline::from_offset(0x03782000)]
pub fn to_string(this: Option<&Il2CppString>, value: *const u8, method_info: OptionalMethod) -> &'static mut Il2CppString;

#[skyline::from_offset(0x03780660)]
pub fn to_char_array(this: &Il2CppString, method_info: OptionalMethod) -> &'static Array<u16>;

#[skyline::from_offset(0x020b8e10)]
fn get_current_mid(method_info: OptionalMethod) -> &'static Il2CppString;

fn is_character_specific() -> bool {
    let mid = unsafe { get_current_mid(None) };
    if str_contains(mid, "MID_KR_") { return true; }
    if str_contains(mid, "MID_GR_") { return true; }
    if str_contains(mid, "MID_DIE") { return true; }
    if str_contains(mid, "MID_RELIANCE") { return true; }
    if str_contains(mid, "MID_LVUP") { return true; }
    if str_contains(mid, "MID_HUB") { 
        if str_contains(mid, "MID_HUB_DLC") { return false; }
        if str_contains(mid, "MID_HUB_Mascot") { return false; }    
        return true; 
    }
    return false; 
}
fn is_emblem_paralogue() -> bool {
    let chapter = GameUserData::get_chapter().cid;
    if str_contains(chapter, "CID_S015") || str_contains(chapter, "CID_S001") || str_contains(chapter, "CID_S002") {
        return false
    }
    return str_contains(chapter, "CID_S0") || str_contains(chapter, "CID_G00");
}


#[skyline::hook(offset=0x024d3c50)]
pub fn get_cmd_info_from_cmd_lines_hook(this: &EventDemoSequence, mut cmd: &mut Il2CppString, method_info: OptionalMethod) -> u64 {
    //println!("MID for CMD: {} / {}", this.demo_name.to_string(), unsafe { get_current_mid(None) }.to_string());
    if str_contains(this.demo_name, "MID_RELIANCE") || str_contains(this.demo_name, "MID_GR") { return call_original!(this, cmd, method_info); }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        let mut list_pid: Vec<usize> = Vec::new();
        for x in 0..41 {
            if str_contains(cmd, PIDS[x]) {
                if PIDS[x] == "PID_ヴェイル" {  //Check for other Veyles
                    if VEYRE.iter().find(|&x| str_contains(cmd, x)).is_none() { list_pid.push(x);  }
                    else if str_contains(cmd, "PID_ヴェイル_包帯") { list_pid.push(43); }
                }
                else if PIDS[x] == "PID_ジェーデ" {
                    if str_contains(cmd, "PID_ジェーデ_兜あり") {  //Check for Jade Helmet
                        list_pid.push(42); 
                    }
                    else { list_pid.push(x); }
                }
                else { list_pid.push(x); }
            }
        }
        for x in list_pid {
            match x {
                42 => { //Alternative Jade
                    let pid = GameVariableManager::get_string("G_R_PID_ジェーデ");
                    cmd = replace_strs_il2str(cmd, "PID_ジェーデ_兜あり", pid);
                }
                43 => { //Alternative Veyle
                    let pid = GameVariableManager::get_string("G_R_PID_ヴェイル");
                    cmd = replace_strs_il2str(cmd, "PID_ヴェイル_包帯", pid);
                }
                _ => {
                    let pid = GameVariableManager::get_string(&format!("G_R_{}", PIDS[x]));
                    cmd = replace_strs_il2str(cmd, PIDS[x], pid);
                }
            }
        }
    }
    if GameVariableManager::get_number("G_Emblem_Mode") != 0 && !is_emblem_paralogue() {
        let mut list_pid2: Vec<usize> = Vec::new();
        for x in 0..EMBLEM_GIDS.len() {
            if str_contains(cmd, EMBLEM_GIDS[x]) { list_pid2.push(x); }
        }
        if str_contains(cmd, "GID_ディミトリ") { list_pid2.push(20); }
        if str_contains(cmd, "GID_クロード") { list_pid2.push(21); }
        for x in list_pid2 {
            if x == 20 {
                let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
                if gid.to_string() != "GID_エーデルガルト" {  cmd = replace_strs_il2str(cmd, "GID_ディミトリ", gid); }
            }
            else if x == 21 {
                let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
                if gid.to_string() != "GID_エーデルガルト" { cmd = replace_strs_il2str(cmd, "GID_クロード", gid); }
            }
            else {
                let pid = GameVariableManager::get_string(&format!("G_R_{}", EMBLEM_GIDS[x]));
                cmd = replace_strs_il2str(cmd, EMBLEM_GIDS[x], pid);
            }
        }
    }
    return call_original!(this, cmd, method_info);
}

#[skyline::hook(offset=0x020c5370)]
pub fn get_active_character_hook(this: &mut TalkPtr, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(this, method_info);
    if is_character_specific() { return result; }
    let str1 = result.to_string();
    if str_contains(result, "ジェーデ_兜あり") {
        if GameVariableManager::exist("G_R_PID_ジェーデ") {
            let pid = GameVariableManager::get_string("G_R_PID_ジェーデ");
            return il2_str_substring(pid, 4);
        }
    }
    if str_contains(result, "ヴェイル_包帯") {
        if GameVariableManager::exist("G_R_PID_ヴェイル") {
            let pid = GameVariableManager::get_string("G_R_PID_ヴェイル");
            return il2_str_substring(pid, 4);
        }
    }
    if GameVariableManager::exist(&format!("G_R_PID_{}", str1)) {
        //PID_
        let pid = GameVariableManager::get_string(&format!("G_R_PID_{}", str1));
        return il2_str_substring(pid, 4);
    }

    if GameVariableManager::exist(&format!("G_R_GID_{}", str1)) && !is_emblem_paralogue()  {
        let gid = GameVariableManager::get_string(&format!("G_R_GID_{}", str1));
        return il2_str_substring(gid, 4);
    }
    if ( str_contains(result, "ディミトリ") || str_contains(result, "クロード") ) && GameVariableManager::exist("G_R_GID_エーデルガルト") {
        let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
        return il2_str_substring(gid, 4);
    }
    if str_contains(result, "M000_マルス") {
        if GameVariableManager::exist("G_R_GID_マルス") && !is_emblem_paralogue()  {
            let gid = GameVariableManager::get_string("G_R_GID_マルス");
            return il2_str_substring(gid, 4);
        }
    }
    return result; 
}

#[unity::class("App", "Talk3D.TalkTagAddLetter")]
pub struct TalkTagAddLetter {
    pub is_line_feed_enable: bool,
    pub add_letter: u16,
    pub result: i32,
}
#[unity::class("App", "Talk3D.TalkPtr")]
pub struct TalkPtr {
    pub original: u64,
    pub now: u64,
}

pub struct TextReplacer {
    pub mid: String,
    pub replace: Vec<u16>,
    pub current_position: usize,
    pub is_enabled: bool,
    pub is_start: bool,
    pub first_char: u16,
    pub diff: i32,
}

impl TextReplacer {
    pub fn get_char(&mut self) -> u16 { 
        if self.current_position < self.replace.len() {
            let out = self.replace[self.current_position];
            self.current_position += 1;
            out
        }
        else { 
            self.is_enabled = false;
            self.is_start = false;
            15
        }
    }
    pub fn reset(&mut self) {
        self.current_position = 0;
        self.mid = String::new();
        self.replace.clear();
        self.is_enabled = false;
        self.is_start = false;
        self.first_char = 0;
        self.diff = 0;
    }
}

#[skyline::hook(offset=0x020c5260)]
pub fn talk_ptr(this: &mut TalkPtr, method_info: OptionalMethod) -> u16 {
    //if IS_GHAST {  return call_original!(this, method_info);  }
    let mut replacer = TEXT_REPLACE.lock().unwrap();
    if !replacer.is_enabled || replacer.replace.len() == 0 {
        return call_original!(this, method_info);
    }
    let original_now_ptr = this.now;
    let mut result = call_original!(this, method_info);
    let original = result;
    if result != 15 && result != 14 {
        if replacer.is_enabled {
            if !replacer.is_start {
                if replacer.first_char == result {
                    replacer.is_start = true;
                    result = replacer.get_char();
                    if replacer.diff > 0 {
                        this.now += (replacer.diff*2 ) as u64;
                    }
                }
            }
            else {  result = replacer.get_char();  }
        }
    }
    else if ( result == 15 || result == 14) && replacer.is_start && replacer.current_position < replacer.replace.len() {
        if result == 14 {
            let tag = call_original!(this, method_info);
            if tag == 6 { return replacer.get_char(); }
        }
        result = replacer.get_char();
        this.now = original_now_ptr;
        println!("Now: {}", this.now - this.original);
    }
    if result == 15 { 
        replacer.reset(); 
        this.now = original_now_ptr;
        return original; 
    }
    //println!("TalkPtr: active ({}) {} / {} => {}", replacer.is_enabled, result, original, std::char::from_u32(result as u32).unwrap() );
    return result;
}

#[skyline::hook(offset=0x020c7e90)]
pub fn calculate_str_width(this: &TalkSequence, add_character_count: i32, method_info: OptionalMethod) {
    //println!("TalkSequence Process Message: {} with Mid: {}", add_character_count, this.mid.unwrap().to_string());
    if let Some(mid) = this.mid { do_replacement(mid);  }
    call_original!(this, add_character_count, method_info);
}

pub fn do_replacement(mid: &Il2CppString) {
    let mut replacer = TEXT_REPLACE.lock().unwrap();
    if replacer.mid == mid.to_string() || replacer.current_position > 0 { return;  }
    replacer.reset();
    let mess = Mess::get(mid);
    if mess.to_u16().len() == 0 {
        replacer.reset();
        return; 
    }
    replacer.mid = mid.to_string();
    let original_str = mess.to_string();
    replacer.current_position = 0;

    replacer.first_char = mess.to_u16().to_vec()[0];
    let mut new_str = Il2CppString::new_static(original_str.clone());
    let names = NAMES.lock().unwrap();

    // Persons
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 && !is_character_specific() {
        let mut persons = Vec::new();
        for x in 0..41 {
            if new_str.contains(names.original_names[x].as_str()) { persons.push(x); }
        }
        persons.iter().for_each(|&x|{
            if GameVariableManager::get_string(format!("G_R2_{}", PIDS[x]).as_str()).to_string() != PIDS[0] {
                let old_name = names.original_names[x].clone().into();
                let new_name = format!("PERSON{}",x).into();
                new_str = unsafe { replace_str(new_str, old_name, new_name, None) };
            }
        });
        persons.iter().for_each(|&x|{
            let old_name = format!("PERSON{}",x).into();
            let new_name = Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", PIDS[x]).as_str()));
            new_str = unsafe { replace_str(new_str, old_name, new_name, None) };
        });
    }
    else if names.is_custom {
        let mut persons = Vec::new();
        for x in 1..41 { if new_str.contains(names.original_names[x].as_str()){ persons.push(x); } }

        persons.iter().for_each(|&x|{
            let old_name = names.original_names[x].clone().into();
            new_str = unsafe { replace_str(new_str, old_name, format!("PERSON{}",x).into(), None) };
        });
        let mut others: Vec<usize> = Vec::new();
    // NPCs
        for x in 0..9 { if new_str.contains(names.other_names[x].1.as_str()){ others.push(x); } }
        others.iter().for_each(|&x|{
            let old_name = names.other_names[x].1.clone().into();
            new_str = unsafe { replace_str(new_str, old_name, format!("OTHER{}",x).into(), None) };
        });

        persons.iter().for_each(|&x|{
            let old_name = format!("PERSON{}",x).into();
            let new_name = Mess::get_name(PIDS[x]);
            new_str = unsafe { replace_str(new_str, old_name, new_name, None) };
        });
    // NPCs
        others.iter().for_each(|&x|{
            let old_name = format!("OTHERS{}",x).into();
            let person = PersonData::try_index_get(names.other_names[x].0).unwrap();
            let new_name = Mess::get_name(person.pid);
            new_str = unsafe { replace_str(new_str, old_name, new_name, None) };
        });
    }
    if GameVariableManager::get_number("G_Emblem_Mode") != 0 && !is_character_specific() && !is_emblem_paralogue() {
        let mut emblems = Vec::new();
        for x in 0..23 {
            if new_str.contains(names.original_emblem_names[x].as_str()){
                if x == 19 || x == 20 {  emblems.push(12); }
                else if x == 21 {  emblems.push(18); }
                else if x == 22 {  emblems.push(11); }
                else {  emblems.push(x)}
            }
        }
        emblems.iter().for_each(|&x|{
            let old = names.original_emblem_names[x].clone().into();
            let new = format!("EMBLEM{}",x).into();
            new_str = unsafe { replace_str(new_str, old, new, None) };
        });

        emblems.iter().for_each(|&x|{
            let old = format!("EMBLEM{}",x).into();
            let gid = GameVariableManager::get_string(format!("G_R_{}", EMBLEM_GIDS[x]).as_str());
            let new = Mess::get(GodData::get(gid).unwrap().mid);
            new_str = unsafe { replace_str(new_str, old, new, None) };
        });
        emblems.clear();
    //Rings
        for x in 0..23 {
            if new_str.contains(names.original_emblem_rings[x].as_str()){
                if x == 19 || x == 20 {  emblems.push(12); }
                else if x == 21 {  emblems.push(18); }
                else if x == 22 {  emblems.push(11); }
                else {  emblems.push(x)}
            }
        }
        emblems.iter().for_each(|&x|{
            let old = names.original_emblem_rings[x].clone().into();
            let new = format!("RING{}",x).into();
            new_str = unsafe { replace_str(new_str, old, new, None) };
        });

        emblems.iter().for_each(|&x|{
            let old = format!("RING{}",x).into();
            let gid = GameVariableManager::get_string(format!("G_R_{}", EMBLEM_GIDS[x]).as_str());
            let god = GodData::get(gid).unwrap();
            let new = Mess::get(god.ring_name.unwrap());
            new_str = unsafe { replace_str(new_str, old, new, None) };
        });
        emblems.clear();
    // NickNames
        for x in 0..19 { if new_str.contains(names.original_emblem_nickname[x].as_str()){ emblems.push(x); } }

        emblems.iter().for_each(|&x|{
            let old = names.original_emblem_nickname[x].clone().into();
            let new = format!("NICK{}",x).into();
            new_str = unsafe { replace_str(new_str, old, new, None) };
        });

        emblems.iter().for_each(|&x|{
            let old = format!("NICK{}",x).into();
            let gid = EMBLEM_ORDER.lock().unwrap()[x] as usize;
            let new = Mess::get(format!("MGEID_{}", RINGS[gid]));
            new_str = unsafe { replace_str(new_str, old, new, None) };
        });

    }
    else {
        let mut emblems = Vec::new();
        for x in 0..23 {
            if new_str.contains(names.original_emblem_names[x].as_str()){ emblems.push(x); }
        }
        emblems.iter().for_each(|&x|{
            let gid = match x {
                19 => { "GID_ディミトリ"}
                20 => { "GID_クロード" }
                21 => { "GID_ルフレ" }
                22 => { "GID_エフラム" }
                _ => { EMBLEM_GIDS[x] }
            };
            let old = names.original_emblem_names[x].clone().into();
            let new = format!("EMBLEM{}", x);
                new_str = unsafe { replace_str(new_str, old, new.into(), None) };
        });
        emblems.iter().for_each(|&x|{
            let gid = match x {
                19 => { "GID_ディミトリ"}
                20 => { "GID_クロード" }
                21 => { "GID_ルフレ" }
                22 => { "GID_エフラム" }
                _ => { EMBLEM_GIDS[x] }
            };
            let old = format!("EMBLEM{}", x);
            let new = Mess::get(GodData::get(gid).unwrap().mid);
            new_str = unsafe { replace_str(new_str, old.into(), new, None) };
        });

    }
    let new_string = new_str.to_string();
    if new_string != original_str {
        replacer.replace = new_str.to_u16().to_vec();
        replacer.is_enabled = true;
        replacer.diff = ( mess.to_u16().to_vec().len() - replacer.replace.len() ) as i32;
    }
    else { replacer.reset(); }
}