use engage::unit::UnitReliance;
use super::*;

pub fn update_ignots(){
    let add_iron = get_continious_total_map_complete_count()*5;
    let mut add_steel = 0;
    let mut add_silver = 0;
    GameUserData::add_iron( add_iron );
    if GameVariableManager::get_bool("G_Cleared_G001") { add_silver += 1; } 
    if DVCVariables::is_main_chapter_complete(4) { add_steel += 1;} 
    if !DVCVariables::is_random_map() {
        if DVCVariables::is_main_chapter_complete(5) { add_steel += 5; } 
        if DVCVariables::is_main_chapter_complete(6) { add_steel += 5; } 
        if DVCVariables::is_main_chapter_complete(7) { add_silver += 1; } 
        if DVCVariables::is_main_chapter_complete(9) { add_silver += 1; } 
        if DVCVariables::is_main_chapter_complete(11) { add_silver += 1; } 
        if DVCVariables::is_main_chapter_complete(16) { 
            add_silver += 1;
            add_steel += 5;
        }
    }
    else {
        let cleared = get_story_chapters_completed();
        if cleared >= 5 { add_steel += 1;} 
        if cleared >= 6 { add_steel += 1;} 
        if cleared >= 7 { add_silver += 1; } 
        if cleared >= 9 { add_silver += 1; } 
        if cleared >= 11 { add_silver += 1 }
        if cleared >= 16 { 
            add_silver += 1;
            add_steel += 5;
        }
    }
    GameUserData::add_steel( add_steel );
    GameUserData::add_silver( add_silver );
}

pub fn add_support_points() {
    let unit_list = unsafe { my_room_reliance_select_get_unit_list(None) };
    for x in 0..unit_list.len() {
        let unit_a = &unit_list[x];
        let is_a_deployed = unit_a.force.unwrap().force_type == 0;
        let is_lueur = unit_a.person.pid.to_string() == PIDS[0];
        for y in x+1..unit_list.len(){
            let unit_b = &unit_list[y];
            if let Some(reliance_data) = UnitReliance::try_get(unit_a, unit_b) {
                let is_b_deployed = unit_b.force.unwrap().force_type == 0;
                let exp_needed = reliance_data.get_next_level_exp(reliance_data.level);
                if exp_needed != 100 && exp_needed != 0 { reliance_data.exp += 1 + (is_a_deployed || is_b_deployed) as i8 + is_lueur as i8; }
            }
        }
    }
}
#[unity::from_offset("App", "MyRoomRelianceSelect","GetUnitList")]
pub fn my_room_reliance_select_get_unit_list(method_info: OptionalMethod) -> &'static List<Unit>;
