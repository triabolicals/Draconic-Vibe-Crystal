use engage::app::Mess;
use crate::{
    dvc::mess::MessageList,
    dvc::talk::window::WindowReplacementType
};

pub fn edit_label_all_pid(label: &str, replacement_type: WindowReplacementType, emblem: bool) {
    if !Mess::is_exist(label) { return; }
    let int_ptr = Mess::get_int_ptr(label);
    let limit = if !emblem { 41 } else { 19 }; 
    if !int_ptr.is_null() {
        let ptr = int_ptr.as_ptr();
        let mut str = crate::dvc::mess::copy_from_u16_ptr(ptr);
        let length = str.len();
        for i in 0..limit{ replacement_type.try_replace_pid(&mut str, i, emblem, None); }
        if str.len() <= length {
            for i in 0..str.len() { unsafe { ptr.add(i).replace(str[i]); } }
        }
    }
}
pub fn edit_label_pid(label: &str, replacement_type: WindowReplacementType, pid_index: i32){
    if !Mess::is_exist(label) { return; }
    let int_ptr = Mess::get_int_ptr(label);
    if !int_ptr.is_null() {
        let ptr = int_ptr.as_ptr();
        let mut str = crate::dvc::mess::copy_from_u16_ptr(ptr);
        let length = str.len();
        replacement_type.try_replace_pid(&mut str, pid_index, false, None);
        MessageList::get().replace_with_tags(&mut str, false);
        if str.len() <= length { for i in 0..str.len() { unsafe { ptr.add(i).replace(str[i]); } } }
    }
}

pub fn replace_for_recruitment_demo_name(demo_name: &str, emblem: bool){
    let mut counter = 1;
    loop {
        let label = format!("{}_#{:03}", demo_name, counter);
        if !Mess::is_exist(label.as_str()) { break; }
        edit_label_all_pid(label.as_str(), WindowReplacementType::RecruitmentSwap, emblem);
        crate::dvc::mess::puppet::edit_all_pid_demo_mess_label(label.as_str());
        counter += 1;
    }
}