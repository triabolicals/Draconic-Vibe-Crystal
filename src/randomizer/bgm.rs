use unity::prelude::*;
use engage::{
    menu::{
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    gamevariable::GameVariableManager,
    gameuserdata::*,
    random::*,
    gamedata::*,
    dialog::yesno::*,
};
use engage::gamedata::dispos::ChapterData;
use std::sync::Mutex;
use super::CONFIG;
static BGM_POOL: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CHAPTER_BGM_LIST: Mutex<[i32; 450]> = Mutex::new([-1; 450]);
static mut BGM_INDEX: [i32; 3] = [0; 3];

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

pub fn get_bgm_pool() {
    if BGM_POOL.lock().unwrap().len() != 0 { return; }
    let music_list = MusicData::get_list().unwrap();
    for x in 6..music_list.len() {
        if crate::utils::str_contains(music_list[x].event_name, "BGM_Sys_ED") { continue; }
        if crate::utils::str_contains(music_list[x].event_name, "BGM_Sys") { 
            BGM_POOL.lock().unwrap().push(music_list[x].event_name.get_string().unwrap());
        }
        else if music_list[x].change_event_name.is_some() {
            BGM_POOL.lock().unwrap().push(music_list[x].change_event_name.unwrap().get_string().unwrap());
        }
        else if x >= 68 && x <= 74 {
            BGM_POOL.lock().unwrap().push(music_list[x].event_name.get_string().unwrap());
        }
    }
    let chapter_list = ChapterData::get_list().unwrap();
    let mut list = CHAPTER_BGM_LIST.lock().unwrap();
    for x in 0..chapter_list.len() {
        if x >= 150 { break; }
        let chapter = &chapter_list[x as usize];
        unsafe {
            list[3*x] = get_music_event_index( chapter_get_player_bgm(chapter, None) );
            list[3*x+1] = get_music_event_index( chapter_get_enemy_bgm(chapter, None) );
            list[3*x+2] = get_music_event_index( chapter_get_ally_bgm(chapter, None) );
        }
    }
}
fn get_music_event_index(bgm: Option<&Il2CppString>) -> i32 {
    if bgm.is_none() { return -1; }
    let music_list = MusicData::get_list().unwrap();
    let compare = bgm.unwrap().get_string().unwrap();

    for x in 0..music_list.len() {
        if music_list[x].change_event_name.is_none() { continue; }
        if music_list[x].change_event_name.unwrap().get_string().unwrap() == compare { return x as i32; }
    }
    return -1;
}


fn get_chapter_bgm(chapter: &ChapterData){
    unsafe {
        let x = chapter.parent.index as usize;
        BGM_INDEX[0] = CHAPTER_BGM_LIST.lock().unwrap()[3*x];
        BGM_INDEX[1] = CHAPTER_BGM_LIST.lock().unwrap()[3*x+1];
        BGM_INDEX[2] = CHAPTER_BGM_LIST.lock().unwrap()[3*x+2];
    }
}
pub fn randomize_bgm_map() {
    if !crate::utils::can_rand() { return; }
    if GameUserData::is_encount_map() { return; }
    let chapter = GameUserData::get_chapter();
    get_chapter_bgm(chapter);
    // Initial Randomized at Map Start
    if !GameVariableManager::get_bool("G_RandomBGM") { return; }
    let rng = Random::get_game();
    let size = BGM_POOL.lock().unwrap().len() as i32;
    let string1 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();
    let string2 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();
    let string3 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();
    unsafe {
        set_phase_bgm(string1, string2, string3, None);
        set_first_played_flag(None);
        chapter_set_ally_bgm(chapter, string3, None);
        chapter_set_enemy_bgm(chapter, string2, None);
        chapter_set_player_bgm(chapter, string1, None);
    }
}
pub fn reset_bgm() {
    let chapter = GameUserData::get_chapter();
    unsafe {
        let x = chapter.parent.index as usize;
        BGM_INDEX[0] = CHAPTER_BGM_LIST.lock().unwrap()[3*x];
        BGM_INDEX[1] = CHAPTER_BGM_LIST.lock().unwrap()[3*x+1];
        BGM_INDEX[2] = CHAPTER_BGM_LIST.lock().unwrap()[3*x+2];
    }
}
pub fn change_bgm() {
    let chapter = GameUserData::get_chapter();
    if unsafe {  BGM_INDEX[0] == -1 && BGM_INDEX[1] == -1 &&  BGM_INDEX[2] == -1 } {
        if GameUserData::is_encount_map() { return; }
        get_chapter_bgm(chapter);
    }
    let rng = Random::get_game();
    if GameVariableManager::get_bool("G_RandomBGM") {
        let size = BGM_POOL.lock().unwrap().len() as i32;
        let string1 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();
        let string2 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();
        let string3 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();

        unsafe {
            set_phase_bgm(string1, string2, string3, None);
            chapter_set_ally_bgm(chapter, string3, None);
            chapter_set_enemy_bgm(chapter, string2, None);
            chapter_set_player_bgm(chapter, string1, None);
            field_bgm_play(0, None);
        }
        println!("Bgm: {}", string1.get_string().unwrap());
    }
    else {
        let music_list = MusicData::get_list().unwrap();
        reset_bgm();
        unsafe {
            if BGM_INDEX[0] > 0 { chapter_set_player_bgm( chapter, music_list[ BGM_INDEX[0] as usize ].change_event_name.unwrap(), None ); }
            if BGM_INDEX[1] > 0 { chapter_set_enemy_bgm(  chapter, music_list[ BGM_INDEX[1] as usize ].change_event_name.unwrap(), None ); }
            if BGM_INDEX[2] > 0 { chapter_set_ally_bgm(  chapter, music_list[ BGM_INDEX[2] as usize ].change_event_name.unwrap(), None ); }
            set_phase_by_chapter(chapter, false, None);
            field_bgm_play(0, None);
        }

    }
}


pub struct RandomBGMMod;
impl ConfigBasicMenuItemSwitchMethods for RandomBGMMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_map_bgm } 
                    else { GameVariableManager::get_bool("G_RandomBGM") };
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_map_bgm = result; }
            else { GameVariableManager::set_bool("G_RandomBGM", result);  }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if GameUserData::get_sequence() == 0 {
            this.help_text = if CONFIG.lock().unwrap().random_map_bgm { "Map BGM will be randomized for each phase." }
            else { "Default Map BGM for each phase."}.into();
        }
        else {
            this.help_text = if GameVariableManager::get_bool("G_RandomBGM") { "Map BGM will be randomized. Press A to Change." }
            else { "Default Map BGM. Press A to Change."}.into();
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_map_bgm } 
                    else { GameVariableManager::get_bool("G_RandomBGM") };
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

#[unity::from_offset("App", "ChapterData", "get_AllyPhaseBgm")]
pub fn chapter_get_ally_bgm(this: &ChapterData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "ChapterData", "get_EnemyPhaseBgm")]
pub fn chapter_get_enemy_bgm(this: &ChapterData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "ChapterData", "get_PlayerPhaseBgm")]
pub fn chapter_get_player_bgm(this: &ChapterData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "ChapterData", "set_AllyPhaseBgm")]
pub fn chapter_set_ally_bgm(this: &ChapterData, value: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "ChapterData", "set_EnemyPhaseBgm")]
pub fn chapter_set_enemy_bgm(this: &ChapterData, value: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "ChapterData", "set_PlayerPhaseBgm")]
pub fn chapter_set_player_bgm(this: &ChapterData, value: &Il2CppString, method_info: OptionalMethod);