use std::sync::RwLock;
use engage::{
    app::{random_2::*, Mess, MsgFile},
    nintendo::message_studio::lib::IBinMsgFileMethods,
    system::collections::generic::{IDictionary_2Methods, InsertionBehavior},
};
use outfit_core::Randomizer;
use unity::{Cast, Il2CppString, IntPtr};

use crate::{
    dvc::{
        mess::{MESS_REPLACEMENT_DATA, swapdata::MessageList},
        talk::window::WindowReplacementType
    },
    DVCVariables,
    gamedata::{EmblemPool, PersonPool}
};
use crate::enums::PIDS;

pub static mut TMP_REPLACE: ReplacementMessData = ReplacementMessData::new();
pub const HUB_MID: [&str; 20] = [
    "MID_HUB_Dinner_@_A", "MID_HUB_Dinner_@_B", "MID_HUB_Dinner_@_C", "MID_HUB_Dinner_@_Foodstuff",
    "MID_HUB_Dinner_@_Before", "MID_HUB_Dinner_@_After", "MID_HUB_Dinner_@_After2", "MID_HUB_Dinner_@_Bento",
    "MID_HUB_@_Cook", "MID_HUB_@_Gift_Good", "MID_HUB_@_Gift_Normal", "MID_HUB_@_Gift_Bad", "MID_HUB_@_Gift_Bad_2",
    "MID_KR_@_TravelTalk_Common_01", "MID_KR_@_TravelTalk_Common_02", "MID_KR_@_TravelTalk_Common_03",
    "MID_KR_@_HeroMVP","MID_KR_@_HeroLVUP", "MID_KR_@_NoParty", "MID_KR_@_ManyKill",
];
const STATS: [&str; 9] = ["Str", "Tec", "Spd", "Lck", "Def", "Mag", "Res", "Phy", "Mov"];
pub struct ReplacementMessData {
    pub chapter_lines: Vec<Vec<u16>>,
    pub other: Vec<Vec<u16>>,
    pub random_data: Vec<u16>,
    pub pt_indexes: Vec<Vec<u8>>,
    missing_pt_data: Vec<u16>,
    argument_mess_buffer: Vec<u16>,
    pub argument_mess: Vec<u16>,
}
impl ReplacementMessData {
    pub const fn new() -> Self {
        Self {
            argument_mess_buffer: Vec::new(),
            argument_mess: Vec::new(), missing_pt_data: Vec::new(),
            chapter_lines: Vec::new(), other: Vec::new(), random_data: Vec::new(), pt_indexes: Vec::new() }
    }
    pub fn get() -> &'static mut Self { unsafe { &mut TMP_REPLACE } }
    pub fn clear() { Self::get().chapter_lines.clear(); }
    pub fn add(str: Vec<u16>, chapter: bool) {
        if chapter { Self::get().chapter_lines.push(str); }
        else { Self::get().other.push(str); }
    }
    pub fn init_generic_mess_data() {
        let data = Self::get();
        let person_pool = PersonPool::get();
        let count = person_pool.pids.len() - 40;
        let dictionary = Mess::s_mess_data_dictionary();
        let vandre = Mess::get("MPID_Vandre").to_string().encode_utf16().collect::<Vec<u16>>();
        let mut bento_name = crate::dvc::mess::copy_from_u16_ptr(Mess::get_int_ptr("MIID_VandrBento").as_ptr());
        let mut bento_help = crate::dvc::mess::copy_from_u16_ptr(Mess::get_int_ptr("MIID_H_VandrBento").as_ptr());
        data.argument_mess = vec![];
        let mut argument_end_points = vec![];
        let mut end = 0;
        if let Some(pos) = bento_name.windows(vandre.len()).position(|w| w == vandre) { bento_name.splice(pos..pos + vandre.len(), [14, 1, 0, 0]); }
        argument_end_points.push((end, bento_name.len()));
        data.argument_mess.extend(bento_name.iter());
        end = data.argument_mess.len();
        if let Some(pos) = bento_help.windows(vandre.len()).position(|w| w == vandre){ bento_help.splice(pos..pos+vandre.len(), [14, 1, 0, 0]); }
        argument_end_points.push((end, bento_help.len()));
        data.argument_mess.extend(bento_help.iter());
        end = data.argument_mess.len();
        STATS.iter().enumerate().for_each(|(_i, s)|{
            for pre in ["MSID", "MSID_H"] {
                for n in 1..6 {
                    let mid = format!("{}_{}_{}", pre, s, n);
                    if Mess::is_exist(mid.as_str()) {
                        let number = (48 + n) as u16;
                        let mut str = crate::dvc::mess::copy_from_u16_ptr(Mess::get_int_ptr(mid.as_str()).as_ptr());
                        if let Some(pos) = str.iter().position(|v| *v == number) {
                            str.splice(pos..pos+1, [14, 1, 0, 0]);
                            data.argument_mess.extend(str.iter());
                            argument_end_points.push((end, str.len()));
                            end = data.argument_mess.len();
                            break;
                        }
                    }
                }
            }
        });
        argument_end_points.iter().enumerate().for_each(|(i, end)|{
            let ptr = data.argument_mess[end.0..].as_ptr();
            let int_ptr = unsafe { std::mem::transmute::<_, IntPtr>(ptr) };
            let mid = format!("MID_DVC_{}", i);
            dictionary.try_insert(mid.as_str().into(), int_ptr, InsertionBehavior::overwrite_existing());
        });
        data.argument_mess_buffer = vec![0; 18 * count];
        let mut stat_mess_offsets = vec![];
        STATS.iter().enumerate().for_each(|(i, stat)| {
            ["MSID", "MSID_H"].iter().enumerate().for_each(|(j, prefix)|{
                let slot = 2 + i*2 + j;
                for n in 1..6 {
                    let mid = format!("{}_{}_{}", prefix, stat, n);
                    if !Mess::is_exist(mid.as_str()) {
                        stat_mess_offsets.push((mid.clone(), data.argument_mess_buffer.len()));
                        data.argument_mess_buffer.extend([14, 14, n as u16, 0, 14, 100 + slot as u16, 0, 0, 0]);
                    }
                }
            })
        });
        let mut ptr = data.argument_mess_buffer.as_mut_ptr();
        person_pool.pids.iter().enumerate().filter(|v| v.0 == 0 || v.0 > 40)
            .for_each(|(idx, v)| {
                [14, 12, idx as u16, 0, 14, 100, 0, 0, 0, 14, 12, idx as u16, 0, 14, 101, 0, 0, 0].into_iter().enumerate().for_each(|(i, v)|{
                    unsafe { ptr.add(i).replace(v) };
                });
                let mid = format!("MIID_{}Bento", v.1.as_str());
                let mid2 = format!("MIID_H_{}Bento", v.1.as_str());
                if !Mess::is_exist(mid.as_str()) {
                    let int_ptr = unsafe { std::mem::transmute::<_, IntPtr>(ptr) };
                    dictionary.try_insert(mid.as_str().into(), int_ptr, InsertionBehavior::overwrite_existing());
                }
                unsafe { ptr = ptr.add(9); }
                if !Mess::is_exist(mid2.as_str()) {
                    let int_ptr = unsafe { std::mem::transmute::<_, IntPtr>(ptr) };
                    dictionary.try_insert(mid2.as_str().into(), int_ptr, InsertionBehavior::overwrite_existing());
                }
                unsafe { ptr = ptr.add(9); }
            });
    }
    pub fn init_random_data(cook: bool) {
        let data = Self::get();
        let person_pool = PersonPool::get();
        let count = person_pool.pids.len() - 40;
        let mut ptr = data.random_data.as_mut_ptr();
        let dictionary = Mess::s_mess_data_dictionary();
        let msg_count =  HUB_MID.len() + 23;
        if cook {
            person_pool.pids.iter().enumerate().filter(|v| v.0 == 0 || v.0 > 40)
                .for_each(|(idx, v)| {
                    let i = if idx == 0 { 0 } else { idx - 40 };
                    for x in 0..8 {
                        let pos = (msg_count*i + x)*5;
                        data.random_data[pos] = 14;
                        data.random_data[pos + 1] = 50 + x as u16;
                        let mid = HUB_MID[x].replace("@", v.1.as_str());
                        if !Mess::is_exist(mid.as_str()) {
                            ptr = data.random_data[pos..].as_mut_ptr();
                            let int_ptr = unsafe { std::mem::transmute::<_, IntPtr>(ptr) };
                            dictionary.try_insert(mid.as_str().into(), int_ptr, InsertionBehavior::overwrite_existing());
                        }
                    }
                });
        }
        else {
            data.random_data.clear();
            data.random_data = vec![0; (count + 1) * 5 * (23 + HUB_MID.len())];
            person_pool.pids.iter().enumerate().filter(|v| v.0 == 0 || v.0 > 40)
                .for_each(|(idx, v)| {
                    let i = if idx == 0 { 0 } else { idx - 40 };
                    for x in 8..HUB_MID.len() {
                        let pos = (msg_count*i + x)*5;
                        data.random_data[pos] = 14;
                        data.random_data[pos + 1] = 50 + x as u16;
                        let mid = HUB_MID[x].replace("@", v.1.as_str());
                        if !Mess::is_exist(mid.as_str()) {
                            ptr = data.random_data[pos..].as_mut_ptr();
                            let int_ptr = unsafe { std::mem::transmute::<_, IntPtr>(ptr) };
                            dictionary.try_insert(mid.as_str().into(), int_ptr, InsertionBehavior::overwrite_existing());
                        }
                    }
                    for x in 0..23 {
                        let pt_index = if x < 21 { x } else if x == 21 { 34 } else { 35 };
                        let xx = HUB_MID.len() + x;
                        let pos = (msg_count*i + xx)*5;
                        data.random_data[pos] = 14;
                        data.random_data[pos + 1] = 50 + xx as u16;
                        let mid = format!("MID_HUB_{}_PT{:02}", v.1.as_str(), pt_index);
                        if !Mess::is_exist(mid.as_str()) {
                            ptr = data.random_data[pos..].as_mut_ptr();
                            let int_ptr = unsafe { std::mem::transmute::<_, IntPtr>(ptr) };
                            dictionary.try_insert(mid.as_str().into(), int_ptr, InsertionBehavior::overwrite_existing());
                        }
                    }
                });
            let mut missing_pt: Vec<(u16, Vec<u8>)> = vec![];
            let mut total = 0;
            for i in 0..23 {
                let mut vec = Vec::new();
                let mut missing = Vec::new();
                let pt_index = if i < 21 { i } else if i == 21 { 34 } else { 35 };
                for x in 1..41 {
                    let mid = format!("MID_HUB_{}_PT{:02}", person_pool.pids[x].1.as_str(), pt_index);
                    if Mess::is_exist(mid.as_str()) { vec.push(x as u8); }
                    else if i != 7 && i != 8 && i < 21 { missing.push(x as u8); }
                }
                if !missing.is_empty() && i < 21 {
                    total += missing.len();
                    missing_pt.push((i, missing));
                }
                data.pt_indexes.push(vec);
            }
            data.missing_pt_data = vec![0; 5*total];
            let mut pos = 0;
            missing_pt.iter().for_each(|(pt_index, missing)|{
                missing.iter().for_each(|&x|{
                    let mid = format!("MID_HUB_{}_PT{:02}", person_pool.pids[x as usize].1.as_str(), *pt_index);
                    data.missing_pt_data[5*pos] = 14;
                    data.missing_pt_data[5*pos + 1] = 50 + (HUB_MID.len() as u16) + pt_index;
                    data.missing_pt_data[5*pos + 2] = x as u16;
                    let ptr = data.missing_pt_data[5*pos..].as_mut_ptr();
                    let int_ptr = unsafe { std::mem::transmute::<_, IntPtr>(ptr) };
                    dictionary.try_insert(mid.as_str().into(), int_ptr, InsertionBehavior::overwrite_existing());
                    pos += 1;
                });
            });
        }
    }
    pub fn get_random_mid(replacement_character: i32, mid_index: i32) -> String {
        let i = replacement_character as usize;
        let mid_index = mid_index as usize;
        let person_pool = PersonPool::get();
        if mid_index < HUB_MID.len() { HUB_MID[mid_index].replace("@", person_pool.pids[i].1.as_str()) }
        else {
            let pt = mid_index - HUB_MID.len();
            let pt_index = if pt < 21 { pt } else if pt == 21 { 34 } else { 35 };
            format!("MID_HUB_{}_PT{:02}", person_pool.pids[i].1.as_str(), pt_index)
        }
    }
    pub fn randomize_hub_data(with_cook: bool) {
        let data = Self::get();
        let rng = Random_2::get_system();
        let person_pool = PersonPool::get();
        let count = person_pool.pids.len() - 40;
        let mid_count = HUB_MID.len();
        let message_count = mid_count + 23;
        let start = if with_cook { 0 } else { 8 };
        for i in 0..count {
            let char = if i == 0 { 0 } else { i + 40 } as u16;
            for c in start..mid_count {
                let mut selection = rng.get_value_2(40) + 1;
                let mut mid = HUB_MID[c].replace("@", person_pool.pids[selection as usize].1.as_str());
                while !Mess::is_exist(mid.as_str()) {
                    selection = rng.get_value_2(40) + 1;
                    mid = HUB_MID[c].replace("@", person_pool.pids[selection as usize].1.as_str());
                }
                data.random_data[i*message_count*5 + 5*c+2] = char | ((selection as u16) << 8);
            }
            for c in 0..23 {
                if let Some(v) = data.pt_indexes[c].get_random_element(rng) {
                    data.random_data[i*message_count*5 + 5*(c + mid_count)+2] = char | ((*v as u16) << 8);
                }
            }
        }
        data.missing_pt_data.chunks_mut(5).for_each(|c|{
            let pt_index = c[1] - 50 - mid_count as u16;
            let char = c[2] & 255;
            if let Some(v) = data.pt_indexes[pt_index as usize].get_random_element(rng) {
                c[2] = char | (*v as u16) << 8;
            }
        });
    }
}
pub fn edit_msgfile_2(filename: Il2CppString, chapter: bool) {
    let (_found, file) = Mess::s_mess_file_dictionary().try_get_value(filename);
    if !file.is_null() { edit_msgfile(file, chapter); }
    crate::dvc::mess::puppet::edit_demo_mess_file(filename);
}
pub fn edit_msgfile_for_chapter(chapter_filename: Il2CppString) {
    let (_found, msg_file) = Mess::s_mess_file_dictionary().try_get_value(chapter_filename);
    if msg_file.is_null() { return; }
    ReplacementMessData::clear();
    let chapter_index = DVCVariables::get_chapter_index();
    let custom_emblem;
    if chapter_index >= 32 && chapter_index < 45 {
        custom_emblem = EmblemPool::get_emblem_by_paralogue_index(chapter_index)
            .filter(|i| EmblemPool::get_replacement_god_3(*i, false).is_some_and(|g| EmblemPool::is_custom(g)));
    }
    else { custom_emblem = None; }
    let num = msg_file.get_text_num();
    let dictionary = Mess::s_mess_data_dictionary();
    let emblem_names = chapter_index < 27;

    let replace_list = MESS_REPLACEMENT_DATA.get_or_init(|| RwLock::new(MessageList::init())).read().unwrap();
    for i in 0..num {
        let ptr = msg_file.get_text_2(i).as_ptr();
        let label = msg_file.get_label(i);
        let mut str = crate::dvc::mess::copy_from_u16_ptr(ptr);
        let len = str.len();
        replace_talk_window_pid_chapter(&mut str, label, chapter_index, custom_emblem.is_some());
        replace_list.replace_with_tags(&mut str, emblem_names);
        if let Some(idx) = custom_emblem.as_ref() { replace_list.replace_for_custom_emblem_paralogue(&mut str, *idx); }
        let len2 = str.len();
        if str.len() <= len { for i in 0..len2 { unsafe { ptr.add(i).replace(str[i]); } } }
        else {
            let new_ptr = unsafe { std::mem::transmute::<_, IntPtr>(str.as_mut_ptr()) };
            dictionary.try_insert(label, new_ptr, InsertionBehavior::overwrite_existing());
            ReplacementMessData::add(str, true);
        }
    }
    crate::dvc::mess::puppet::edit_demo_mess_file(chapter_filename);
}
pub fn edit_msgfile(msg_file: MsgFile, chapter: bool){
    let num = msg_file.get_text_num();
    let replace_list = MESS_REPLACEMENT_DATA.get_or_init(|| RwLock::new(MessageList::init())).read().unwrap();
    let chapter_index;
    let custom_emblem;
    if chapter {
        chapter_index = DVCVariables::get_chapter_index();
        if chapter_index >= 32 && chapter_index < 45 {
            custom_emblem = EmblemPool::get_emblem_by_paralogue_index(chapter_index)
                .filter(|i| EmblemPool::get_replacement_god_3(*i, false).is_some_and(|g| EmblemPool::is_custom(g)));
        }
        else { custom_emblem = None; }
        ReplacementMessData::clear();
    }
    else {
        chapter_index = 30;
        custom_emblem = None;
    }
    let dictionary = Mess::s_mess_data_dictionary();
    let emblem_names = chapter_index < 27;
    for i in 0..num {
        let ptr = msg_file.get_text_2(i).as_ptr();
        let label = msg_file.get_label(i);
        let mut str = crate::dvc::mess::copy_from_u16_ptr(ptr);
        let len = str.len();
        replace_talk_window_pid_chapter(&mut str, label, chapter_index, custom_emblem.is_some());
        replace_list.replace_with_tags(&mut str, emblem_names);
        if let Some(idx) = custom_emblem.as_ref() { replace_list.replace_for_custom_emblem_paralogue(&mut str, *idx); }
        let len2 = str.len();
        if str.len() <= len { for i in 0..len2 { unsafe { ptr.add(i).replace(str[i]); } } }
        else {
            let new_ptr = unsafe { std::mem::transmute::<_, IntPtr>(str.as_mut_ptr()) };
            dictionary.try_insert(label, new_ptr, InsertionBehavior::overwrite_existing());
            ReplacementMessData::add(str, chapter);
        }
    }
}

fn is_m022_character_talk(label: Il2CppString, chapter_index: i32) -> bool {
    if chapter_index != 22 { false }
    else {
        let str = label.to_rust_string();
        str.contains("MID_TK_") && (str.contains("1_#0") || str.contains("2_#0"))
    }
}
pub fn replace_talk_window_pid_chapter(str: &mut Vec<u16>, label: Il2CppString, chapter_index: i32, custom_emblem: bool) {
    if is_m022_character_talk(label, chapter_index) {
        for i in 0..41 { WindowReplacementType::MapMindSwap.try_replace_pid(str, i, false, None); }
    }
    else {
        for i in 0..41 { WindowReplacementType::RecruitmentSwap.try_replace_pid(str, i, false, None); }
        if chapter_index < 27 || custom_emblem || (chapter_index >= 51 && chapter_index < 57 && label.contains("OP") || label.contains("ED")){
            for i in 0..24 {
                WindowReplacementType::RecruitmentSwap.try_replace_pid(str, i, true, None);
            }
        }
        if chapter_index > 60 {
            let chapter = format!("E00{}_", chapter_index - 60);
            for x in 36..41 {
                let pid = PIDS[x].replace("PID_", &chapter);
                WindowReplacementType::RecruitmentSwap.replace_with_tag(str, &pid, x as u16, None);
                WindowReplacementType::RecruitmentSwap.try_alias_swap(str);
            }
        }
    }
}