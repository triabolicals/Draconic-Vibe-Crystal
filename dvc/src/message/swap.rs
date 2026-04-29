use engage::{
    gameuserdata::GameUserData,
    mess::MessStaticFields,
};
use unity::prelude::*;
use crate::{
    DVCVariables,
    enums::{EMBLEM_GIDS, EMBLEM_PARA, PIDS},
    message::{
        original::{MessDataString, MessageList},
        swap_kinds::MessSwapType,
        swap_command::*,
    },
    randomizer::data::EmblemPool,
};

pub const RING_PICTURE: [&str; 21] = [
    "Tex_Event_ItemPicture_01", "Tex_Event_ItemPicture_02", "Tex_Event_ItemPicture_04", "Tex_Event_ItemPicture_05",
    "Tex_Event_ItemPicture_06", "Tex_Event_ItemPicture_03", "Tex_Event_ItemPicture_08", "Tex_Event_ItemPicture_07",
    "Tex_Event_ItemPicture_09", "Tex_Event_ItemPicture_11", "Tex_Event_ItemPicture_10", "Tex_Event_ItemPicture_12",
    "Tex_Event_DLC_ItemPicture_07", "Tex_Event_DLC_ItemPicture_01", "Tex_Event_DLC_ItemPicture_02", "Tex_Event_DLC_ItemPicture_06",
    "Tex_Event_DLC_ItemPicture_03", "Tex_Event_DLC_ItemPicture_05", "Tex_Event_DLC_ItemPicture_04", "Tex_Event_ItemPicture_13",
    "Tex_Event_ItemPicture_14",
];

pub struct TextSwapper {
    pub current_talk_lines: Vec<Vec<u16>>,
    pub label_swaps: Vec<TalkSwapData>,
    pub original_data: MessageList,
    pub talk_file: String,
    pub talk_swap_data: TalkSwapData,
}
impl TextSwapper {
    pub fn init() -> Self {
        Self {
            current_talk_lines: vec![],
            label_swaps: vec![],
            original_data: MessageList::init(),
            talk_file: String::new(),
            talk_swap_data: TalkSwapData::new(),
        }
    }
    pub fn get_chapter_talk(&mut self) {
        if GameUserData::is_encount_map() { return; }
        let cid = GameUserData::get_chapter().cid.to_string();
        let mut mess = GameUserData::get_chapter().mess.to_string();
        if mess == "*" { mess = cid.trim_start_matches("CID_").to_string(); }
        self.get_chapter_data(mess.as_str());
    }
    pub fn change_char_puppet(mess: &str) {
        let sf = Il2CppClass::from_name("App", "Mess").unwrap().get_static_fields_mut::<MessStaticFields>();
        if let Some(file) = sf.event_file_dictionary.get_item(Il2CppString::new(mess)) {
            let entry_count = file.get_text_num();
            for x in 0..entry_count {
                let text_ptr = file.get_text(x);
                let mut message = copy_from_u16_ptr(text_ptr);
                let original_length = message.len();
                if original_length == 0 { continue; }
                if DVCVariables::UnitRecruitment.get_value() != 0 && !mess.contains("RELIANCE") {
                    PIDS.iter().enumerate().for_each(|(i, &x)| { find_and_splice_for_pid(x, i, &mut message); });
                    find_and_splice_for_pid("PID_デモ用_竜石なし_ラファール", 43, &mut message);
                    find_and_splice_for_pid("PID_ジェーデ_兜あり", 42, &mut message);
                }
                if DVCVariables::EmblemRecruitment.get_value() != 0 {
                    if mess.contains("M0") || mess.contains("S001") || mess.contains("S002") {
                        find_and_splice_for_gid("GID_ディミトリ", 20, &mut message);
                        find_and_splice_for_gid("GID_クロード", 21, &mut message);
                        for x in 0..19 { find_and_splice_for_gid(EMBLEM_GIDS[x], x, &mut message); }
                    }
                    else if EMBLEM_PARA.contains(&mess) && mess.contains("S0") {
                        for x in 0..12 { replace_for_custom_gid(x, &mut message); }
                    }
                }
                if message.len() <= original_length {
                    for x in 0..message.len() { unsafe { *text_ptr.add(x) = message[x]; } }
                }
            }
        }
    }
    pub fn get_chapter_data(&mut self, mess: &str) {
        let file = mess.to_string();
        self.talk_swap_data.load(&file);
        self.current_talk_lines.clear();
        self.talk_file = mess.to_string();
        let sf = Il2CppClass::from_name("App", "Mess").unwrap().get_static_fields_mut::<MessStaticFields>();
        if let Some(file) = sf.mess_file_dictionary.get_item(Il2CppString::new(mess)) {
            let entry_count = file.get_text_num();
            for x in 0..entry_count {
                let label = file.get_label(x);
                let label_str = label.to_string();
                let text_ptr = file.get_text(x);
                let mut message = copy_from_u16_ptr(text_ptr);
                let original_length = message.len();
                let mut changed = self.replace_picture(&mut message);
                changed |= self.replace_for_names(&mut message, &label_str);
                /*
                if let Some(wait) = message.windows(2).rposition(|w| w[0] == 14 && w[1] == 4){
                    let str = format!(" {}", label_str.trim_start_matches("MID_")).encode_utf16().collect::<Vec<u16>>();
                    message.splice(wait..wait, str);
                    changed = true;
                }
                if let Some(wait) = message.windows(2).rposition(|w| w[0] == 14 && w[1] == 7){
                    /*
                        w[0] = 14
                        w[1] = 7
                        w[2] = Tag
                        w[3] = size
                        w[4] = Sec
                        w[5] = Sec
                        w[6] = Color
                     */
                    let ty = if message[wait+2] == 1 { 1 } else { 0 };
                    message.drain(wait..wait+5+ty);
                    changed = true;
                }

                 */
                if changed {
                    if message.len() <= original_length {
                        for x in 0..message.len() { unsafe { *text_ptr.add(x) = message[x]; } }
                    }
                    else {
                        self.current_talk_lines.push(message);
                        if let Some(last) = self.current_talk_lines.last() {
                            sf.mess_data_dictionary.set_item(label, last.as_ptr());
                        } 
                    }
                }
            }
        }
    }
    pub fn apply_to_message(&self, message: &mut Vec<u16>, swap_type: &MessSwapType) -> bool {
        let mut args = swap_type.create_tag_arguments(false, 0);
        let check_cap = swap_type.check_capitalization();
        let pos_len =
            match swap_type {
                MessSwapType::UnitName(idx) => { self.original_data.person_list.get(*idx as usize).and_then(|d| d.find_position(message, false)) }
                MessSwapType::EmblemName(idx) => { self.original_data.emblem_list.get(*idx as usize).and_then(|d| d.find_position(message, false)) }
                MessSwapType::HeroAlias(_) => { self.original_data.alias.get(0).and_then(|d| d.find_position(message, false)) }
                MessSwapType::RingName(idx) => { self.original_data.emblem_alias.get(*idx as usize).and_then(|d| d.find_position(message, false)) }
                MessSwapType::EmblemInvocation(idx) =>{ self.original_data.emblem_alias.get(*idx as usize+40).and_then(|d| d.find_position(message, false)) }
                MessSwapType::EmblemAlias(idx) => { self.original_data.emblem_alias.get(*idx as usize+20).and_then(|d| d.find_position(message, false)) }
                MessSwapType::HeroJob => { 
                    self.original_data.hero_jobs.get(1).and_then(|x| x.find_position(message, true))
                }
                MessSwapType::UnitGenderTextSwap { person_idx: _, txt_idx }|MessSwapType::EmblemGenderTextSwap { emblem_idx: _, txt_idx } => {
                    self.original_data.gender.get(*txt_idx as usize).and_then(|x| x.find_position(message))
                        .map(|p|{ 
                            args[6] = p.2 as u16;
                            p
                        })
                }
                MessSwapType::LiberationKind => {
                    self.original_data.item_kinds[0].iter().enumerate()
                        .find_map(|(i, c)| c.find_position(message, true).zip(Some(i)))
                        .map(|(p, i)|{
                            args[4] = i as u16;
                            p
                        })
                }
                MessSwapType::RingBracelet(_) => {
                    self.original_data.gender[17].contains_by_gender(message, 1)
                        .map(|p|{
                            args[5] = p.2 as u16;
                            p
                        })
                }
                MessSwapType::UnitJob(person, txt) => {
                    self.original_data.text[*txt as usize].find_position(message, true)
                }
                _ => { None }
            };
        if let Some((pos, len, _is_upper)) = pos_len {
            let mut new_line_pos = 0;
            for x in 0..len {
                if message[pos+x] == 10 {
                    new_line_pos = x;
                    break;
                }
            }
            if check_cap {
                let upper = char::from_u32(message[pos] as u32).map(|v| v.is_uppercase()).unwrap_or(false);
                args = swap_type.create_tag_arguments(upper, new_line_pos);
            }
            else if new_line_pos > 0 {
                if let Some(last) = args.last_mut() { *last = new_line_pos as u16; }
            }
            let mut string = String::new();
            args.iter().for_each(|arg| {
                string += " ";
                string += arg.to_string().as_str();
            });
            // println!("Replacement Args: {}", string);
            message.splice(pos..pos+len, args);
            true
        }
        else { false }
    }
    pub fn replace_picture(&self, message: &mut Vec<u16>) -> bool {
        let mut init = 0;
        let mut count = 0;
        for x in 0..20 {
            let pic_name = MessDataString::from_str(RING_PICTURE[x]);
            if let Some((pos, len, _)) =  pic_name.find_from(message, false, init) {
                let new_emblem = DVCVariables::get_dvc_emblem_index(x as i32, false);
                let slice =
                if new_emblem < 20 { RING_PICTURE[new_emblem] }
                else { RING_PICTURE[20] }.encode_utf16().collect::<Vec<u16>>();
                init = pos + slice.len() - 1;
                message[pos-1] = 2 * slice.len() as u16;
                message[pos-4] += 2 * (slice.len() - len) as u16;
                message.splice(pos..pos + len, slice);
                count += 1;
            }
        }
        count > 0
    }
    pub fn replace_for_names(&self, message: &mut Vec<u16>, label: &String) -> bool {
        let mut change_count = 0;
        /*
        if let Some(window_pid) = message.windows(5)
            .position(|x| x[0] == 14 && (x[1] == 3 || (x[1] == 5)) )
            .map(|x| x + 4)
        {
            let pid_len = (message[window_pid] >> 1 ) as usize;
            let pid_start = window_pid +1;
            let command_len = message[2];
            if let Ok(pid) = String::from_utf16(&message[pid_start..pid_len+pid_start]) {
                if DVCVariables::UnitRecruitment.get_value() != 0 || DVCVariables::EmblemRecruitment.get_value() != 0 {
                    if let Some(pos) = PIDS.iter()
                        .map(|pid| pid.trim_start_matches("PID_"))
                        .position(|x| pid == x)
                        .or_else(|| if "ジェーデ_兜あり" == pid { Some(16) } else { None })
                    {
                        message.splice(pid_start..pid_start + pid_len, [0, pos as u16]);
                        message[3] = command_len - message[window_pid] + 4;
                        message[2] += 10;
                        message[window_pid] = 4;
                    }
                    else if let Some(pos) = EMBLEM_GIDS.iter()
                        .map(|gid| gid.trim_start_matches("GID_"))
                        .position(|x| pid == x)
                    {
                        message.splice(pid_start..pid_start + pid_len, [1, pos as u16]);
                        message[3] = command_len - message[window_pid] + 4;
                        message[2] += 10;
                        message[window_pid] = 4;
                    }
                }
            }
        }
        */
        if let Some(line) = self.talk_swap_data.try_get_line(label) {
            line.commands.iter().for_each(|line|{ if self.apply_to_message(message, line) { change_count += 1; } });
        }
        if let Some(demo) = self.talk_swap_data.try_get_demo(label) {
            demo.demo_person_swaps.iter().filter(|x| x.check_label(label))
                .for_each(|x|{ if self.process_person_swaps(message, x) { change_count += 1; } });
        }
        while let Some((pos, length, _)) = self.original_data.alias[0].find_position(message, false) {
            let mut new_line_pos = 0;
            for x in 0..length {
                if message[pos+x] == 10 {
                    new_line_pos = x;
                    break;
                }
            }
            message.splice(pos..pos + length, MessSwapType::HeroAlias(false).create_tag_arguments(false, new_line_pos));
            change_count += 1;
        }
        while let Some((pos, length, _)) = self.original_data.alias[41].find_position(message, false) {
            let mut new_line_pos = 0;
            for x in 0..length {
                if message[pos+x] == 10 {
                    new_line_pos = x;
                    break;
                }
            }
            message.splice(pos..pos + length, MessSwapType::HeroAlias(true).create_tag_arguments(false, new_line_pos));
            change_count += 1;
        }
        self.original_data.person_list.iter().enumerate()
            .for_each(|(i, x)| {
                while let Some((pos, length, _)) = x.find_position_for_name(message) {
                    // println!("Unit Name Replace #{}", i);
                    message.splice(pos..pos + length, MessSwapType::UnitName(i as u16).create_tag_arguments(false, 0));
                    change_count += 1;
                    let search_pos = if pos < 2 { 0 } else { pos - 2 };
                    if i < 41 && !self.original_data.gender.is_empty() {
                        for x in 0..3 {
                            if x == 1 { continue; }
                            if let Some((pos_h, len_h, is_upper)) = self.original_data.gender.get(x)
                                .and_then(|x| x.find_from(message, search_pos))
                                .filter(|(pos_h, _, _)| *pos_h < pos)
                            {
                               // println!("Found Honor at {}", pos_h);
                                message.splice(
                                    pos_h..pos_h + len_h,
                                    MessSwapType::UnitGenderTextSwap { person_idx: i as u16, txt_idx: x as u16 }.create_tag_arguments(is_upper, 0)
                                );
                                change_count += 1;
                                break;
                            }
                        }
                    }
                }
            });
        self.original_data.emblem_list.iter().enumerate().for_each(|(txt_idx, x)| {
            while let Some((pos, length, _)) = x.find_position_for_name(message) {
                message.splice(pos..pos + length, [14, 6, 200+txt_idx as u16, 0]);
               // println!("Emblem Name Replace #{}", txt_idx);
                change_count += 1;
            }
        });
        for x in 0..20 {
            for y in 0..4 {
                let index = (x + y * 20) as usize;
                if let Some((pos, len, _)) = self.original_data.emblem_alias[index].find_position(message, false){
                    // let mut new_line_pos = 0;
                    for x in 0..len {
                        if message[pos+x] == 10 {
                            // new_line_pos = x;
                            break;
                        }
                    }
                    message.splice(pos..pos + len, [14, 6, 300+(index as u16), 0]);
                    change_count += 1;
                    // println!("{} Replace #{}", EMBLEM_REPLACE_KIND[y as usize], x);
                }
            }
        }
        self.talk_swap_data.g_person_swap.iter()
            .filter(|x| x.check_label(label))
            .for_each(|x|{ if self.process_person_swaps(message, x) { change_count += 1; } });

        change_count > 0
    }
    pub fn process_person_swaps(&self, message: &mut Vec<u16>, data: &PersonTalkSwapData) -> bool {
        let txt_idx = data.gender_swap_idx as u16;
        let person_idx = data.person_index as u16;
        if let Some((pos_h, len_h, is_upper)) = self.original_data.gender
            .get(txt_idx as usize)
            .and_then(|x2| x2.contains_by_gender(message, data.gender)) 
        {
            let ty = if data.emblem {
                if txt_idx == 17 { MessSwapType::RingBracelet(person_idx) }
                else { MessSwapType::EmblemGenderTextSwap{ emblem_idx: person_idx, txt_idx } }
            }
            else { MessSwapType::UnitGenderTextSwap { person_idx, txt_idx } };

            message.splice(pos_h..pos_h + len_h, ty.create_tag_arguments(is_upper, 0));
            true
        }
        else { false }
    }
}
pub fn copy_from_u16_ptr(ptr: *const u16) -> Vec<u16> {
    let mut out = vec![];
    let mut count = 0;
    unsafe {
        loop {
            let c = *ptr.add(count);
            out.push(c);
            count += 1;
            if c == 14 {
                let tag = *ptr.add(count);  count += 1;
                out.push(tag);
                let tag_group = *ptr.add(count);    count += 1;
                out.push(tag_group);
                let command_lenth = *ptr.add(count);count += 1;
                out.push(command_lenth );
                if command_lenth > 0 {
                    let commad_size = command_lenth >> 1;
                    for _ in 0..commad_size {
                        let c = *ptr.add(count);    count += 1;
                        out.push(c);
                    }
                }
            } else if c == 0 {
                return out;
            }
        }
    }
}
fn find_and_splice_for_pid(pid: &str, index: usize, message: &mut Vec<u16>) {
    let pid_slice = pid.encode_utf16().collect::<Vec<u16>>();
    if message.len() < pid_slice.len() { return; }
    let recruitment_index =
    match index {
        0..41 => { index }
        41|42 => { 16 }
        43 => { 37 }
        50..60 => { 32 }
        _ => { return; }
    };
    if DVCVariables::get_dvc_recruitment_index(recruitment_index as i32) != recruitment_index as i32 {
        while let Some(pos) = message.windows(pid_slice.len()+1)
            .position(|window| *window.last().unwrap() != 95 && pid_slice.iter().zip(window.iter()).all(|v| *v.0 == *v.1))
        {
            message.splice(pos..pos + pid_slice.len(), [14, 6, 400 + index as u16, 0]);
            if message.len() < pid_slice.len() { return; }
        }
    }
}
fn find_and_splice_for_gid(gid: &str, index: usize, message: &mut Vec<u16>) {
    let pid_slice = gid.encode_utf16().collect::<Vec<u16>>();
    if message.len() < pid_slice.len() { return; }
    let recruitment_index =
    match index {
        20|21 => 12,
        0..20 => index,
        _ => { return; }
    };
    if DVCVariables::get_dvc_emblem_index(recruitment_index as i32, false) != recruitment_index {

        while let Some(pos) = message.windows(pid_slice.len()).position(|window| window == pid_slice) {
            message.splice(pos..pos + pid_slice.len(), [14, 6, 500 + index as u16, 0]);
            if message.len() < pid_slice.len() { return; }
        }
    }
}
fn replace_for_custom_gid(index: usize, message: &mut Vec<u16>) {
    let gid = EMBLEM_GIDS[index];
    let gid_slice = gid.encode_utf16().collect::<Vec<u16>>();
    if message.len() < gid_slice.len() { return; }
    if EmblemPool::get_dvc_emblem_data(EMBLEM_GIDS[index]).filter(|g| EmblemPool::is_custom(g)).is_some(){
        while let Some(pos) = message.windows(gid_slice.len()).position(|window| window == gid_slice) {
            message.splice(pos..pos + gid_slice.len(), [14, 6, 500 + index as u16, 0]);
            if message.len() < gid_slice.len() { return; }
        }
    }
}