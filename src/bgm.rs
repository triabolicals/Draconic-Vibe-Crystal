use unity::prelude::*;
use engage::{
    menu::{
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    mess::*,
    gamevariable::*,
    gameuserdata::*,
    random::*,
    gamedata::*,
};
use engage::gamedata::dispos::ChapterData;
use std::sync::Mutex;
use super::{CONFIG, DeploymentConfig};
static BGM_POOL: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[unity::class("App", "MusicData")]
pub struct MusicData {
    pub parent: StructBaseFields,
    pub event_name: &'static Il2CppString,
    pub name: &'static Il2CppString,
    pub help: &'static Il2CppString,
    pub condition: &'static Il2CppString,
    pub amiibo: &'static Il2CppString,
    pub change_event_name: &'static Il2CppString,
    pub is_change: bool,
}
impl Gamedata for MusicData {}

pub fn get_bgm_pool() {
    if BGM_POOL.lock().unwrap().len() != 0 { return; }
    let music_list = MusicData::get_list().unwrap();
    for x in 0..music_list.len() {
        if music_list[x].is_change == false { continue; }
        BGM_POOL.lock().unwrap().push(music_list[x].event_name.get_string().unwrap());
       // println!("BGM: {} Added: {}.", music_list[x].event_name.get_string().unwrap(), Mess::get(music_list[x].name).get_string().unwrap());
    }
}
pub fn randomize_bgm_map() {
    if !CONFIG.lock().unwrap().random_map_bgm { return; }
    let rng = Random::get_game();
    let size = BGM_POOL.lock().unwrap().len() as i32;
    let string1 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();
    let string2 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();
    let string3 = (&BGM_POOL.lock().unwrap()[ rng.get_value( size ) as usize]).into();
    let chapter = GameUserData::get_chapter();
    unsafe {
        set_phase_bgm(string1, string2, string3, None);
        set_first_played_flag(None);
        chapter_set_ally_bgm(chapter, string3, None);
        chapter_set_enemy_bgm(chapter, string2, None);
        chapter_set_player_bgm(chapter, string1, None);
    }
}

pub struct RandomBGMMod;
impl ConfigBasicMenuItemSwitchMethods for RandomBGMMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_map_bgm);
        if CONFIG.lock().unwrap().random_map_bgm != result {
            CONFIG.lock().unwrap().random_map_bgm = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_map_bgm { this.help_text = "Map BGM will be randomized for each phase.".into(); }
        else { this.help_text = "No changes to map BGM.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_map_bgm { this.command_text = "Randomized".into();  }
        else { this.command_text = "Normal".into(); }
    }
}

#[skyline::from_offset(0x0228c330)]
pub fn set_phase_bgm(player: &Il2CppString, enemy: &Il2CppString,ally: &Il2CppString, method_info: OptionalMethod);

#[skyline::from_offset(0x0228d100)]
pub fn set_first_played_flag(method_info: OptionalMethod);

#[unity::from_offset("App", "ChapterData", "set_AllyPhaseBgm")]
pub fn chapter_set_ally_bgm(this: &ChapterData, value: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "ChapterData", "set_EnemyPhaseBgm")]
pub fn chapter_set_enemy_bgm(this: &ChapterData, value: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "ChapterData", "set_PlayerPhaseBgm")]
pub fn chapter_set_player_bgm(this: &ChapterData, value: &Il2CppString, method_info: OptionalMethod);