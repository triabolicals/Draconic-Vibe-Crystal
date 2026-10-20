use unity::{Cast, Il2CppString};
use engage::{
    app::{
        mapmind::*,
        ISingletonClass_1Methods, IUnitMethods, ISingletonProcInst_1Methods,
    talk3_d::{ITalkPtrMethods, ITalkSequenceMethods, ITalkTagWindow, TalkCharacterManager, TalkPtr, TalkSequence, TalkTagWindow, TalkTagWindow_TagID},
    HubSequence,
    IHubAccessMethods, IHubPlayerControllerMethods, IHubSequenceMethods, }
};
use unity::OptionalMethod;
use crate::{
    dvc::asset::AppearanceManager, dvc::talk::get_pid_replacement, DVCFlags::GodNames, enums::EMBLEM_GIDS,
    gamedata::PersonPool, utils::to_opt, dvc::mess::MessDataPosition, enums::PIDS, utils::il2str
};
use crate::dvc::mess::copy_from_u16_ptr;
use crate::enums::MPIDS;
use crate::gamedata::EmblemPool;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowReplacementType {
    NoReplacement = 0,
    RecruitmentSwap = 1,
    MapMindSwap = 2,
    DirectSwap = 3,
    HeroSwap = 4,
    HubAccessSwap = 5,
}
impl WindowReplacementType {
    pub fn from_window_tag(tag_id: i32) -> Self {
        let v = tag_id >> 4;
        match v {
            1 => WindowReplacementType::RecruitmentSwap,
            2 => WindowReplacementType::MapMindSwap,
            3 => WindowReplacementType::DirectSwap,
            4 => WindowReplacementType::HeroSwap,
            5 => WindowReplacementType::HubAccessSwap,
            _ => WindowReplacementType::NoReplacement,
        }
    }
    pub fn edit_label_by_string(self, label: impl Into<Il2CppString>, search: &String, replace_index: u16) -> bool {
        let int_ptr = engage::app::Mess::get_int_ptr(label);
        if !int_ptr.is_null() {
            let ptr = int_ptr.as_ptr();
            let mut str = copy_from_u16_ptr(ptr);
            let len = str.len();
            self.replace_with_tag(&mut str, search, replace_index, None);
            if str.len() <= len { unsafe {  str.into_iter().enumerate().for_each(|(i, c)|{ ptr.add(i).replace(c); }) }; }
            true
        }
        else { false }
    }
    pub fn to_mess_tag(self, window_tag_id: u16) -> u16 { window_tag_id | ((self as u16) << 4) }
    pub fn replace_with_tag(self, message: &mut Vec<u16>, search: &String, replace_index: u16, start: Option<usize>){
        let mut start = start.unwrap_or(0);
        while let Some(pos) = MessDataPosition::get(message, &search, false, Some(start)) {
            start = pos.position + 1;
            if Self::position_match(message, &pos) {
                let tag = message[pos.position - 3];
                let command_length = message[pos.position - 2] - message[pos.position - 1];
                message.splice(pos.position - 3..pos.position + pos.length, [self.to_mess_tag(tag), command_length, replace_index]);
            }
        }
    }
    pub fn try_alias_swap(self, message: &mut Vec<u16>){
        if let Some(pos) = message.windows(6).position(|w| w[0] == 14 && w[1] == 6 && w[2] == 16 && w[4] < 41){
            let mpid = MPIDS[message[4+pos] as usize].to_string();
            if let Some(search) = MessDataPosition::get(message, &mpid, false, Some(pos+5)) {
                message.splice(search.position - 1.. search.position+search.length, [0]);
                message[3+pos] = 4;
            }
        }
    }
    pub fn try_replace_pid(self, message: &mut Vec<u16>, pid: i32, emblem: bool, replace_index: Option<i32>){
        if !message.windows(3).any(|w| w[0] == 14 && (w[1] == 3 || w[1] == 5 || w[1] == 6) && w[2] < 10) { return; }
        let search = if emblem { EmblemPool::GIDS[pid as usize].to_string() } else { PersonPool::PIDS[pid as usize].trim_start_matches("PID_").to_string() };
        let replace_index = replace_index.unwrap_or(pid) as u16 + if emblem { 100 } else { 0 };
        let start = 5;
        self.replace_with_tag(message, &search, replace_index, Some(start));
        if pid == 16 {
            let alt_jade = "ジェーデ_兜あり".to_string();
            self.replace_with_tag(message, &alt_jade, replace_index, Some(start));
        }
        if replace_index < 41 { self.try_alias_swap(message); }

    }
    fn position_match(message: &mut Vec<u16>, position: &MessDataPosition) -> bool {
        if position.position < 5 { false }
        else {
            /*
                Pos - 5 = 14
                Pos - 4 = Tag Type of Window (3) / Anim (5) / Name (6)
                Pos - 3 = Tag Group
                Pos - 2 = Command Length
                Pos - 1 = String Length
             */
            let tag_type = message[position.position - 4];
            let group = message[position.position - 3];
            let tag_group = (tag_type == 6 && group == 0) || ((tag_type == 3 ||  tag_type == 5) && group <= 10);
            let length_match = (message[position.position - 1] >> 1) == (position.length as u16);
            message[position.position - 5] == 14 && tag_group && length_match
        }
    }
    pub fn get_swap(self, i: i32) -> String {
        let talk_sequence = TalkSequence::get_instance();
        let mut person =
        if i < 100 { PersonPool::get().pids[i as usize].0.trim_start_matches("PID_").to_string() }
        else {
            let i = i - 100;
            EmblemPool::GIDS[i as usize].to_string()
        };
        match self {
            WindowReplacementType::RecruitmentSwap => { person = get_pid_replacement(i).trim_start_matches("PID_").to_string() }
            WindowReplacementType::MapMindSwap => {
                if to_opt(MapMind::get_instance()).is_some_and(|v| !v.get_unit().is_null()) && !talk_sequence.is_null() && !talk_sequence.is_null() {
                    let unit = MapMind::get_instance().get_unit();
                    let pid = unit.get_pid().to_rust_string();
                    person = pid.trim_start_matches("PID_").to_string();
                }
            }
            WindowReplacementType::HeroSwap => {
                if PersonPool::get_replacement_index(0, true).is_some_and(|h| h == i){
                    person = PIDS[0].trim_start_matches("PID_").to_string();
                }
            }
            WindowReplacementType::HubAccessSwap => {
                if let Some(hub_access) = to_opt(HubSequence::get_instance()).and_then(|hub| to_opt(hub.get_player()))
                    .and_then(|controller| to_opt(controller.try_get_last_access()))
                    .and_then(|access| il2str(access.try_get_pid()))
                {
                    person = hub_access.trim_start_matches("PID_").to_string();
                }
            }
            _ => {}
        }
        person
    }
}
pub fn talk_tag_window(this: TalkTagWindow, ptr: TalkPtr, _: OptionalMethod) {
    let tag_id = ptr.read_int16();
    // #[cfg(feature = "debug")] println!("TalkTagWindow: {}", tag_id);
    let _command_length = ptr.read_int16();
    let talk_sequence = TalkSequence::get_instance();
    let tag = tag_id & 15;
    let replace_type = WindowReplacementType::from_window_tag(tag_id);
    this.set_m_tag_id(TalkTagWindow_TagID { value: tag});
    if replace_type == WindowReplacementType::NoReplacement {
        let pid = ptr.read_pid();
        let pid_to_create = TalkCharacterManager::pid_to_pid_for_create(pid);
        this.set_m_pid(pid);
        this.set_m_pid_for_create(pid_to_create);
        if !pid.is_null() && !talk_sequence.is_null() {
            let pid_str = pid.to_rust_string();
            if GodNames.get_value() {
                if let Some(pos) = EmblemPool::GIDS.iter().position(|gid| *gid == pid_str) {
                    let name = AppearanceManager::get_emblem_appearance_name(pos as i32);
                    if !name.is_null() { talk_sequence.add_replace_talker_name(EMBLEM_GIDS[pos], name); }
                }
            }
        }
    }
    else {
        let i = ptr.read_int16();
        // #[cfg(feature = "debug")] println!("TAG: {}, INDEX: {}", this.m_tag_id().value, i);
        let pid_swap = replace_type.get_swap(i);
        this.set_m_pid(pid_swap.as_str().into());
        this.set_m_pid_for_create(this.m_pid());
        if replace_type == WindowReplacementType::RecruitmentSwap {
            if i >= 100 && GodNames.get_value(){
                if let Some(pos) = EmblemPool::GIDS.iter().position(|v| pid_swap.contains(*v)) {
                    let name = AppearanceManager::get_emblem_appearance_name(pos as i32);
                    if !name.is_null() { TalkSequence::get_instance().add_replace_talker_name(pid_swap.as_str(), name); }
                }
            }
        }
        else { this.set_m_pid_for_create(TalkCharacterManager::pid_to_pid_for_create(this.m_pid())); }
    }
    if tag == 0 || tag == 3 {
        let string_parm = ptr.read_string_param().to_rust_string();
        if string_parm == "t" { this.set_m_location_name("Top".into()); }
        else if string_parm == "b" { this.set_m_location_name("Bottom".into()); }
        else { this.set_m_location_name(string_parm.into()); }
    }
}
pub fn talk_tag_window_execute(this: TalkTagWindow, _: OptionalMethod) {
    unsafe { TalkTagWindow::execute(this); }
    if this.m_tag_id().value == 0 || this.m_tag_id().value == 3 {
        let talk = TalkSequence::get_instance();
        if talk.is_null() { return; }
        let pid = this.m_pid().to_rust_string();
        if let Some(s) = EmblemPool::GIDS.iter().position(|v| *v == pid).filter(|&v| v != 19){
            if GodNames.get_value() {
                let name = AppearanceManager::get_emblem_appearance_name(s as i32);
                if !name.is_null() {
                    TalkSequence::get_instance().add_replace_talker_name(pid.as_str(), name);
                }
            }
        }
    }
}