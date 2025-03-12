use super::*;

pub fn random_map_mode_level() -> i32 {
    crate::utils::max( (crate::continuous::get_story_chapters_completed()-6)*2, crate::continuous::get_story_chapters_completed() + 4)
}

pub fn continous_rand_emblem_adjustment() {
    if GameVariableManager::get_number(DVCVariables::CONTINIOUS) < 3 { return; }
    unsafe { enable_map_rewind(None); }
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
    if let Some(god_data) = if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { GodData::get(gid) }
        else { GodData::get( GameVariableManager::get_string(format!("G_R_{}", gid)))} {
        if let Some(god_unit) = GodPool::try_get(god_data, true){
            god_unit.set_escape(escape);
            if escape {
                if let Some(parent) = god_unit.parent_unit {  parent.clear_parent(); }
            }
        }
    }
}
fn is_god_available(gid: &str, randomized: bool) -> bool {
    if let Some(god_data) = if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 || !randomized { GodData::get(gid) }
        else { GodData::get( GameVariableManager::get_string(format!("G_R_{}", gid)))} {
        if let Some(god_unit) = GodPool::try_get(god_data, true) {
            return !god_unit.get_escape();   
        }
        else { return false; }
    }
    false
}

pub fn set_next_random_chapter(current_chapter: &ChapterData) {
    let prefixless = current_chapter.get_prefixless_cid().to_string();
    continous_rand_emblem_adjustment();
    if crate::randomizer::RANDOMIZER_STATUS.read().unwrap().continious_random_chapter == prefixless && !GameVariableManager::get_bool(format!("G_Cleared_{}", prefixless)) { return; }
    let dlc = continuous_mode_dlc_allowed();

    let completed = GameVariableManager::find_starts_with("G_Cleared_").iter().filter(|key| GameVariableManager::get_bool(key.to_string())).count();
    let mut available: Vec<String> = Vec::new();
    let m011_cleared = DVCVariables::is_main_chapter_complete(11);
    let m011 = DVCVariables::is_main_chapter_complete(6) && DVCVariables::is_main_chapter_complete(8) && DVCVariables::is_main_chapter_complete(9);
    ["M005", "M006", "M007", "M008", "M009", "M010", "M012", "M013", "M015", "M018", "S001", "S002"].iter()
        .for_each(|key| if !GameVariableManager::get_bool(format!("G_Cleared_{}", key)) { available.push(key.to_string());} );

    if DVCVariables::is_main_chapter_complete(7) && DVCVariables::is_main_chapter_complete(10) {
        if !DVCVariables::is_main_chapter_complete(14) { available.push("M014".to_string()); } 
        if !DVCVariables::is_main_chapter_complete(16) { available.push("M016".to_string()); }
    }

    if m011_cleared { ["M017", "M019", "M020"].iter().for_each(|key| if !GameVariableManager::get_bool(format!("G_Cleared_{}", key)) { available.push(key.to_string()); } ); }
    else if m011 { available.push("M011".to_string());  }
    if DVCVariables::is_main_chapter_complete(14) && DVCVariables::is_main_chapter_complete(16) && DVCVariables::is_main_chapter_complete(17) 
        && DVCVariables::is_main_chapter_complete(19) && !DVCVariables::is_main_chapter_complete(21) { available.push("M021".to_string()); }

    let m022 = ["M011", "M013", "M014", "M015", "M016", "M017", "M019", "M020", "M021"].iter().filter(|&x|GameVariableManager::get_bool(format!("G_Cleared_{}", x))).count();
    if m022 == 9 { 
        if !DVCVariables::is_main_chapter_complete(22) { available.push("M022".to_string()); }
        else if !GameVariableManager::get_bool("G_Cleared_S015") { available.push("S015".to_string()); }
    }
    if completed >= 20 && DVCVariables::is_main_chapter_complete(22) { 
        ["M023", "M024", "M025", "M026"].iter().for_each(|key| if !GameVariableManager::get_bool(format!("G_Cleared_{}", key)) {available.push(key.to_string());} );
    }
    if let Some(pos) = available.iter().position(|key| *key == prefixless) { available.remove(pos); }
    if completed >= 15 {    // Paralogues
        for x in 0..12 {
            let e_index = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
            let para = EMBELM_PARA[ x as usize ];
            if e_index == 13 {
                if is_god_available(EMBLEM_GIDS[x as usize], false) && m011_cleared {
                    if !GameVariableManager::get_bool(format!("G_Cleared_{}", para)) { available.push(para.to_string()); }
                }
            }
            else if ( ( x < 6 && m011_cleared ) || x >= 6 ) && is_god_available(EMBLEM_GIDS[x as usize], false) {
                if !GameVariableManager::get_bool(format!("G_Cleared_{}", para)) { available.push(para.to_string()); } 
            }
        }
    }
    if dlc {
        for x in 1..7 {
            let god = format!("G00{}", x);
            if !GameVariableManager::get_bool(format!("G_Cleared_{}", god)) {available.push(god); }
        }
    }
    if available.len() == 0 {
        current_chapter.set_next_chapter("CID_M026");
        return; 
    }
    let rng = Random::get_game();
    let mut key= available[ rng.get_value( available.len() as i32 ) as usize ].to_string();

    if ( dlc && key.contains("G00") ) || ( completed >= 15 && key.contains("S0")) {
        key = available[ rng.get_value( available.len() as i32 ) as usize ].to_string();
    }
    let cid = format!("CID_{}", key);

    if GameUserData::get_sequence() == 7 {
        let chapter = current_chapter.cid;
        if chapter.contains("M011") && !m011 || chapter.contains("M022") && m022 < 9 {
            GameUserData::set_chapter(ChapterData::get(cid).unwrap());
            return;
        }
    }
    println!("Current Chapter: {}", current_chapter.cid);
    println!("New Random Chapter: {} out of {} Possible", cid, available.len() );
    println!("Number of Map Completed: {}", completed);
    println!("Number of Story Maps Completed: {}", get_story_chapters_completed());

    crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.continious_random_chapter = prefixless.to_string()).unwrap();
    current_chapter.set_next_chapter(cid.as_str());
}

