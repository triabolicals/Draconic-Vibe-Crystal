use unity::prelude::*;
use super::*;
use engage::{
    battle::BattleCalculator, force::ForceType, gamesound::GameSound,gamevariable::GameVariableManager,
    sequence::{combatsequence::CombatSequence, mapsequence::battle::*, sortie::SortieSequence},
};
use std::sync::OnceLock;
use crate::assets::transform::is_monster_class;

pub static BGM_POOL: OnceLock<Vec<String>> = OnceLock::new();

pub fn initalize_bgm_pool() {
    BGM_POOL.get_or_init(||{
        let mut list = Vec::new();
        let music_list = music::MusicData::get_list().unwrap();
        for x in 6..music_list.len() {
            let event_name = music_list[x].event_name.to_string();
            if event_name.contains("BGM_Sys") { continue; }
            if let Some(event) = music_list[x].change_event_name { 
                list.push(event.to_string());
            }
            else if (x >= 56 && x <= 74) || x > 190 { list.push(event_name); }
        }
        list.push("BGM_Sys_Gmap".to_string());
        list.push("BGM_DLC_Sys_Gmap".to_string());
        list.push("BGM_Sys_Title_ST_Play".to_string());
        list
    });
 }

fn get_current_chapter_chapter_bgm(){
    let chapter = GameUserData::get_chapter();
    GameVariableManager::make_entry_str("OBGM4", chapter.sortie_bgm);
    GameVariableManager::make_entry_str("CBGM4", chapter.sortie_bgm);
    GameVariableManager::make_entry_str("OBGM1", chapter.player_phase_bgm);
    GameVariableManager::make_entry_str("OBGM2", chapter.enemy_phase_bgm);
    GameVariableManager::make_entry_str("OBGM3", chapter.ally_phase_bgm);
    GameVariableManager::make_entry_str("CBGM1", chapter.player_phase_bgm);
    GameVariableManager::make_entry_str("CBGM2", chapter.enemy_phase_bgm);
    GameVariableManager::make_entry_str("OBGM3", chapter.ally_phase_bgm);

}
pub fn set_random_bgm_phase(){
    let chapter = GameUserData::get_chapter();
    let pool = BGM_POOL.get().unwrap();
    let rng = Random::get_game();
    let size = pool.len() as i32;
    let player = rng.get_value( size );
    let enemy = rng.get_value( size );
    let ally = rng.get_value( size );
    let string1 = pool[ player as usize].as_str().into();
    let string2 = pool[ enemy as usize].as_str().into();
    let string3 = pool[ ally as usize].as_str().into();
    let string4 = &pool[rng.get_value( size ) as usize];
    GameVariableManager::set_string("CBGM1", string1);
    GameVariableManager::set_string("CBGM2", string2);
    GameVariableManager::set_string("CBGM3", string3);
    GameVariableManager::set_string("CBGM4", string4);
    chapter.sortie_bgm = string4.into();
    GameSound::field_bgm_set_phase_bgm2(string1, string2, string3);
    GameSound::field_bgm_set_first_played_flag();
    chapter.player_phase_bgm = string1.into();
    chapter.enemy_phase_bgm = string2.into();
    chapter.ally_phase_bgm = string3.into();
}

pub fn randomize_bgm_map() {
    if GameUserData::get_sequence() == 0 { return; }
    if GameUserData::is_encount_map() { return; }
    if !DVCFlags::BGM.get_value() { return; }
    get_current_chapter_chapter_bgm();
    set_random_bgm_phase();
    get_random_special(true);
}

fn get_random_special(set: bool) -> &'static Il2CppString {
    let rng = Random::get_game();
    let pool = BGM_POOL.get().unwrap();
    let size = pool.len() as i32;
    if !GameVariableManager::exist("SBGM") {
        GameVariableManager::make_entry_str("SBGM", pool[ rng.get_value( size ) as usize].as_str());
    }
    else if set { GameVariableManager::set_string("SBGM", pool[ rng.get_value( size ) as usize].as_str()); }
    GameVariableManager::get_string("SBGM") 
}

pub fn reset_bgm() {
    GameVariableManager::set_string("CBGM1",  GameVariableManager::get_string("OBGM1"));
    GameVariableManager::set_string("CBGM2",  GameVariableManager::get_string("OBGM2"));
    GameVariableManager::set_string("CBGM3",  GameVariableManager::get_string("OBGM3"));
    GameVariableManager::set_string("CBGM4",  GameVariableManager::get_string("OBGM4"));
    GameSound::field_bgm_set_phase_bgm2(GameVariableManager::get_string("OBGM1"), GameVariableManager::get_string("OBGM2"), GameVariableManager::get_string("OBGM3"));
}
pub fn change_bgm() {
    if GameUserData::is_encount_map() || GameUserData::get_sequence() == 0 { return; }
    let chapter = GameUserData::get_chapter();
    if !GameVariableManager::exist("CBGM1") || !GameVariableManager::exist("CBGM2") || !GameVariableManager::exist("CBGM3") {
        get_current_chapter_chapter_bgm();
    }
    if DVCFlags::BGM.get_value() {
        set_random_bgm_phase();
        get_random_special(true);
    }
    else {
        reset_bgm();
        chapter.player_phase_bgm = GameVariableManager::get_string("OBGM1");
        chapter.enemy_phase_bgm = GameVariableManager::get_string("OBGM2");
        chapter.ally_phase_bgm = GameVariableManager::get_string("OBGM3");
        chapter.sortie_bgm = GameVariableManager::get_string("OBGM4");
    }
    GameSound::field_bgm_set_phase_bgm(chapter, false);
}
pub fn random_special_bgm(calculator: &BattleCalculator) -> bool {
    if GameSound::field_bgm_is_special_battle_bgm(calculator) {
        GameSound::field_bgm_start_special_battle_bgm_continue_turn();
        let s = get_random_special(false);
        if !GameSound::is_event_playing(s) {
            if GameSound::field_bgm_set_phase_bgm2(s, s, s) {
                GameSound::field_bgm_play(ForceType::Player);
                GameSound::set_state_special_battle("PreSpecialCombat".into());
            }
        }
        true
    }
    else { false }
}

pub extern "C" fn map_sequence_battle_pre_bgm(this: &mut MapSequenceBattle, _method_info: OptionalMethod) {
    if DVCFlags::BGM.get_value() { if random_special_bgm(this.calculator) { return; } }
    this.to_pre_bgm();
}
pub extern "C" fn map_sequence_battle_action_pre_bgm(this: &mut MapSequenceBattleAction, _method_info: OptionalMethod) {
    for x in 0..2 {
        if let Some(s) = this.calculator.get_side(x) {
            if s.unit_item.is_some_and(|u| u.item.kind == 9 && u.item.iid.str_contains("チキ")) {
                if s.unit.is_some_and(|t|(crate::assets::transform::is_tiki_engage(t) && DVCFlags::EngageWeapons.get_value()) || !crate::assets::transform::is_tiki_engage(t)) {
                    s.status.value |= -2147483648;
                }
            }
            if s.unit.is_some_and(|u|is_monster_class(u)) && s.unit_item.is_some_and(|i| i.item.kind == 9 ){
                s.status.value |= -2147483648;
            }
        }
    }
    if DVCFlags::BGM.get_value() { if random_special_bgm(this.calculator) {return; } }
    this.to_pre_bgm();
}

pub extern "C" fn sortie_play_bgm(this: &mut SortieSequence, _optional_method: OptionalMethod) {
    if DVCFlags::BGM.get_value() { set_random_bgm_phase(); }
    this.play_bgm();
}
pub extern "C" fn combat_sequence_pre_bgm(this: &mut CombatSequence, _method_info: OptionalMethod) {
    if DVCFlags::BGM.get_value() && this.calculator.mode == 0 {
        if random_special_bgm(this.calculator) { return; }
    }
    this.to_pre_bgm();
}