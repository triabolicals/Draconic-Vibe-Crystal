pub use engage::app::{mess::*, talk3_d::{talksequence::*, talkptr::*, talktagname::*}};
pub use crate::dvc::mess::*;
use engage::{
    app::{
        godunit::*, goddata::*, unit::*,
        talk3_d::{
            talktagaddletter::*, talktag::*, 
            talktaganimation::*, talktagparser::*, talktagpicture::*
        },
        ISingletonProcInst_1Methods, GameSound, ITalkSound, IUnitEdit, UnitPool
    },
    system::collections::generic::{IDictionary_2Methods},
};
use unity::{Cast, FromIlInstance, IntPtr, MethodInfo, OptionalMethod};
use crate::{
    DVCVariable,
    dvc::talk::tag::TalkTagTextReplace,
    enums::{EMBLEM_GIDS, PIDS, RING_PICTURE},
    gamedata::{EmblemPool, PersonPool},
    utils::{il2str, override_vtable},
};
use crate::dvc::talk::name::talk_tag_name;
use crate::dvc::talk::window::WindowReplacementType;

pub mod tag;
mod addletter;
pub mod window;
mod name;
pub static mut MID_BUFFER: [u16; 512] = [0; 512];

pub fn talk_tag_vtable_edit() {
    override_vtable("App.Talk3D", "TalkTagAnimation", "Initialize", talk_tag_anim as _);
    override_vtable("App.Talk3D", "TalkTagWindow", "Initialize", window::talk_tag_window as _);
    override_vtable("App.Talk3D", "TalkTagWindow", "Execute", window::talk_tag_window_execute as _);
    override_vtable("App.Talk3D", "TalkTagPicture", "Initialize", talk_tag_picture as _);
    override_vtable("App.Talk3D", "TalkTagName", "Initialize", talk_tag_name as _);
}
pub fn lueur_talk_voice(talk: TalkSequence, female: bool) {
    if let Some(voice) = il2str(talk.m_talk_sound().m_reserved_talk_voice()) {
        if female && !voice.ends_with("_F") {
            let female_voice = voice + "_F";
            if GameSound::is_event_loaded(female_voice.as_str()) {
                talk.m_talk_sound().set_m_reserved_talk_voice(female_voice.into());
            }
        }
        else if !female && voice.ends_with("_F") {
            let male_voice = voice.trim_end_matches("_F");
            if GameSound::is_event_loaded(male_voice) { talk.m_talk_sound().set_m_reserved_talk_voice(male_voice.into()); }
        }
    }
}
fn lueur_talk_gender_edit(talk: TalkSequence, talk_ptr: TalkPtr, is_female: bool) {
    lueur_talk_voice(talk, is_female);
    let mut mid = talk.get_mid().to_rust_string();
    let mid_f = mid.ends_with("_F");
    if is_female && !mid_f {
        mid += "_F";
        if Mess::is_exist(mid.as_str()) {
            let int_ptr = Mess::get_int_ptr(mid.as_str());
            talk_ptr.ctor(int_ptr);
        }
    }
    else if !is_female && mid_f {
        mid = mid.trim_end_matches("_F").to_string();
        if Mess::is_exist(mid.as_str()) {
            let int_ptr = Mess::get_int_ptr(mid.as_str());
            talk_ptr.ctor(int_ptr);
        }
    }
}
fn edit_talk_gender(talk_ptr: TalkPtr) {
    let now = unsafe { std::mem::transmute::<_, u64>(talk_ptr.get_now_ptr()) };
    let original = unsafe { std::mem::transmute::<_, u64>(talk_ptr.get_original_ptr()) };
    if now == original {
        if let Some(talk) = crate::utils::to_opt(TalkSequence::get_instance()) {
            let mid = talk.get_mid().to_rust_string();
            if mid.contains("Lueur") || mid.contains("LUEUR") {
                lueur_talk_gender_edit(talk, talk_ptr, DVCVariable::LueurGender.get_value() == 2);
            }
            else if !mid.contains("RELIANCE") && !mid.contains("GR_") && !mid.contains("_Die") {
                if let Some(female) = crate::utils::to_opt(UnitPool::get_hero(false)).map(|u| u.m_edit().m_gender().value == 2){
                    lueur_talk_gender_edit(talk, talk_ptr, female);
                }
            }
        }
    }
}
#[unity::hook("App.Talk3D", "TalkTagParser", "Parse")]
pub fn talk_tag_parser_parse(this: TalkTagParser, talk_ptr: TalkPtr, method_info: MethodInfo) -> TalkTag{
    edit_talk_gender(talk_ptr);
    let char = talk_ptr.read_char();
    if char != 15 {
        if char == 14 {
            let tag_group = talk_ptr.read_tag_group();
            let (found, tag) = this.m_tag_dictionary().try_get_value(tag_group);
            if found {
                tag.initialize(talk_ptr);
                return tag;
            }
            else if tag_group.value >= 15 && tag_group.value < 50 {
                let replace_tag = TalkTagTextReplace::instantiate().unwrap();
                replace_tag.set_replacement_from_tag(tag_group.value as u16);
                replace_tag.initialize(talk_ptr);
                return unsafe { replace_tag.cast() };
            }
            else if tag_group.value >= 50 && tag_group.value < 100 {
                let char = talk_ptr.read_char();
                let character = char & 255;
                let replace = char >> 8;
                let mut mid = ReplacementMessData::get_random_mid(replace as i32, tag_group.value - 50);
                if !Mess::is_exist(mid.as_str()){ mid = format!("{}_#001", mid); }
                let int_ptr = Mess::get_int_ptr(mid.as_str());
                let mut v = copy_from_u16_ptr(int_ptr.as_ptr());
                WindowReplacementType::DirectSwap.try_replace_pid(&mut v, replace as i32, false, Some(character as i32));
                unsafe {
                    MID_BUFFER.iter_mut().for_each(|x| *x = 0);
                    for i in 0..v.len(){ MID_BUFFER[i] = v[i]; }
                    let ptr = MID_BUFFER.as_ptr();
                    let int_ptr =  std::mem::transmute::<_, IntPtr>(ptr);
                    talk_ptr.ctor(int_ptr);
                }
                return talk_tag_parser_parse(this, talk_ptr, method_info);
            }
        }
        else if char != 0 {
            this.m_tag_add_letter().set_m_add_letter(char);
            return unsafe {this.m_tag_add_letter().cast() };
        }
    }
    unsafe { this.m_tag_finish().cast() }
}

fn get_pid_replacement(index: i32) -> String {
    if index < 41 {
        let index = PersonPool::get_replacement_index(index, false).unwrap_or(index);
        PersonPool::get().pids[index as usize].0.clone()
    }
    else if index >= 100 {
        let recruitment_index = index - 100;
        if recruitment_index == 20 || recruitment_index == 21 {
            if let Some(idx) = EmblemPool::get_replacement_index_2(12, false) {
                if idx == 18 { EMBLEM_GIDS[22].trim_start_matches("GID_").to_string() }
                else if idx == 11 { EMBLEM_GIDS[23].trim_start_matches("GID_").to_string() }
                else if let Some(str) = EmblemPool::get_replacement_god_3(idx, false).map(|g| g.get_gid().to_rust_string()){ str.trim_start_matches("GID_").to_string() }
                else { EMBLEM_GIDS[recruitment_index as usize].trim_start_matches("GID_").to_string() }
            }
            else { EMBLEM_GIDS[recruitment_index as usize].trim_start_matches("GID_").to_string() }
        }
        else {
            if let Some(god_unit) = EmblemPool::get_replacement_god_unit(recruitment_index, false){
                let god = god_unit.get_gid().to_rust_string();
                god.trim_start_matches("GID_").to_string()
            }
            else if let Some(god) = EmblemPool::get_replacement_god_3(recruitment_index, false).map(|g| g.get_gid().to_rust_string()){
                god.trim_start_matches("GID_").to_string()
            }
            else { EmblemPool::GIDS[recruitment_index as usize].to_string() }
        }
    }
    else { PIDS[index as usize].trim_start_matches("PID_").to_string() }
}
fn talk_tag_anim(this: TalkTagAnimation, ptr: TalkPtr, _: OptionalMethod) {
    let tag_id = ptr.read_int16();
    this.set_m_tag_id(TalkTagAnimation_TagID{ value: tag_id & 15});
    let _length = ptr.read_int16();
    let replace_tag = WindowReplacementType::from_window_tag(tag_id);
    if replace_tag != WindowReplacementType::NoReplacement {
        let i = ptr.read_int16();
        let pid = replace_tag.get_swap(i);
        this.set_m_pid(pid.into());
    }
    else {
        let pid = ptr.read_pid();
        this.set_m_pid(pid);
    }
    let anim_name = ptr.read_string_param();
    this.set_m_anime_name(anim_name);
}
fn talk_tag_picture(this: TalkTagPicture, ptr: TalkPtr, _: OptionalMethod){
    unsafe { TalkTagPicture::initialize(this, ptr) };
    if let Some(new_ring_picture) =
        il2str(this.m_texture_name()).and_then(|v| RING_PICTURE.iter().position(|&x| x == v).filter(|v| *v < 19))
        .and_then(|v| EmblemPool::get_replacement_index_2(v as i32, false)).map(|v| if v < 19 { RING_PICTURE[v as usize] } else { RING_PICTURE[20] })
    {
        this.set_m_texture_name(new_ring_picture.into());
    }
}