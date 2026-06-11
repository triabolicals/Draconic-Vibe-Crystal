use engage::{gamesound::{GameSound, GameSoundFadeSpeedType}, random::Random, sequence::eventdemo::*};
use unity::prelude::OptionalMethod;
use crate::{
    assets::data::SEARCH_LIST, config::DVCFlags, DVCVariables,
    message::RING_PICTURE, randomizer::Randomizer
};

pub fn event_demo_function_edit() {
    if let Some(event) = EventDemoSequence::get_instance() {
        if DVCFlags::CutsceneBackground.get_value() {
            if let Some(set_background) = event.get_func(EventDemoSequenceCmdFunc::SetBackground){ 
                set_background.method_ptr = set_random_background as _; 
            }
        }
        if let Some(bgm) = event.get_func(EventDemoSequenceCmdFunc::SoundEvent){
            bgm.method_ptr = sound_event as _;
        }
        if let Some(show_picture) = event.get_func(EventDemoSequenceCmdFunc::ShowPicture) {
            show_picture.method_ptr = event_demo_show_picture as _;
        }
        if let Some(hide_picture) = event.get_func(EventDemoSequenceCmdFunc::HidePicture) {
            hide_picture.method_ptr = event_demo_hide_picture as _;
        }
        if let Some(char_motion) = event.get_func(EventDemoSequenceCmdFunc::PlayCharacterMotion){
            char_motion.method_ptr = set_motion as _;
        }
    }
}
fn event_demo_show_picture(this: &EventDemoSequence, cmd_info: &mut CmdInfo, _: OptionalMethod) -> EventDemoSequenceEventCmdResult {
    let text = cmd_info.args[0].to_string();
    if let Some(pos) = RING_PICTURE.iter().position(|r| *r == text){
        let new_index = DVCVariables::get_dvc_emblem_index(pos as i32, false);
        if let Some(pos) = RING_PICTURE.get(new_index) { cmd_info.args[0] = pos.into(); }
    }
    this.func_picture_show(cmd_info)
}
fn event_demo_hide_picture(this: &EventDemoSequence, cmd_info: &mut CmdInfo) -> EventDemoSequenceEventCmdResult {
    let text = cmd_info.args[0].to_string();
    if let Some(pos) = RING_PICTURE.iter().position(|r| *r == text){
        let new_index = DVCVariables::get_dvc_emblem_index(pos as i32, false);
        if let Some(pos) = RING_PICTURE.get(new_index) { cmd_info.args[0] = pos.into(); }
    }
    this.func_picture_hide(cmd_info)
}
fn set_random_background(this: &EventDemoSequence, cmd_info: &mut CmdInfo, _: OptionalMethod) -> EventDemoSequenceEventCmdResult {
    if cmd_info.args.len() >= 1 {
        if let Some(data) = SEARCH_LIST.get().and_then(|s| s.map_events.get_random_element(Random::get_system())){
            cmd_info.args[0] = data.as_str().into();
        }
    }
    this.func_set_back_ground(cmd_info)
}
fn set_motion(this: &EventDemoSequence, cmd_info: &mut CmdInfo, _: OptionalMethod) -> EventDemoSequenceEventCmdResult {
    let rng = Random::get_system();
    if cmd_info.args.len() >= 3 {
        if DVCFlags::CutsceneMotion.get_value() {
            let select = rng.get_value(47);
            cmd_info.args[2] = outfit_core::BODY_EXPRESS[select as usize].into();
        }
        if DVCFlags::CutsceneFacial.get_value() {
            let facial = rng.get_value(13);
            cmd_info.args[1] = outfit_core::FACIAL_STATES[facial as usize].0.into();
        }
    }
    this.func_character_play_motion(cmd_info)
}
fn sound_event(this: &EventDemoSequence, cmd_info: &mut CmdInfo, _: OptionalMethod) -> EventDemoSequenceEventCmdResult {
    if cmd_info.args.len() >= 1 && DVCFlags::CutsceneBGM.get_value()  {
        if cmd_info.args[0].str_contains("BGM") {
            GameSound::stop_all_bgm(GameSoundFadeSpeedType::Normal);
            if let Some(bgm) = crate::randomizer::bgm::BGM_POOL.get().and_then(|v| v.get_random_element(Random::get_system())) {
                cmd_info.args[0] = bgm.into();
            }
        }
    }
    this.func_sound_event(cmd_info)
}