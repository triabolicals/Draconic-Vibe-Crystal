use std::sync::{RwLock, RwLockReadGuard};
use engage::{
    app::{GodData, IGodDataMethods, IStructData_1Methods, mess::*, Language},
    Dictionary_2Ext,
    system::collections::generic::{IDictionary_2Methods, InsertionBehavior},
};
use unity::{IntPtr, prelude::*};
use crate::{
    dvc::mess::MESS_REPLACEMENT_DATA,
    enums::{EMBLEM_GIDS, MPIDS},
    dvc::{
        mess::{chapter::ReplacementMessData, copy_from_u16_ptr},
        talk::tag::TextReplacementKind
    },
    gamedata::EmblemPool,
    utils::il2str,
    enums::RINGS,
    gamedata::PersonPool
};
#[derive(Clone)]
pub struct MessDataString{ pub string: String, }

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
        while let Some((s1, s2)) = spilt.next().zip(spilt.next()) { local.push((s1.to_string(), s2.to_string())); }
        Some(Self{male, female, local})
    }
    pub fn from(male: &str, female: &str) -> Self {
        Self { male: male.to_string(), female: female.to_string(), local: Vec::new() }
    }
    pub fn get(&self, gender: i32, first_char_upper: bool) -> Il2CppString {
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
    pub fn find_position(&self, message: &mut Vec<u16>) -> Option<MessDataPosition> {
        let mut try_order = [1, 2];
        if self.female.len() > self.male.len() { try_order = [2, 1]; }
        self.contains_by_gender(message, try_order[0])
            .or_else(|| self.contains_by_gender(message, try_order[1]))
    }
    pub fn find_from(&self, message: &mut Vec<u16>, pos: usize) -> Option<MessDataPosition> {
        let len_m = self.male.encode_utf16().count();
        let len_f = self.female.encode_utf16().count();
        let start_m = if len_m < pos { pos - len_m } else { 0 };
        let start_f = if len_f < pos { pos - len_f } else { 0 };
        MessDataPosition::get(message, &self.male, true, Some(start_m))
            .or_else(|| MessDataPosition::get(message, &self.female, false, Some(start_f)))
            .filter(|x| x.position <= pos)
    }
    pub fn contains_by_gender(&self, message: &Vec<u16>, gender: i32) -> Option<MessDataPosition> {
        if gender == 1 {
            self.local.iter().find_map(|v|  MessDataPosition::get(message, &v.0, true, None))
                .or_else(|| MessDataPosition::get(message, &self.female, true, None))
        }
        else {
            self.local.iter().find_map(|v|  MessDataPosition::get(message, &v.1, true, None))
                .or_else(||  MessDataPosition::get(message, &self.female, true, None))
        }
    }
}
impl MessDataString {
    pub fn from(str: Il2CppString) -> Self { Self { string: str.to_rust_string() } }
    pub fn from_str(str: &str) -> Self { Self { string: String::from(str) } }
    pub fn from_slice(slice: &[u16]) -> Option<Self> {
        let str = String::from_utf16(slice).ok()?;
        Some(Self::from_str(&str))
    }
    pub fn to_str(&self) -> Il2CppString { self.string.as_str().into() }
    pub fn find_position(&self, message: &Vec<u16>, ignore_first_case: bool) -> Option<MessDataPosition> {
        MessDataPosition::get(&message, &self.string, ignore_first_case, None)
    }
    pub fn find_position_for_name(&self, message: &Vec<u16>) -> Option<MessDataPosition> {
        MessDataPosition::get(&message, &self.string, false, None)
    }
    pub fn find_from(&self, message: &Vec<u16>, ignore_case: bool, init_position: usize) -> Option<MessDataPosition> {
        MessDataPosition::get(&message, &self.string, ignore_case, Some(init_position))
    }
}
#[derive(Clone, Copy)]
pub struct MessDataPosition {
    pub position: usize,
    pub length: usize,
    pub line_break_pos: Option<u16>,
    pub upper: bool,
    pub all_upper: bool,
}
impl MessDataPosition {
    pub fn get(message: &Vec<u16>, string: &String, ignore_case: bool, start_from: Option<usize>) -> Option<MessDataPosition>  {
        let mut v: Vec<char> =
            string.encode_utf16()
                .flat_map(|v_u16| char::from_u32(v_u16 as u32))
                .collect();

        let mut v2: Vec<char> = message.iter()
            .flat_map(|x| char::from_u32(*x as u32))
            .collect();
        let message_copy = v2.clone();
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
            let mut length = length;
            let all_upper = message_copy[start..length+start].iter().all(|x| x.is_uppercase() || x.is_whitespace());
            let line_break_pos =
            v2[start..length+start].iter().position(|v| *v == '\n')
                .map(|p|{
                    let mut l = p as u16;
                    if let Some(end_char) = v2.get(start+length){
                        let n = *end_char as u16;
                        if n == 10 { 
                            length += 1;
                            l |= 32;
                        }
                        else if n == 32 {
                            length += 1;
                            l |= 64;
                        }
                    }
                    l
                });
            Some(
                Self {
                    position: start,
                    length,
                    line_break_pos,
                    all_upper,
                    upper: message.get(start).and_then(|v| char::from_u32(*v as u32)).is_some_and(|v| v.is_uppercase())
                }
            )
        }
        else {
            v2[start..]
                .windows(length + 2)
                .position(|w|
                    (!w[0].is_alphabetic() && w.last().is_none_or(|v| !v.is_alphabetic() )) &&
                        w[1..length+1].iter().zip(v.iter())
                            .all(|(x1, x2)| (x1 == x2) || (x1.is_whitespace() && x2.is_whitespace()))
                )
                .map(|p| {
                    let position = start + p + 1;
                    let mut length = length;
                    let all_upper = message_copy[start..length+start].iter().all(|x| x.is_uppercase() || x.is_whitespace());
                    let line_break_pos =
                    v2[position..length+position].iter().position(|v| *v == '\n')
                        .map(|p|{
                            let mut l = p as u16;
                            if let Some(end_char) = v2.get(position+length){
                                let n = *end_char as u16;
                                if n == 10 { length += 1;l |= 32; }
                                else if n == 32 { length += 1;l |= 64; }
                            }
                            l
                        });

                    Self {
                        position, line_break_pos, length, all_upper,
                        upper: message.get(start + p + 1).and_then(|v| char::from_u32(*v as u32)).is_some_and(|v| v.is_uppercase())
                    }
                })
        }
    }
    pub fn replace<I: IntoIterator<Item=u16>>(&self, message: &mut Vec<u16>, s: I) {
        let total = self.position + self.length;
        if total > message.len() {
            let n = total - message.len();
            for _ in 0..n { message.push(0); }
        }
        message.splice(self.position..self.position + self.length, s);
    }
    pub fn replace_for_text(&self, message: &mut Vec<u16>, kind: TextReplacementKind, value: u16){
        let total = self.position + self.length;
        if total > message.len() {
            let n = total - message.len();
            for _ in 0..n { message.push(0); }
        }
        if let Some(line_break) = self.line_break_pos.as_ref() {
            message.splice(self.position..self.position + self.length, kind.create_tag_with_flag(value, *line_break, self.all_upper));
        }
        else if self.all_upper {
            message.splice(self.position..self.position + self.length, kind.create_tag_with_flag(value, 0, self.all_upper));
        }
        else { message.splice(self.position..self.position + self.length, kind.create_tag(value)); }
    }
}

pub struct MessageList {
    pub person_list: Vec<MessDataString>,
    pub emblem_list: Vec<MessDataString>,
    pub alias: Vec<MessDataString>,
    pub rings: Vec<MessDataString>,
    pub emblem_alias: Vec<MessDataString>,
    pub gender: Vec<GenderConditionString>,
    pub hero_jobs: Vec<MessDataString>,
    pub item_kinds: [Vec<MessDataString>; 10],
    pub text: Vec<MessDataString>,
    pub honorifics: Vec<MessDataString>,
}
#[allow(non_upper_case_globals)]
impl MessageList {
    pub fn init() -> Self {
        let mut item_kinds = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), ];
        let person_list: Vec<MessDataString> = PersonPool::MPIDS.iter().map(|ascii| MessDataString::from(Mess::get(format!("MPID_{}", ascii)))).collect();
        let emblem_list: Vec<MessDataString> =
            EMBLEM_GIDS.iter()
                .map(|&gid| GodData::get(gid.into()))
                .map(|gd| MessDataString::from(Mess::get(gd.get_mid())))
                .collect();

        let rings: Vec<MessDataString> =
            EmblemPool::get_recruitment_gids().iter()
                .map(|v| GodData::get(v.as_str().into()))
                .map(|v| {
                    let ring_name = v.get_ringname();
                    let name =
                        if v.get_ringname().is_null() {
                            let engrave_word = v.get_engrave_word();
                            if !engrave_word.is_null() { Mess::get(engrave_word).to_rust_string() }
                            else { Mess::get(v.get_mid()).to_rust_string() }
                        }
                        else { Mess::get(ring_name).to_rust_string().trim_end_matches('.').to_string() };
                    MessDataString::from_str(name.as_str())
                }).collect();

        let mut emblem_alias = Vec::new();
        let mut alias: Vec<_> = MPIDS.iter().map(|m|{ MessDataString::from(Mess::get(m.replace("MPID_", "MPID_alias_"))) }).collect();
        let mut text = vec![];

        let lang = Language::s_lang().value;
        let lines =
            match lang {
                2|5 => { include_str!("../../data/text_swaps/fr.txt").lines() }
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
                    5 => { text.push(MessDataString::from_str(line)); }
                    _ => {}
                }
            }
        });
        replace_mess_char("MID_RULE_M", (&person_list, TextReplacementKind::UnitName), Some((&emblem_list, TextReplacementKind::EmblemName)), None);
        replace_mess_char("MID_RULE_DLC", (&person_list, TextReplacementKind::UnitName), None, None);
        replace_mess_char("MID_RULE_E", (&person_list, TextReplacementKind::UnitName), None, None);
        replace_mess_char("MID_RULE_S0", (&person_list, TextReplacementKind::UnitName), Some((&emblem_list, TextReplacementKind::RingParalogueSwap)), None);
        replace_mess_char("MID_RULE_G0", (&emblem_list, TextReplacementKind::EmblemNameNoSwap), None, None);
        replace_mess_char("MTTID_Ring_", (&rings, TextReplacementKind::EmblemRing), None, None);
        replace_mess_char("MID_TUT_NAVI_M022_GET_", (&rings, TextReplacementKind::EmblemRing), None, None);
        for i in 1..12 {
            let int_ptr = Mess::get_int_ptr(format!("MTID_Ring_{}", RINGS[i]));
            if !int_ptr.is_null() {
                let ptr: *mut u16 = int_ptr.as_ptr();
                [14, 23, i as u16, 0, 0, 0].into_iter().enumerate().for_each(|(i, tag)| {
                    unsafe { ptr.add(i).replace(tag) };
                });
            }
        }
        let list =
            Self {
                person_list, alias, gender, emblem_list, emblem_alias, rings,
                hero_jobs, item_kinds, honorifics: vec![], text,
            };
        /*
        include_str!("label_swap.txt").lines().flat_map(|l| crate::message::swap_command::TalkLine::new(l))
            .for_each(|line|{
                let ptr = Mess::get_int_ptr_mut(line.mid.as_str());
                let mut s = copy_from_u16_ptr(ptr);
                let original_len = s.len();
                if original_len > 0 {
                    if line.execute(&mut s, &list) {
                        if s.len() <= original_len {
                            for x in 0..s.len() { unsafe { *ptr.add(x) = s[x]; } }
                        }
                    }
                }
            });
        */
        ["MID_TUT_NAVI_VANDRE_FUNDS", "MID_HUB_Mascot_01_#003"].into_iter().for_each(|v| {
            let int_ptr = Mess::get_int_ptr(v).as_ptr();
            let mut str = copy_from_u16_ptr(int_ptr);
            let len1 = str.len();
            if let Some(pos) = list.person_list[1].find_position(&mut str, false) {
                pos.replace(&mut str, TextReplacementKind::UnitName.create_tag(1u16));
            }
            if str.len() <= len1 { for i in 0..str.len() { unsafe { int_ptr.add(i).replace(str[i]); } } }
        });
        list
    }
    pub fn replace_with_tags(&self, str: &mut Vec<u16>, emblem_name: bool){
        self.replace_for_unit_name(str);
        self.replace_for_hero_alias(str);
        if emblem_name {
            replace_table_entries_with_tag(str, &self.rings, TextReplacementKind::EmblemRing);
            replace_table_entries_with_tag(str, &self.emblem_alias, TextReplacementKind::EmblemAlias);
            replace_table_entries_with_tag(str, &self.emblem_list, TextReplacementKind::EmblemName);
        }
    }
    pub fn replace_for_hero_alias(&self, str: &mut Vec<u16>) {
        if let Some(pos) = self.hero_jobs.get(0).and_then(|s| s.find_position(str, false)){
            pos.replace(str, TextReplacementKind::HeroAlias.create_tag(0u16));
        }
        if let Some(pos) = self.alias.get(1).and_then(|s| s.find_position(str, false)){
            pos.replace(str, TextReplacementKind::HeroJob.create_tag(0u16));
        }
    }
    pub fn replace_for_unit_name(&self, str: &mut Vec<u16>) {
        self.person_list.iter().enumerate().for_each(|(char, p)|{
            if let Some(pos) = p.find_position(str, false) {
                if char != 41 { pos.replace(str, TextReplacementKind::UnitName.create_tag(char as u16)); }
                if char < 41 {
                    for i in 0..3 {
                        if let Some(pos2) = self.gender.get(i).and_then(|mess| mess.find_from(str, pos.position - 2)).filter(|p| p.position < pos.position) {
                            pos2.replace_for_text(str, TextReplacementKind::GenderStringSwap, char as u16 | ((i as u16) << 8));
                            break;
                        }
                    }
                }
            }
        });
    }
    pub fn replace_for_custom_emblem_paralogue(&self, str: &mut Vec<u16>, emblem_index: i32) {
        if let Some(pos) = self.emblem_list[emblem_index as usize].find_position(str, false) {
            pos.replace(str, TextReplacementKind::EmblemName.create_tag(emblem_index as u16));
        }
    }
    pub fn get() -> RwLockReadGuard<'static, MessageList> {
        MESS_REPLACEMENT_DATA.get_or_init(|| RwLock::new(Self::init())).read().unwrap()
    }
}
fn replace_mess_char(
    mess_key_prefix: &str,
    table: (&Vec<MessDataString>, TextReplacementKind),
    table2: Option<(&Vec<MessDataString>, TextReplacementKind)>,
    table3: Option<(&Vec<MessDataString>, TextReplacementKind)>,
) {
    let dictionary = Mess::s_mess_data_dictionary();
    let mut replace: Vec<(Il2CppString, IntPtr)> = vec![];
    dictionary.iter().filter(|(key, _)| il2str(*key).is_some_and(|v| v.starts_with(mess_key_prefix)))
        .for_each(|(key, _)| {
            let str_ptr = Mess::get_int_ptr(key).as_ptr();
            let mut str = copy_from_u16_ptr(str_ptr);
            let original_str = str.clone();
            let len1 = str.len();
            replace_table_entries_with_tag(&mut str, table.0, table.1);
            if let Some((table, kind)) = table2 { replace_table_entries_with_tag(&mut str, table, kind); }
            if let Some((table, kind)) = table3 { replace_table_entries_with_tag(&mut str, table, kind); }
            loop {
                if let Some(remove) = str.windows(2).position(|v| v[0] == 0 && v[1] == 0) {
                    str.remove(remove);
                }
                else { break; }
            }
            if str.len() <= len1 && !original_str.iter().eq(str.iter()) {
                for i in 0..str.len() { unsafe { str_ptr.add(i).replace(str[i]); } }
            }
            else {
                let new_ptr = unsafe { std::mem::transmute::<_, IntPtr>(str.as_mut_ptr()) };
                replace.push( (key, new_ptr));
                ReplacementMessData::add(str, false);
            }
        });
    replace.into_iter().for_each(|(key, ptr)| { dictionary.try_insert(key, ptr, InsertionBehavior::overwrite_existing()); });
}
fn replace_table_entries_with_tag(str: &mut Vec<u16>, table: &Vec<MessDataString>, kind: TextReplacementKind) {
    table.iter().enumerate().for_each(|(i, p)|{
        if let Some(pos) = p.find_position(str, false) {
            pos.replace_for_text(str, kind, i as u16);
        }
    });
}