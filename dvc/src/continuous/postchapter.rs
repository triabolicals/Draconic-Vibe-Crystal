use engage::{god::{GodPool, GodUnit}, unit::UnitReliance};
use crate::randomizer::data::GameData;
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

pub fn update_bonds() {
    if DVCVariables::Continuous.get_value() == 0 {
        for x in 0..19 {
            if let Some(g_unit) = GodPool::try_get_gid(EMBLEM_GIDS[x], true) {
                if g_unit.bonds == 0 { reset_bonds(g_unit); }
            }
        }
        return; 
    }
    let units: Vec<_> = Force::get(ForceType::Player).unwrap().iter().chain(Force::get(ForceType::Absent).unwrap().iter()).collect();
    GameData::get_playable_emblem_hashes().into_iter()
        .flat_map(|index| GodData::try_get_hash(index))
        .flat_map(|god| GodPool::try_get(god, false))
        .for_each(|god_unit|{
            if god_unit.data.force_type == 0 {
                if god_unit.bonds == 0 { reset_bonds(god_unit); }
                else {
                    let mut max_level = 1;
                    let mut bond_exp: u16 = 0;
                // Get highest bond level 
                    units.iter().flat_map(|unit| god_unit.get_bond(unit)).for_each(|g_bond|{
                        if g_bond.level == 4 || g_bond.level == 19 { g_bond.level_up(); }
                        if max_level < g_bond.level { 
                            max_level = g_bond.level;
                            bond_exp = g_bond.exp;
                        }
                    });
                    units.iter().for_each(|unit|{
                        if let Some(g_bond) = god_unit.get_bond(unit) {
                            while g_bond.level < max_level { 
                                g_bond.level_up(); 
                                g_bond.exp = bond_exp;
                                unit.inherit_apt(god_unit);
                            }
                        }
                    });
                }
            }
        }
    );
}

fn reset_bonds(g_unit: &GodUnit) {
    let escape = g_unit.get_escape();
    // println!("Emblem {} has broken bonds and will be reset.", Mess::get(g_unit.data.mid));
    let data = g_unit.data;
    GodPool::delete(g_unit);
    if let Some(god_unit2) = GodPool::create(data) {
        god_unit2.set_escape(escape); 
        Force::get(ForceType::Absent).unwrap().iter()
            .for_each(|unit|{
                if let Some(g_bond) = g_unit.get_bond(unit){
                    for _x in 0..9 { g_bond.level_up(); }
                    unit.inherit_apt(g_unit);  
                }
            }  
        );
    }
}

#[unity::from_offset("App", "MyRoomRelianceSelect","GetUnitList")]
pub fn my_room_reliance_select_get_unit_list(method_info: OptionalMethod) -> &'static List<Unit>;
