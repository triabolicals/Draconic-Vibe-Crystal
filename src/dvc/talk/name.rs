use super::*;
use engage::app::{IUnitMethods, UnitPool};
use unity::{IntPtr, OptionalMethod};
use crate::dvc::talk::window::WindowReplacementType;
use crate::utils::to_opt;

pub fn talk_tag_name(this: TalkTagName, ptr: TalkPtr, _: OptionalMethod) {
    let current_ptr = unsafe { std::mem::transmute::<_, u64>(ptr.get_now_ptr()) };
    let tag_id = ptr.read_int16();
    if tag_id == 16 {
        this.set_m_tag_id(Mess_TagID_Name{value: 0});
        let replace_type = WindowReplacementType::from_window_tag(tag_id);
        let length = ptr.read_int16();
        let i = ptr.read_int16();
        let pid_swap = replace_type.get_swap(i);
        this.set_m_pid(pid_swap.as_str().into());
        if length == 4 {
            let _ = ptr.read_int16();
            let pid = format!("PID_{}", pid_swap);
            if let Some(unit) = to_opt(UnitPool::get_from_person_2(pid.as_str(), false)){ this.set_m_replace_name(unit.get_name()); }
            else { this.set_m_replace_name(Mess::get_game_data_name(pid)); }
        }
        else {
            let spid = ptr.read_string_param();
            if spid.to_rust_string() == "UserName" {
                if let Some(unit) = to_opt(UnitPool::get_hero(false)){ this.set_m_replace_name(unit.get_name()); }
                else { this.set_m_replace_name(Mess::replace_str_default_hero_name()); }
            }
            else { this.set_m_replace_name(Mess::get(spid)); }
        }
    }
    else {
        unsafe {
            ptr.set_now_ptr(std::mem::transmute::<u64, IntPtr>(current_ptr));
            TalkTagName::initialize(this, ptr);
        }
    }
}