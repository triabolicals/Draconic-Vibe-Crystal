use engage::gamedata::{Gamedata, GodData};
use engage::language::*;
use engage::mess::{Mess, MessStaticFields};
use unity::prelude::*;
use crate::enums::{EMBLEM_GIDS, MPIDS, RINGS};
use crate::message::swap::copy_from_u16_ptr;
use crate::message::swap_command::TalkLine;

const NPC_MPIDS: [&str; 22] = [
    "MPID_Sepia", "MPID_Gris", "MPID_Marron", "MPID_Lumiere", "MPID_M003_Boss", "MPID_M004_Boss",
    "MPID_M005_Boss", "MPID_M006_Boss", "MPID_M013_BossA", "MPID_M013_BossB", "MPID_Sombre",
    "MPID_JeanFather","MPID_JeanMother","MPID_S002_Boss","MPID_Eve","MPID_Morion","MPID_Hyacinth",
    "MPID_Sfoglia","MPID_AccessoriesShop","MPID_BlackSmith","MPID_ItemShop","MPID_WeaponShop",
];

#[derive(Clone)]
pub struct MessDataString{
    pub string: String,
}

pub struct GenderConditionString {
    male: String,
    female: String,
    local: Vec<(String, String)>,
}
impl GenderConditionString {
    pub fn from_str(line: &str) -> Option<Self> {
        if line.starts_with("---") { return None; }
        let mut spilt = line.split(":");
        let male = spilt.next()?.to_string();
        let female = spilt.next()?.to_string();
        let mut local = Vec::new();
        while let Some((s1, s2)) = spilt.next().zip(spilt.next()) {
            local.push((s1.to_string(), s2.to_string()));
        }
        Some(Self{male, female, local})
        
    }
    pub fn from(male: &str, female: &str) -> Self {
        Self { male: male.to_string(), female: female.to_string(), local: Vec::new() }
    }
    pub fn get(&self, gender: i32, first_char_upper: bool) -> &'static mut Il2CppString {
        let str = if gender == 1 { self.male.as_str() } else { self.female.as_str() };
        let mut chars = str.chars();
        match chars.next() {
            Some(first) => {
                let first =
                    if first_char_upper { first.to_uppercase().collect::<String>() }
                    else { first.to_lowercase().collect::<String>() };
                first + chars.as_str()
            },
            None => str.to_string(),
        }.into()
    }
    pub fn find_position(&self, message: &mut Vec<u16>) -> Option<(usize, usize, bool)> {
        let mut try_order = [1, 2];
        if self.female.len() > self.male.len() { try_order = [2, 1]; }
        self.contains_by_gender(message, try_order[0])
            .or_else(|| self.contains_by_gender(message, try_order[1]))
    }
    pub fn find_from(&self, message: &mut Vec<u16>, pos: usize) -> Option<(usize, usize, bool)> {
        let len_m = self.male.encode_utf16().count();
        let len_f = self.female.encode_utf16().count();
        let start_m = if len_m < pos { pos - len_m } else { 0 };
        let start_f = if len_f < pos { pos - len_f } else { 0 };
        find_position(message, &self.male, true, Some(start_m)).filter(|x| x.0 <= pos)
            .or_else(||{
                find_position(message, &self.female, true, Some(start_f)).filter(|x| x.0 <= pos)
            })
    }
    pub fn contains_by_gender(&self, message: &Vec<u16>, gender: i32) -> Option<(usize, usize, bool)> {
        if gender == 1 { 
            self.local.iter().find_map(|v| find_position(message, &v.0, true, None))
                .or_else(||find_position(message, &self.female, true, None))
        }
        else {
            self.local.iter().find_map(|v| find_position(message, &v.1, true, None))
                .or_else(|| find_position(message, &self.female, true, None))
        }
    }
}
impl MessDataString {
    pub fn from(str: &Il2CppString) -> Self { Self { string: str.to_string() } }
    pub fn from_str(str: &str) -> Self { Self { string: String::from(str) } }
    pub fn from_slice(slice: &[u16]) -> Option<Self> {
        let str = String::from_utf16(slice).ok()?;
        Some(Self::from_str(&str))
    }
    pub fn to_str(&self) -> &'static mut Il2CppString { self.string.as_str().into() }
    pub fn find_position(&self, message: &Vec<u16>, ignore_first_case: bool) -> Option<(usize, usize, bool)> { 
        find_position(&message, &self.string, ignore_first_case, None)
    }
    pub fn find_position_for_name(&self, message: &Vec<u16>) -> Option<(usize, usize, bool)> {
        find_position(&message, &self.string, false, None)
    }
    pub fn find_from(&self, message: &Vec<u16>, ignore_case: bool, init_position: usize) -> Option<(usize, usize, bool)> {
        find_position(&message, &self.string, ignore_case, Some(init_position))
    }
}

pub struct MessageList {
    pub person_list: Vec<MessDataString>,
    pub emblem_list: Vec<MessDataString>,
    pub alias: Vec<MessDataString>,
    pub emblem_alias: Vec<MessDataString>,
    pub gender: Vec<GenderConditionString>,
    pub hero_jobs: Vec<MessDataString>,
    pub item_kinds: [Vec<MessDataString>; 10],
    pub honorifics: Vec<MessDataString>,
    pub cmd_function_anim: MessDataString,
}

impl MessageList {
    pub fn init() -> Self {
        let cmd_function_anim = MessDataString::from_str("キャラアニメーター切替");
        let mut item_kinds = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), ];
        let person_list: Vec<MessDataString> = MPIDS.iter().chain(NPC_MPIDS.iter())
            .map(|mpid| { MessDataString::from(Mess::get(mpid)) }).collect();
        let emblem_list =
            EMBLEM_GIDS.iter()
                .flat_map(|gid| GodData::get(gid))
                .map(|gd| MessDataString::from(Mess::get(gd.mid)))
                .collect();

        let mut emblem_alias: Vec<_> =
            RINGS.iter().enumerate()
                .filter(|x| x.0 < 19)
                .map(|(_, x)| MessDataString::from(Mess::get(format!("MGID_Ring_{}", x))))
                .collect();
        emblem_alias.push(MessDataString::from(Mess::get("MGID_Ring_Lueur")));
        let mut alias: Vec<_> = MPIDS.iter().map(|m|{
            MessDataString::from(Mess::get(m.replace("MPID_", "MPID_alias_")))
        }).collect();
        
        let lines =
        match Language::get_lang() {
            LanguageLangs::USFrench | LanguageLangs::EUFrench => { include_str!("../../data/text_swaps/fr.txt").lines() }
            _ => { include_str!("../../data/text_swaps/en.txt").lines() }
        };
        let mut section = 0;
        let mut hero_jobs = vec![];
        let mut gender = vec![];
        let mut item_kind = 0;
        lines.for_each(|line| {
            if line.starts_with("END") { section += 1; }
            else {
                match section {
                    0 => { emblem_alias.push(MessDataString::from_str(line)); }
                    1 => { hero_jobs.push(MessDataString::from_str(line)); }
                    2 => {
                        if let Some(entry) = GenderConditionString::from_str(line) { gender.push(entry); }
                        else { gender.push(GenderConditionString::from("MALE_EMPTY", "FEMALE_EMPTY")); }
                    }
                    3 => {
                        if item_kind < 10 {
                            item_kinds[item_kind] = line.split(",").map(|s| MessDataString::from_str(s)).collect();
                            item_kind += 1;
                        }
                    }
                    4 => { alias.push(MessDataString::from_str(line)); }
                    _ => {}
                }
            }
        });
        
        // emblem_alias.iter().enumerate().for_each(|(i, m)| { println!("Emblem Alias #{}: {}", i, m.to_str()); });
        let sf = Il2CppClass::from_name("App", "Mess").unwrap().get_static_fields_mut::<MessStaticFields>();
        sf.mess_data_dictionary.entries.iter().filter_map(|v| v.key.filter(|v| v.str_contains("MID_RULE")))
            .for_each(|rule|{
                let ptr = Mess::get_int_ptr_mut(rule);
                let mut copy = copy_from_u16_ptr(ptr);
                let len = copy.len();
                let mut changed = false;
                person_list.iter().enumerate().for_each(|(i,p)|{
                    if let Some((pos, len, _)) = p.find_position(&copy, false) {
                        copy.splice(pos..pos + len, [14, 6, 100+i as u16, 0]);
                        changed = true;
                    }
                });
                if changed && copy.len() <= len {
                    // println!("Edited {} in place", rule);
                    for x in 0..copy.len() { unsafe { *ptr.add(x) = copy[x]; } }
                }
            });
        let list =       
            Self { 
                cmd_function_anim,
                person_list, alias, gender, emblem_list, emblem_alias, 
                hero_jobs, item_kinds, honorifics: vec![], 
            };
        include_str!("label_swap.txt").lines().flat_map(|l| TalkLine::new(l))
            .for_each(|line|{
                let ptr = Mess::get_int_ptr_mut(line.mid.as_str());
                let mut s = copy_from_u16_ptr(ptr);
                let original_len = s.len();
                if original_len > 0 {
                    if line.execute(&mut s, &list) {
                        if s.len() <= original_len {
                            // println!("Edited {} in place", line.mid);
                            for x in 0..s.len() { unsafe { *ptr.add(x) = s[x]; } }
                        }
                    }
                }
            });
        list
    }
}
fn find_position(message: &Vec<u16>, string: &String, ignore_case: bool, start_from: Option<usize>) -> Option<(usize, usize, bool)>  {
    let mut v: Vec<char> =
        string.encode_utf16()
        .flat_map(|v_u16| char::from_u32(v_u16 as u32))
        .collect();

    let mut v2: Vec<char> = message.iter()
        .flat_map(|x| char::from_u32(*x as u32))
        .collect();

    let start = start_from.unwrap_or(0);
    let length = v.len();
    if length + start > message.len() { return None; }
    if ignore_case {
        v = v.iter().map(|c| c.to_uppercase()).flatten().collect();
        v2 = v2.iter().map(|c| c.to_uppercase()).flatten().collect();
    }
    if v2[start..length+start]
        .iter()
        .zip(v.iter())
        .all(|(x1, x2)| (x1 == x2) || (x1.is_whitespace() && x2.is_whitespace()))
{
        Some((start, length, message.get(start).and_then(|v| char::from_u32(*v as u32)).is_some_and(|v| v.is_uppercase())))
    }
    else {
        v2[start..]
            .windows(length + 2)
            .position(|w|
                (!w[0].is_alphabetic() && w.last().is_none_or(|v| !v.is_alphabetic() )) &&
                    w[1..length+1].iter().zip(v.iter())
                        .all(|(x1, x2)| (x1 == x2) || (x1.is_whitespace() && x2.is_whitespace()))
            )
            .map(|v|
                (
                    start+v+1,
                    length,
                    message.get(start+v+1)
                        .and_then(|v| char::from_u32(*v as u32))
                        .is_some_and(|v| v.is_uppercase()))
            )
    }
}

