use std::sync::RwLock;
use engage::{
    gamevariable::GameVariableManager, mess::Mess, sequence::talk::*, tmpro::TextMeshProUGUI,
    language::{Language, LanguageLangs::*},
};
use unity::prelude::*;
use crate::{
    config::DVCFlags, DVCVariables, enums::{EMBLEM_GIDS, RINGS}, message::{TextSwapper, MESSAGE_SWAPPER},
    randomizer::{data::EmblemPool, names::AppearanceRandomizer, item, RANDOMIZED_DATA}
};

#[unity::class("App.Talk3D", "TalkUI")]
pub struct TalkUI {
    junk: [u8; 0x28],
    pub focus_object: &'static mut TalkObject,
}
#[repr(C)]
pub struct TalkObject {
    pub klass: &'static Il2CppClass,
    monitor: *const u8,
    junk: [u8; 0x28],
    pub main_text: &'static mut TextMeshProUGUI,
}


#[unity::hook("App", "Mess", "Load")]
pub fn mess_load(filename: &Il2CppString, method_info: OptionalMethod) -> bool {
    let result = call_original!(filename, method_info);
    if filename.str_contains("M0") || filename.str_contains("S0") || filename.str_contains("G0") || filename.str_contains("E0"){
        if let Some(mut text) = MESSAGE_SWAPPER.get_or_init(|| RwLock::new(TextSwapper::init())).write().ok() {
            text.get_chapter_data(filename.to_string().as_str());
        }
        TextSwapper::change_char_puppet(filename.to_string().as_str());
    }
    result
}
pub fn talk_tag_name_initialize(this: &mut TalkTagName, talk_ptr: &mut TalkPtr, _optional_method: OptionalMethod) {
    let tag = talk_ptr.read_int16();
    // let mid = unsafe { crate::talk::get_current_mid(None) }.to_string();
    // println!("TalkTagName Initialize [{}]: {}", mid, tag);
    if tag >= 10 {
        let length = talk_ptr.read_int16() >> 1;
        let mut args = vec![];
        for _ in 0..length {
            args.push(talk_ptr.read_int16());
        }
        this.tag_id = 3;    // TagID = 3,4,5 will replace text
        this.replacement_name = "".into();
        if let Some(replacement) = get_replacement_name(tag, &args) {
            // println!("Replacement [{}]: {}", tag, replacement);
            this.replacement_name = replacement; 
        }
        else { this.replacement_name = "".into(); }
    }
    else {
        talk_ptr.now_ptr -= 2;
        this.initialize_(talk_ptr);
    }
}
pub fn talk_tag_add_letter_execute_edit(this: &mut TalkTagAddLetter, _optional_method: OptionalMethod) {
    let next_char = this.add_letter;
    if (next_char == 10 && this.is_line_feed_enabled) || next_char == 32 {
        if let Some(talk_ui) = engage::util::try_get_instance_monobehaviour::<TalkUI>() {
            let text = talk_ui.focus_object.main_text.get_text();
            let text_u16 = text.to_u16();
            let current_size = text_u16.len();
            let last_new_line  = text.to_u16().iter().rposition(|x| *x == 10 ).unwrap_or(0);
            if last_new_line < current_size {
                let size = current_size - last_new_line;
                if next_char == 10 {
                    if size < 40 { this.add_letter = 32; }
                }
                else if size > 44 { this.add_letter = 10; }
            }
        }
    }
    this.execute_();
}
pub fn get_replacement_name(tag: u16, args: &Vec<u16>) -> Option<&'static mut Il2CppString>{
    let mut mess = 
    if let Some(text) = MESSAGE_SWAPPER.get_or_init(|| RwLock::new(TextSwapper::init())).read().ok() {
        match tag {
            100..200 => { // Person Name Replacement
                let person = tag as i32 - 100;
                if person < 41 {
                    DVCVariables::get_dvc_unit(person, false).map(|u| u.get_name())
                        .or_else(||DVCVariables::get_dvc_person_data(person, false).map(|p| p.get_name()))
                }
                else {
                    if DVCFlags::RandomBossesNPCs.get_value() {
                        if let Some(v) = RANDOMIZED_DATA.get().and_then(|s| s.read().ok()){
                            v.person_appearance.get_npc_name(person - 41).map(|n| Mess::get(n))
                        }
                        else { DVCVariables::get_dvc_person_data(person, false).map(|p| p.get_name()) }
                    }
                    else { DVCVariables::get_dvc_person_data(person, false).map(|p| p.get_name()) }
                }
            }
            200..224 => { // Emblem Swap
                let index = (tag - 200) as usize;
                if let Some(god) = DVCVariables::get_current_god(index as i32){
                    crate::randomizer::names::get_emblem_person(god.mid).map(|p| p.get_name())
                        .or_else(||Some(Mess::get(god.mid.to_string().replace("MGID", "MPID"))))
                }
                else { Some(Mess::get(format!("MPID_{}", RINGS[index]))) }
            }
            300..380 => { // Ring Name
                let emblem_index = tag % 20;
                let offset = ((tag - 300) / 20) as usize;
                let index = DVCVariables::get_dvc_emblem_index(emblem_index as i32, false);
                if DVCFlags::GodNames.get_value() && offset == 0 {  // Ring Name
                    AppearanceRandomizer::get_emblem_app_person_index(index as i32).map(|v| AppearanceRandomizer::get_alias(v.1))
                }
                else { Some(text.original_data.emblem_alias[index+20*offset].to_str()) }
            }
            400..460 => {
                let recruitment_index = 
                match tag {
                    450..460 => { 32 }
                    442 => { 16 }
                    443 => { 37 }
                    _ => { tag - 400 }
                };
                DVCVariables::get_dvc_person_data(recruitment_index as i32, false)
                    .map(|p| p.pid.to_string().into())
            }
            500..542 => {
                if tag < 522 {
                    let recruitment_index =
                        match tag {
                            520|521 => { 12 }
                            _ => { tag - 500 }
                        };
                    DVCVariables::get_current_god(recruitment_index as i32).map(|g| g.gid.to_string().into())
                }
                else if tag >= 530 {
                    let recruitment_index = tag - 530;
                    if let Some(name) = EmblemPool::get_dvc_emblem_data(EMBLEM_GIDS[recruitment_index as usize])
                        .filter(|g| EmblemPool::is_custom(g))
                    {
                        Some(Mess::get(name.mid))
                    }
                    else if DVCFlags::GodNames.get_value() {
                        AppearanceRandomizer::get_emblem_app_person_index(recruitment_index as i32).map(|v| v.1.get_name())
                    }
                    else { None }
                }
                else { None }
            }
            16 => { // Divine Dragon to New Alias
                let offset = if args[0] == 1 { 41 } else { 0 };
                let person_index = DVCVariables::get_dvc_recruitment_index(0);
                if person_index == -1 { Some(text.original_data.alias[82].to_str()) }
                else { Some(text.original_data.alias[offset+person_index as usize].to_str()) }
            }
            17 => { // Text Swap based on person gender
                let mut text_idx = args[1] as usize;
                let gender = DVCVariables::get_dvc_unit(args[0] as i32, false)
                    .map(|v| v.get_gender() as i32)
                    .or_else(|| DVCVariables::get_dvc_person_data(args[0] as i32, false)
                        .map(|p| p.get_gender())
                    )?;
                if text_idx == 15 {
                    let sibling = if args[0] == 11 { 14 } else { 17 };
                    let gender2 = DVCVariables::get_dvc_unit(sibling, false)
                        .map(|v| v.get_gender() as i32)
                        .unwrap_or(gender);

                    if gender2 != gender { text_idx = 2; }
                }
                text.original_data.gender.get(text_idx ).map(|v| v.get(gender, args[2] != 0))
            }
            18 => { // Text Swap based on emblem gender
                let god = DVCVariables::get_current_god(args[0] as i32)?;
                let gender = god.female + 1;
                text.original_data.gender.get(args[1] as usize).map(|v| v.get(gender, args[2] != 0))
            }
            19 => { // Divine Dragon Job
                DVCVariables::get_dvc_unit(0, false)
                    .and_then(|u|{
                        let job = &u.job;
                        job.get_high_job1().map(|v| Mess::get_name(v)).or_else(|| Some(Mess::get_name(job.jid)))
                    })
            }
            20 => { // Liberation
                if GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE) == 0  { item::change_liberation_type(); }
                let k = GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE);
                let kind = if k == 0 { 0 } else { k - 1 } as usize;
                text.original_data.item_kinds[kind].get(args[0] as usize)
                    .or_else(|| text.original_data.item_kinds[kind].last()).map(|w| w.to_str())
            }
            22 => {
                let r = DVCVariables::get_god_from_index(args[0] as i32, true)
                    .map(|g| g.flag.value & 32).unwrap_or(0);
                text.original_data.gender.get(17).map(|v| v.get(if r & 32 != 0 { 2 } else { 1 }, args[1] != 0))
            }
            24 => {
                DVCVariables::get_dvc_unit(args[0] as i32, false)
                    .map(|u| u.job.jid)
                    .or_else(||
                        DVCVariables::get_dvc_person_data(args[0] as i32, false)
                            .and_then(|p| p.get_job())
                            .map(|j| j.jid)
                    )
                    .map(|j| {
                        let job_name = Mess::get_name(j).to_string().to_lowercase();
                        if args[1] == 1 { apply_article(job_name.as_str(), true).into() }
                        else { job_name.into() }
                    })
            }
            _ => { None }
        }
    }
    else { None };
    // Convert Space into New Line if the replacement text new lines in the middle
    if let Some( (new_line_pos, new_mess))= args.last().filter(|v| **v > 0).map(|v| *v as usize).zip(mess.as_mut()){
        if let Some(space) = new_mess.to_u16_mut().iter_mut().enumerate().find(|(i, v)| *i >= new_line_pos && **v == 32) {
            *space.1 = 10;
        }
        else { return Some(format!("{}\n", new_mess).into()); }
    }
    mess
}
fn apply_article(name: &str, lower: bool) -> String {
    let lang = Language::get_lang();
    if lang == USEnglish || lang == EUEnglish {
        let start = if lower { name.to_lowercase() }
        else { name.to_string() };
        if start.starts_with(|c| c == 'A' || c == 'a' || c == 'e' || c == 'E' || c == 'I' || c == 'i' || c == 'o' || c == 'O') {
            format!("an {}", start)
        }
        else { format!("a {}", start) }
    }
    else { name.to_string() }
}
