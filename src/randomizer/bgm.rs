use unity::prelude::*;
use super::*;
use engage::{
    menu::{
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    gamevariable::GameVariableManager,
};
use engage::gamedata::dispos::ChapterData;
use std::sync::OnceLock;
static BGM_POOL: OnceLock<Vec<String>> = OnceLock::new();

pub fn initalize_bgm_pool() {
    BGM_POOL.get_or_init(||{
        let mut list = Vec::new();
        let music_list = MusicData::get_list().unwrap();
        for x in 6..music_list.len() {
            let event_name = music_list[x].event_name.to_string();
            if event_name.contains("BGM_Sys") { continue; }
            if let Some(event) = music_list[x].change_event_name { 
                list.push(event.to_string());
            }
            else if x >= 68 && x <= 74 { list.push(event_name); }
        }
        list

    });
 }

#[unity::class("App", "MusicData")]
pub struct MusicData {
    pub parent: StructBaseFields,
    pub event_name: &'static Il2CppString,
    pub name: &'static Il2CppString,
    pub help: &'static Il2CppString,
    pub condition: &'static Il2CppString,
    pub amiibo: &'static Il2CppString,
    pub change_event_name: Option<&'static Il2CppString>,
    pub is_change: bool,
}
impl Gamedata for MusicData {}

fn get_current_chapter_chapter_bgm(){
    let chapter = GameUserData::get_chapter();
    GameVariableManager::make_entry_str("OBGM1", chapter.get_player_bgm().unwrap());
    GameVariableManager::make_entry_str("OBGM2", chapter.get_enemy_bgm().unwrap());
    GameVariableManager::make_entry_str("OBGM3", chapter.get_ally_bgm().unwrap());
    GameVariableManager::make_entry_str("CBGM1", chapter.get_player_bgm().unwrap());
    GameVariableManager::make_entry_str("CBGM2", chapter.get_enemy_bgm().unwrap());
    GameVariableManager::make_entry_str("CBGM3", chapter.get_ally_bgm().unwrap());
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

    GameVariableManager::set_string("CBGM1", string1);
    GameVariableManager::set_string("CBGM2", string2);
    GameVariableManager::set_string("CBGM3", string3);
    unsafe {
        set_phase_bgm(string1, string2, string3, None);
        set_first_played_flag(None);
    }
    chapter.set_player_bgm(string1);
    chapter.set_enemy_bgm(string2);
    chapter.set_ally_bgm(string3);
}


pub fn randomize_bgm_map() {
    if !DVCVariables::random_enabled() { return; }
    if GameUserData::is_encount_map() { return; }
    get_current_chapter_chapter_bgm();
    if !GameVariableManager::get_bool(DVCVariables::BGM_KEY) { return; }
    set_random_bgm_phase();
    get_random_special(true);
}
#[skyline::hook(offset=0x0228deb0)]
pub fn special_bgm_hook(calculator: u64, method_info: OptionalMethod) -> Option<&'static Il2CppString> {
    let result = call_original!(calculator, method_info);
    if result.is_some() && GameVariableManager::get_bool(DVCVariables::BGM_KEY) { return Some(get_random_special(false)); }
    else { return result; }
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

    unsafe {
        set_phase_bgm(GameVariableManager::get_string("OBGM1"), GameVariableManager::get_string("OBGM2"), GameVariableManager::get_string("OBGM3"), None);
    }
}
pub fn change_bgm() {
    if GameUserData::is_encount_map() { return; }
    let chapter = GameUserData::get_chapter();
    if !GameVariableManager::exist("CBGM1") || !GameVariableManager::exist("CBGM2") || !GameVariableManager::exist("CBGM3") {
        get_current_chapter_chapter_bgm();
    }
    if GameVariableManager::get_bool(DVCVariables::BGM_KEY) {
        set_random_bgm_phase();
        get_random_special(true);
    }
    else {
        reset_bgm();
        chapter.set_player_bgm( GameVariableManager::get_string("OBGM1") ); 
        chapter.set_enemy_bgm( GameVariableManager::get_string("OBGM2") ); 
        chapter.set_ally_bgm( GameVariableManager::get_string("OBGM3") ); 
    }
    unsafe {
        set_phase_by_chapter(chapter, false, None);
        field_bgm_play(0, None);
    }
}


pub struct RandomBGMMod;
impl ConfigBasicMenuItemSwitchMethods for RandomBGMMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_map_bgm } 
            else { GameVariableManager::get_bool(DVCVariables::BGM_KEY) };
            
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_map_bgm = result; }
            else { GameVariableManager::set_bool(DVCVariables::BGM_KEY, result);  }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if GameUserData::get_sequence() != 3 {
            this.help_text = if CONFIG.lock().unwrap().random_map_bgm { "Map BGM will be randomized for each phase." }
            else { "Default Map BGM for each phase."}.into();
        }
        else {
            this.help_text = if  GameVariableManager::get_bool(DVCVariables::BGM_KEY) { "Map BGM will be randomized. Press A to Change." }
            else { "Default Map BGM. Press A to Change."}.into();
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_map_bgm } 
            else {  GameVariableManager::get_bool(DVCVariables::BGM_KEY) };
        this.command_text = if value { "Random" } else { "Default" }.into();
    }
}

pub struct BGMConfirm;
impl TwoChoiceDialogMethods for BGMConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        change_bgm();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn bgm_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() != 3 {return BasicMenuResult::new(); }
    YesNoDialog::bind::<BGMConfirm>(this.menu, "Change Map BGM?", "Do it!", "Nah..");
    return BasicMenuResult::new();
}

pub extern "C" fn vibe_bgm() -> &'static mut ConfigBasicMenuItem {  
    let switch =  ConfigBasicMenuItem::new_switch::<RandomBGMMod>("Map BGM Setting");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = bgm_acall as _ );
    switch
}


#[skyline::from_offset(0x0228c330)]
pub fn set_phase_bgm(player: &Il2CppString, enemy: &Il2CppString,ally: &Il2CppString, method_info: OptionalMethod);

#[skyline::from_offset(0x0228c0f0)]
fn set_phase_by_chapter(chapter: &ChapterData, is_encounter: bool, method_info: OptionalMethod);

#[skyline::from_offset(0x0228c470)]
fn field_bgm_play(force: i32, method_info: OptionalMethod);

#[skyline::from_offset(0x0228d100)]
pub fn set_first_played_flag(method_info: OptionalMethod);
