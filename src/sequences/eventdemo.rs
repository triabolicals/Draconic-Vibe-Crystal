
use engage::{
    app::{
        eventdemosequence::*, gamesound::*, IRandom_2Methods,
        ISingletonProcInst_1Methods, Random_2, Gender,
        IProcInst, IStructBase, IUnitMethods,
        Mess, Proc, ProcInst, ProcVoidMethod, UnitPool
    },
    combat::{ICharacterAppearance, ICharacterAssetT_1Methods},
    system::{collections::generic::IDictionary_2Methods, IDelegate},
    ProcVoidMethodExt,
    nintendo::message_studio::lib::IBinMsgFileMethods
};
use outfit_core::{get_outfit_data, Randomizer};
use unity::{MethodInfo, prelude::OptionalMethod};
use super::*;
use crate::{
    dvc::{
        asset::SEARCH_LIST,
        mess::{copy_from_u16_ptr, MessageList, puppet::find_and_splice},
        talk::{tag::TextReplacementKind, window::WindowReplacementType}
    },
    enums::{MPIDS, RING_PICTURE},
    gamedata::{EmblemPool, GameData, PersonPool},
    utils::to_opt, DVCFlags,
    enums::{EMBLEM_GIDS, PIDS}
};

// const CREATE_CHARACTER: &'static str = "キャラ配置";
const SET_BACKGROUND: &'static str = "背景";
const SET_SCENE_CAMERA: &'static str = "シーンカメラ";
const SOUND_EVENT: &'static str = "サウンドイベント";
const CHARACTER_SET_ANIMATOR: &'static str = "キャラアニメーター切替";
const PLAY_CHARACTER_MOTION: &'static str = "キャラモーション再生";
const SHOW_PICTURE: &'static str = "一枚絵表示";
const HIDE_PICTURE: &'static str = "一枚絵非表示";
const DIMITRI_JUMPS: &[(&'static str, i32)] = &[
    ("_2_#003", 1), ("_2_#011", 4), ("_2_#016", 1), ("_2_#023", 3), ("_2_#027", 5), ("OP2_#007", 4), ("3_#004", 1)
];
const JUMPS: &[(&'static str, i32)] = &[("_2_#002", 3), ("_2_#010", 100), ("OP2_#005", 100), ("3_#003", 10)];

pub fn event_demo_proc_edit(proc: ProcInst){
    if let Some(proc) = proc.try_cast::<EventDemoSequence>() {
        let descs = proc.m_descs();
        descs.set(5, Proc::call_2(ProcVoidMethod::from_fn(proc, event_demo_sequence_load_load_event_cmd).unwrap()));
    }
}
fn edit_g007() {
    let (found, file) = Mess::s_event_cmd_file_dictionary().try_get_value("G007".into());
    if found && !file.is_null() {
        let entries = file.get_text_num();
        let replace_edelgard = EmblemPool::get_replacement_index_2(12, false).filter(|p| *p != 12).is_some();
        let replacement_dimitri = EmblemPool::get_replacement_index_2(12, false)
            .filter(|p| *p != 12).and_then(|v| if v == 11 { Some(23) } else if v== 18 { Some(22) } else { None });
        let swap = MessageList::get();
        for i in 0..entries {
            let ptr = file.get_text_2(i).as_ptr();
            let mut message: Vec<u16> = copy_from_u16_ptr(ptr);
            let label = file.get_label(i).to_rust_string();
            let mess_ptr = Mess::get_int_ptr(label.as_str()).as_ptr();
            let mut talk_message = copy_from_u16_ptr(mess_ptr);
            let len = message.len();
            let talk_message_len = talk_message.len();
            if len == 0 { continue; }
            swap.replace_with_tags(&mut talk_message, true);
            find_and_splice(PIDS[0], 0, &mut message, TextReplacementKind::DemoPID);
            WindowReplacementType::RecruitmentSwap.try_replace_pid(&mut talk_message, 0, false, None);
            if replace_edelgard {
                WindowReplacementType::RecruitmentSwap.try_replace_pid(&mut talk_message, 12, true, None);
                find_and_splice(EMBLEM_GIDS[12], 12, &mut message, TextReplacementKind::DemoGID);
                if let Some(dimitri_replacement) = replacement_dimitri.as_ref() {
                    if let Some((_, skip)) = DIMITRI_JUMPS.iter().find(|(x, _)| label.ends_with(*x)) {
                        let str = format!("\nSKIP( {}, )\n", *skip);
                        let mut slice = str.encode_utf16().collect::<Vec<u16>>();
                        slice.push(10);
                        slice.push(0);
                        slice.push(0);
                        slice.push(0);
                        unsafe {
                            if slice.len() <= len { for i in 0..slice.len() {ptr.add(i).replace(slice[i]); } }
                        }
                        continue;
                    }
                    else {
                        WindowReplacementType::DirectSwap.try_replace_pid(&mut talk_message, 20, true, Some(*dimitri_replacement as i32));
                        find_and_splice(EMBLEM_GIDS[20], *dimitri_replacement, &mut message, TextReplacementKind::DemoGIDNoSwap);
                        dvc::mess::puppet::remove_line_with_argument(&mut message, EMBLEM_GIDS[21]);
                    }
                }
                else { 
                    dvc::mess::puppet::remove_line_with_argument(&mut message, EMBLEM_GIDS[20]);
                    dvc::mess::puppet::remove_line_with_argument(&mut message, EMBLEM_GIDS[21]);
                    if let Some((_, skip)) = JUMPS.iter().find(|(x, _)| label.ends_with(*x)) {
                        let str = format!("\nSKIP( {}, {} )\n", *skip, *skip);
                        let mut slice = str.encode_utf16().collect::<Vec<u16>>();
                        slice.push(10);
                        slice.push(0);
                        slice.push(0);
                        unsafe {
                            if slice.len() <= len { for i in 0..slice.len() {ptr.add(i).replace(slice[i]); } }
                        }
                        continue;
                    }
                }
            }
            unsafe {
                if talk_message.len() <= talk_message_len { for i in 0..talk_message.len() { mess_ptr.add(i).replace(talk_message[i]); } }
                if message.len() <= len { for i in 0..message.len() {ptr.add(i).replace(message[i]); } }
            }
        }
    }
}
fn replace_event_demo_function(key: &'static str, func: *const u8){
    let instance = EventDemoSequence::get_instance();
    let dictionary = instance.m_func_dictionary();
    let (found, fun) = dictionary.try_get_value(key.into());
    if !fun.is_null() && found { fun.set_method_ptr(unsafe { std::mem::transmute(func) }); }
}
fn add_event_demo_function(key: &'static str, method_info: &'static MethodInfo){
    let instance = EventDemoSequence::get_instance();
    let dictionary = instance.m_func_dictionary();
    dictionary.add(key.into(), EventDemoSequence_CmdFunc::new(instance.into(),  method_info.into()));
}
fn set_character_animator(this: EventDemoSequence, cmd_info: EventDemoSequence_CmdInfo, _: OptionalMethod) -> EventDemoSequence_EventCmdResult {
    let args = cmd_info.get_args();
    if args.len() < 2 {     return this.func_character_set_animator(cmd_info); }
    let arg2 = args.get(1).to_rust_string();
    if let Some((path, anim)) = arg2.rsplit_once('/'){
        let db = get_outfit_data();
        let mut anim_gender = Gender::none();
        if anim.starts_with("AOC") {
            if anim.ends_with("F") { anim_gender = Gender::female(); }
            else if anim.ends_with("M") { anim_gender = Gender::male(); }
        }
        else if anim.starts_with("UAS_") {
            if anim.ends_with("F") { anim_gender = Gender::female(); }
            else if anim.ends_with("M"){ anim_gender = Gender::male(); }
        }
        if anim_gender != Gender::none() {
            let dress_gender = to_opt(this.get_character_appearance(args.get(0), true))
                .and_then(|app| to_opt(app.assets().get(1)))
                .and_then(|a| to_opt(a.get_name()))
                .map(|dress| db.get_dress_gender(dress))
                .unwrap_or_else(|| Gender::none());
            if dress_gender == Gender::none() { return this.func_character_set_animator(cmd_info); }
            if anim.contains("UAS") {
                let mut changed = false;
                if dress_gender != anim_gender {
                    if anim == "UAS_Sdk0AM" {
                        args.set(1, format!("{}/UAS_Sds0AF", path).into());
                        changed |= true;
                    }
                    else if anim == "UAS_Sdp0AF" {
                        args.set(1, format!("{}/UAS_Sdk0AM", path).into());
                        changed |= true;
                    }
                }
                if !changed {
                    if let Some(unit) = to_opt(UnitPool::get_from_person_2(args.get(0), false)).filter(|v| v.get_move_type().value == 1) {
                        let hash = unit.get_job().hash();
                        if let Some(anim) = db.anims.job_anims.iter().find(|j| j.hash == hash && dress_gender == j.gender && j.mode_1.is_some()).and_then(|j| j.mode_1.as_ref()) {
                            args.set(1, format!("{}/{}", path, anim).into());
                            changed |= true;
                        }
                    }
                }
                if !changed && dress_gender != anim_gender {
                    let mut new_uas = anim.to_string();
                    if dress_gender == Gender::female() { new_uas.replace_last("M", "F"); } else { new_uas.replace_last("F", "M"); }
                    if db.anims.uas.contains(&new_uas) { args.set(1, format!("{}/{}", path, new_uas).into()); }
                }
            }
        }
    }
    this.func_character_set_animator(cmd_info)
}
#[unity::callback]
extern "C" fn jump_to_number(this: EventDemoSequence, cmd_info: EventDemoSequence_CmdInfo, method_info: OptionalMethod) -> EventDemoSequence_EventCmdResult {
    let arg = cmd_info.get_args().get(0);
    let arg_str = arg.to_rust_string();
    if let Some(number) = arg_str.parse::<i32>().ok() {
        let current_mess_index = this.m_cur_mess_label_index();
        this.set_m_cur_mess_label_index(current_mess_index + number);
        this.set_m_cur_cmd_info_index(0);
        this.set_m_event_cmd_seq(EventDemoSequence_EventCmdSeq::before_talk());
        return EventDemoSequence_EventCmdResult::retry();
    }
    EventDemoSequence_EventCmdResult::r#continue()
}

pub fn event_demo_function_edit() {
    replace_event_demo_function(SET_BACKGROUND, set_random_background as _);
    replace_event_demo_function(SOUND_EVENT, sound_event as _);
    replace_event_demo_function(SHOW_PICTURE, event_demo_show_picture as _);
    replace_event_demo_function(HIDE_PICTURE, event_demo_hide_picture as _);
    replace_event_demo_function(PLAY_CHARACTER_MOTION, set_motion as _);
    replace_event_demo_function(CHARACTER_SET_ANIMATOR, set_character_animator as _);
    add_event_demo_function("SKIP", jump_to_number_method_info());
}
fn event_demo_show_picture(this: EventDemoSequence, cmd_info: EventDemoSequence_CmdInfo, _: OptionalMethod) -> EventDemoSequence_EventCmdResult {
    let args = cmd_info.get_args();
    let text = args.get(0).to_rust_string();
    if let Some(pic) = RING_PICTURE.iter().position(|r| *r == text)
        .and_then(|v| EmblemPool::get_replacement_index_2(v as i32, false))
        .and_then(|v|RING_PICTURE.get(v as usize))
    {
        args.set(0, (*pic).into());
    }
    this.func_picture_show(cmd_info)
}
fn event_demo_hide_picture(this: EventDemoSequence, cmd_info: EventDemoSequence_CmdInfo, _: OptionalMethod) -> EventDemoSequence_EventCmdResult {
    let args = cmd_info.get_args();
    let text = args.get(0).to_rust_string();
    if let Some(pic) = RING_PICTURE.iter().position(|r| *r == text)
        .and_then(|v| EmblemPool::get_replacement_index_2(v as i32, false))
        .and_then(|v|RING_PICTURE.get(v as usize))
    {
        args.set(0, (*pic).into());
    }
    this.func_picture_hide(cmd_info)
}
fn set_random_background(this: EventDemoSequence, cmd_info: EventDemoSequence_CmdInfo, _: OptionalMethod) -> EventDemoSequence_EventCmdResult {
    let args = cmd_info.get_args();
    if args.len() >= 1 && DVCFlags::CutsceneBackground.get_value() {
        if let Some(data) = SEARCH_LIST.get().and_then(|s| s.map_events.get_random_element(Random_2::get_system())){
            args.set(0, data.as_str().into());
        }
    }
    this.func_set_back_ground(cmd_info)
}
fn set_motion(this: EventDemoSequence, cmd_info: EventDemoSequence_CmdInfo, _: OptionalMethod) -> EventDemoSequence_EventCmdResult {
    let args = cmd_info.get_args();
    let rng = Random_2::get_system();
    if args.len() >= 3 {
        if DVCFlags::CutsceneMotion.get_value() {
            let select = rng.get_value_2(47);
            args.set(2, outfit_core::BODY_EXPRESS[select as usize].into());
        }
        if DVCFlags::CutsceneFacial.get_value() {
            let facial = rng.get_value_2(13);
            args.set(1, outfit_core::FACIAL_STATES[facial as usize].0.into());
        }
    }
    this.func_character_play_motion(cmd_info)
}
fn sound_event(this: EventDemoSequence, cmd_info: EventDemoSequence_CmdInfo, _: OptionalMethod) -> EventDemoSequence_EventCmdResult {
    let args = cmd_info.get_args();
    if args.len() >= 1 && DVCFlags::CutsceneBGM.get_value()  {
        if args.get(0).to_rust_string().contains("BGM"){
            GameSound::stop_all_bgm(GameSound_FadeSpeedType::normal());
            if let Some(bgm) = GameData::get().bgm.get_random_element(Random_2::get_system()) {
                args.set(0, bgm.as_str().into());
            }
        }
    }
    this.func_sound_event(cmd_info)
}

extern "C" fn event_demo_sequence_load_load_event_cmd(this: EventDemoSequence, _method_info: OptionalMethod) {
    let mess = this.m_mess_file_name().to_rust_string();
    let demo_name = this.m_demo_name().to_rust_string();
    event_demo_function_edit();
    if mess.contains("G007"){
        Mess::load(mess.as_str());
        edit_g007();
        let mut counter = 1;
        loop {
            let label = format!("{}_#{:03}", demo_name, counter);
            if !Mess::is_exist(label.as_str()) { return; }
            let (_found, before, after) = Mess::get_event_cmd_text_2(label.as_str());
            this.m_event_cmd_text_exec_before().add(label.as_str().into(), before);
            this.m_event_cmd_text_exec_after().add(label.as_str().into(), after);
            counter += 1;
        }
    }
    if mess.starts_with("R_") && mess.split('_').count() == 3 && (DVCFlags::Supports.get_value() ||  !Mess::is_file_exist(mess.as_str())){
        let demo_name = this.m_demo_name().to_rust_string();
        let new =
            if demo_name.contains("RELIANCE_S") { format!("R_Lueur_{}", MPIDS[Random_2::get_system().get_value_2(40) as usize + 1].trim_start_matches("MPID_")) }
            else { dvc::asset::other::OtherAssetData::get().reliance.get_random_element(Random_2::get_system()).unwrap().clone() };

        this.set_m_mess_file_name(new.as_str().into());

        let person_pool = PersonPool::get();
        let mpids = mess.split("_").collect::<Vec<&str>>();
        let mpids2 = new.split("_").collect::<Vec<&str>>();
        let unit_a = person_pool.pids.iter().position(|v| v.1 == mpids[1]);
        let unit_b =  person_pool.pids.iter().position(|v| v.1 == mpids[2]);
        let unit_c = person_pool.pids.iter().position(|v| v.1 == mpids2[1]);
        let unit_d = person_pool.pids.iter().position(|v| v.1 == mpids2[2]);
        let ab = unit_a.zip(unit_b);
        let cd = unit_c.zip(unit_d);
        if let Some(((a, b), (c, d))) = ab.zip(cd) {
            Mess::load(new.as_str());
            let (a1, a2) = if Random_2::get_system().get_value() & 1 == 1  { (a, b) } else { (b, a) };
            let pid_c = person_pool.pids[c].0.as_str();
            let pid_d = person_pool.pids[d].0.as_str();
            dvc::mess::puppet::edit_demo_mess_for_reliance(new.as_str().into(), pid_c, pid_d, a1 as i32, a2 as i32);
            dvc::mess::puppet::edit_reliance_talk_character(new.as_str().into(),c as i32, d as i32, a1 as i32, a2 as i32);
            let mut counter = 1;
            loop {
                let label = format!("{}_#{:03}", demo_name, counter);
                if !Mess::is_exist(label.as_str()) { break; }
                let (_found, before, after) = Mess::get_event_cmd_text_2(label.as_str());
                this.m_event_cmd_text_exec_before().add(label.as_str().into(), before);
                this.m_event_cmd_text_exec_after().add(label.as_str().into(), after);
                counter += 1;
            }
            return;
        }
    }
    this.load_event_cmd();
}
