use engage::{god::GodPool, map::history::MapHistory};
use super::*;

pub fn random_map_mode_level() -> i32 {
    max( (get_story_chapters_completed()-6)*2, get_story_chapters_completed() + 4)
}

pub fn continous_rand_emblem_adjustment() {
    if DVCVariables::Continuous.get_value() < 2 { return; }
    MapHistory::rewind_enable();
    if DVCVariables::is_main_chapter_complete(22) { for x in 0..12 { escape_god(EMBLEM_GIDS[x], false); }   return; }
    let current_chapter = GameUserData::get_chapter().cid.to_string();
    if current_chapter.contains("M011") && !DVCVariables::is_main_chapter_complete(11) {
        for x in 0..6 { escape_god(EMBLEM_GIDS[x], true); }  
        return;
    }
    if current_chapter.contains("M022") {
        escape_god(EMBLEM_GIDS[0], false);
        for x in 1..12 { escape_god(EMBLEM_GIDS[x], true); } 
        return;
    }
    if DVCVariables::is_main_chapter_complete(10) && !DVCVariables::is_main_chapter_complete(11){
        for x in 0..6 { escape_god(EMBLEM_GIDS[x], false); }  
    }
    if DVCVariables::is_main_chapter_complete(21) && !DVCVariables::is_main_chapter_complete(22){
        for x in 0..12 { escape_god(EMBLEM_GIDS[x], false); }  
    }

}

pub fn escape_god(gid: &str , escape: bool) {
    if let Some(god_data) = if DVCVariables::EmblemRecruitment.get_value() == 0 { GodData::get(gid) }
        else { GodData::get( GameVariableManager::get_string(format!("G_R_{}", gid)))} {
        if let Some(god_unit) = GodPool::try_get(god_data, true){
            god_unit.set_escape(escape);
            if escape {
                if let Some(parent) = god_unit.parent_unit {  parent.clear_god_unit(); }
            }
        }
    }
}
pub fn is_god_available(gid: &str, randomized: bool) -> bool {
    if let Some(god_data) = if DVCVariables::EmblemRecruitment.get_value() == 0 || !randomized { GodData::get(gid) }
        else { GodData::get( GameVariableManager::get_string(format!("G_R_{}", gid)))} {
        if let Some(god_unit) = GodPool::try_get(god_data, true) {
            return !god_unit.get_escape();   
        }
        else { return false; }
    }
    false
}

pub fn set_next_random_chapter(current_chapter: &ChapterData) -> Option<&'static ChapterData> {
    let prefixless = current_chapter.prefixless_cid.to_string();
    GameVariableManager::set_bool(format!("G_Cleared_{}", prefixless), true);
    continous_rand_emblem_adjustment();
    let dlc = continuous_mode_dlc_allowed();
    let mut complete_mask = 0i64;
    for x in 1..57 {
        if DVCVariables::is_main_chapter_complete(x) { complete_mask |= 1 << x; }
    }
    let mut available: Vec<i32> = Vec::new();
    let m011_cleared = complete_mask & (1 << 11) != 0;

    // No Previous Chapter Completed needed
    available.extend([5, 6, 7, 8, 9, 10, 12, 13, 15, 18, 31, 32].iter().filter(|&&v| complete_mask & (1 << v) == 0));

    // Chapter 7 + 10 for 14 and 16
    if complete_mask & 640 == 640 { available.extend([14, 16].iter().filter(|&&v| complete_mask & (1 << v) == 0)); }
    if m011_cleared {
        available.extend([17, 19, 20, 21].iter().filter(|&&v| complete_mask & (1 << v) == 0));
        available.extend([44, 39, 43, 41, 42, 40, 33, 34, 35, 36, 37, 38].into_iter().enumerate()
            .filter(|(i, v)|is_god_available(EMBLEM_GIDS[*i], false) && complete_mask & (1 << *v) == 0)
            .map(|(_, x)| x)
        );
    }
    else if complete_mask & 2880 == 832 { available.push(11); }
    if complete_mask & 8120320 == 3926016 { available.push(22); }
    if complete_mask & 4194304 != 0 {
        available.extend([23, 24, 25, 26, 45].iter().filter(|&&v| complete_mask & (1 << v) == 0));
    }
    let completed = get_continious_total_map_complete_count();
    if completed > 10 && dlc {
        available.extend((51..57).filter(|&v| complete_mask & (1 << v) == 0));
    }
    if available.len() == 0 { return ChapterData::get("CID_M026"); }
    let rng = Random::get_game();
    loop {
        if let Some(i) = available.get_random_element(rng).map(|v| get_chapter_from_idx(*v))
            .and_then(|i| ChapterData::get(format!("CID_{}", i)))
        {
            /*
            println!("Current Chapter: {}", current_chapter.cid);
            println!("New Random Chapter: {} out of {} Possible", i.cid, available.len() );
            println!("Number of Map Completed: {}", completed);
            println!("Number of Story Maps Completed: {}", get_story_chapters_completed());

             */
            return Some(i);
        }
    }
}

