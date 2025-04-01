use emblem::EMBLEM_LIST;
use engage::reliance::*;
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
    if GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 0 {
        EMBLEM_GIDS.iter()
            .for_each(|gid|{
                if let Some(g_unit) = GodPool::try_get_gid(gid, true) {
                    if g_unit.bonds == 0 { reset_bonds(g_unit); }
                }
            }
        );
        return; 
    }
    let units: Vec<_> = Force::get(ForceType::Player).unwrap().iter().chain(Force::get(ForceType::Absent).unwrap().iter()).collect();
    EMBLEM_LIST.get().unwrap().iter()
        .flat_map(|&index| GodData::try_get_hash(index))
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
    println!("Emblem {} has broken bonds and will be reset.", Mess::get(g_unit.data.mid));
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

pub fn create_bond_rings() {
    collect_bond_frags_from_achievements();
    if !DVCVariables::is_main_chapter_complete(3) { return; }
    let ring_list = RingData::get_list().unwrap();
    let mut active_emblems = 0;
    for x in 0..12 {
        if let Some(god_unit) = GodPool::try_get_gid(EMBLEM_GIDS[x], false){
            let bond_variable_count = format!("G_{}_Ring_Count", EMBLEM_ASSET[x]);
            if !GameVariableManager::exist(&bond_variable_count) { GameVariableManager::make_entry(&bond_variable_count, 0); }
            if GameVariableManager::get_number(&bond_variable_count) == 0 {
                for _x in 0..10 {
                    let rank = get_ring_rank() as usize; 
                    let ring_index = Random::get_game().get_value(10) as usize;
                    let ring_index = x * 40 + ring_index * 4 + rank;
                    add_ring(ring_list[ring_index].rid);
                }
                GameVariableManager::set_number(&bond_variable_count, 10);
            }
            if !god_unit.get_escape() { active_emblems += 1; }
        }
    }
    let total_cost = 500*active_emblems;
    // DLC
    for x in 12..19 {
        if let Some(god_unit) = GodPool::try_get_gid(EMBLEM_GIDS[x], false){
            if !god_unit.get_escape() { active_emblems += 1; }
        }
    }
    GameUserData::add_bond(500 + 150*active_emblems);
    let bond_frag = GameUserData::get_piece_bond();
    let mut ring_count = 0;
    RingData::get_list().unwrap().iter().for_each(|ring| ring_count += ring.get_pool_ring_stock());

    if total_cost < bond_frag && ring_count < 700 {
        for x in 0..12 {
            if let Some(god_unit) = GodPool::try_get_gid(EMBLEM_GIDS[x], false) {
                if !god_unit.get_escape() {
                    for _x in 0..5 {
                        let ring_index = x * 40 + ( Random::get_game().get_value(10) * 4) as usize + get_ring_rank() as usize;
                        add_ring(ring_list[ring_index].rid);
                    }
                }
            }
        }
        GameUserData::add_bond(total_cost);
    }
    // Auto merge rings
    let ring_cost = [game_parameter("指輪合成指輪コストB"), game_parameter("指輪合成指輪コストA"), game_parameter("指輪合成指輪コストS")];
    for x in 0..12 {    // Emblem Index
        for y in 0..10 {   // Ring Index
            for z in 0..3 {    //Rank Index
                let index = x*40 + y*4 + z;
                if ring_list[index].get_pool_ring_stock() >= ring_cost[z] {
                    UnitRingPool::sub_ring(ring_list[index].rid, None, ring_cost[z]);
                    add_ring(ring_list[index + 1].rid);
                }
            }
        }
    }
}

fn get_ring_rank() -> i32 {
    let rng = Random::get_game();
    let value = rng.get_value(100);
    let s_rate = 2*game_parameter("指輪精製確率S");
    let a_rate = game_parameter("指輪精製確率A") + s_rate;
    let b_rate = game_parameter("指輪精製確率B") + a_rate;
    if value < s_rate { return 3; }
    else if value < a_rate { return 2; }
    else if value < b_rate { return 1; }
    else { return 0; }
}

fn bond_frags_from_achievement(this: &AchieveData) -> i32 {
    let status = this.get_status();
    if status == AchieveDataStatus::Showed || status == AchieveDataStatus::Cleared {
        this.set_status(AchieveDataStatus::Completed);
        return this.get_reward();
    }
    return 0;
}

fn collect_bond_frags_from_achievements() {
    AchieveData::get_list().unwrap().iter().for_each(|achieve| { GameUserData::add_bond( bond_frags_from_achievement( achieve ) ); });
}

fn game_parameter(value: &str) -> i32 { unsafe { get_game_parameter(value.into(), None) } }
fn add_ring(ring_id: &Il2CppString) { UnitRingPool::add_ring(ring_id, None, 1 ); }

#[unity::from_offset("App", "MyRoomRelianceSelect","GetUnitList")]
pub fn my_room_reliance_select_get_unit_list(method_info: OptionalMethod) -> &'static List<Unit>;

#[skyline::from_offset(0x02280c20)]
fn get_game_parameter(value: &Il2CppString, method_info: OptionalMethod) -> i32;