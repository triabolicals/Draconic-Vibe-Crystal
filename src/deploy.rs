use unity::{il2cpp::class::Il2CppRGCTXData, prelude::*};
use skyline::patching::Patch;
use engage::random::Random;
use engage::{
    gamevariable::*, 
    gameuserdata::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    godpool::*,
    force::*,
    script::*,
    singleton::SingletonClass,
    gamedata::{*, GodData, unit::*},
};
use unity::system::List;
use unity::il2cpp::object::Array;
use super::CONFIG;
use crate::person::lueur_on_map;
use crate::{enums::*, person, item};

// Calculate the unit's displayed rating 
pub fn get_unit_rating(this: &Unit) -> i32 {
    let mut result: i32 = 0;
    for x in 1..9 { result += this.get_capability(x as i32, false);  }
    result
}
pub fn unit_status() {
    let player_force = Force::get(ForceType::Player).unwrap();
    let mut force_iter = Force::iter(player_force);
    while let Some(unit) = force_iter.next() {
        println!("Player Status of {}: {}", unit.person.get_name().unwrap().get_string().unwrap(), unit.status.value);
    }
    let absent_force = Force::get(ForceType::Absent).unwrap();
    let mut force_iter = Force::iter(absent_force );
    while let Some(unit) = force_iter.next() {
        println!("Absent Status of {}: {}", unit.person.get_name().unwrap().get_string().unwrap(), unit.status.value);
    }
    let dead_force = Force::get(ForceType::Dead).unwrap();
    let mut force_iter = Force::iter( dead_force  );
    let emblem_lueur = GodPool::try_get_gid("GID_リュール", false);
    let has_emblem_lueur = emblem_lueur.is_some();
    while let Some(unit) = force_iter.next() {
        println!("Dead Status of {}: {}", unit.person.get_name().unwrap().get_string().unwrap(), unit.status.value);
        if unit.person.pid.get_string().unwrap() == "PID_リュール" && has_emblem_lueur {
            let god_lueur = GodData::get("GID_リュール").unwrap();
            if god_lueur.get_flag().value & -2147483648 != 0 {
                god_lueur.get_flag().value -= -2147483648;
            }
        }
    }

 }
// Generating the list of equipable emblems
pub fn get_emblem_list() -> Vec<&'static str> {
    let mut result: Vec<&str> = Vec::new();
    for x in EMBLEM_GIDS {
        let god_unit = GodPool::try_get_gid(x, true);
        if god_unit.is_some() { if !god_unit.unwrap().get_escape() { result.push(x); } }
    }
    result
}
pub fn unit_selection_menu_enable(enabled: bool) {
    if enabled { Patch::in_text(0x01d78578).bytes(&[0x20, 0x00, 0x80, 0x52]).unwrap(); }
    else { Patch::in_text(0x01d78578).bytes(&[0x80, 0x00, 0x80, 0x52]).unwrap(); }
}
pub fn emblem_selection_menu_enable(enabled: bool) {
    if enabled { Patch::in_text(0x01d76fb8).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap(); }
    else { Patch::in_text(0x01d76fb8).bytes(&[0x88, 0x00, 0x80, 0x52]).unwrap(); }
}
//Hook to function that creates the sortie deploy positions to do deployment stuff

pub fn get_emblem_paralogue_level() {
    let cid = GameUserData::get_chapter().get_prefixless_cid();
    let mut emblem_index = 0;
    let new_emblem_index;
    // Find indices
    GameVariableManager::make_entry("G_Paralogue_Level", 0);
    GameVariableManager::set_number("G_Paralogue_Level", 0);
    if !crate::utils::can_rand() { return; }
    loop {
        if crate::utils::str_contains(cid, EMBELM_PARA[emblem_index as usize]) {
            let found = person::pid_to_index(&EMBLEM_GIDS[emblem_index as usize].to_string(), false);
            if found != -1 { 
                new_emblem_index = found; 
                break;
            }
            else { return; }
        }
        else { emblem_index += 1; }
        if emblem_index >= 12 { return; }
    }
    let level_difference;
    if new_emblem_index >= 12 {
        let party_average = crate::autolevel::get_difficulty_adjusted_average_level();
        println!("Party Average: {}", party_average);
        level_difference = party_average - 2 - PARA_LEVEL[emblem_index as usize];
        if level_difference >= 0 { GameVariableManager::set_number("G_Paralogue_Level", 0); }
        else { GameVariableManager::set_number("G_Paralogue_Level", level_difference); }

    }
    else {
        level_difference = PARA_LEVEL[ new_emblem_index as usize ] - PARA_LEVEL[emblem_index as usize];
        GameVariableManager::set_number("G_Paralogue_Level", level_difference);
    }
    println!("Paralogue Level Difference: {} | {}", level_difference, GameVariableManager::get_number("G_Paralogue_Level"));
}


#[unity::hook("App", "MapDispos", "CreatePlayerTeam")]
pub fn create_player_team(group: &Il2CppString, method_info: OptionalMethod){
    let absent_force = Force::get(ForceType::Absent).unwrap();
    let hero_unit = absent_force.get_hero_unit();
    if GameVariableManager::get_number("G_DeploymentMode") == 3 {
        if GameUserData::get_status().value & 64 != 0 { GameUserData::get_status().value -= 64;  }  //Disables Continuous Flag 
    }
    if crate::utils::can_rand() {  
        if GameVariableManager::get_number("G_Random_Recruitment") != 0 { person::change_map_dispos(); }
        let random_job =  GameVariableManager::get_number("G_Random_Job") == 1 || GameVariableManager::get_number("G_Random_Job") == 3;
        if random_job && !GameVariableManager::get_bool("G_Lueur_Random") {
            item::unit_change_to_random_class(hero_unit);
            GameVariableManager::set_bool("G_Lueur_Random", true);
            person::adjust_unit_items(hero_unit);
        }
        // Liberation Weapon Change
        crate::item::change_liberation_type();
    }
    call_original!(group, method_info);
    if crate::utils::str_contains(GameUserData::get_chapter().cid, "CID_S0") && GameVariableManager::get_number("G_Emblem_Mode") != 0 { get_emblem_paralogue_level(); }
    if !GameVariableManager::get_bool("G_Cleared_M003") {return; }
    let player_force = Force::get(ForceType::Player).unwrap();
    let max_player = player_force.get_count();
    let mut player_count;
    let absent_count = absent_force.get_count();
    let rng = Random::get_game();

    if GameVariableManager::get_number("G_DeploymentMode") == 3 && !GameUserData::is_encount_map() { 
        if hero_unit.status.value & 20 != 0 { hero_unit.status.value -= 20; }
        hero_unit.status.value = hero_unit.status.value | 1073741832;
        if GameVariableManager::get_number("G_EmblemDeployMode")!= 0 {
            emblem_selection_menu_enable(false);
            unsafe {remove_all_rings(0, None); }
        }
        return;
    }
    unsafe {
        if absent_count == 0 || GameUserData::is_evil_map() { 
            unit_selection_menu_enable(true);
            emblem_selection_menu_enable(true);
            if GameUserData::is_evil_map() { return; }
        }
        if GameVariableManager::get_number("G_EmblemDeployMode")!= 0 && GameVariableManager::get_number("G_DeploymentMode")  == 0 {
            unit_selection_menu_enable(true);
            emblem_selection_menu_enable(false);
            remove_all_rings(0, None);
            return;
        }
        //Normal Deployment
        if GameVariableManager::get_number("G_DeploymentMode")  == 0 || absent_count == 0 {
            unit_selection_menu_enable(true);
            return;
        } 
        // Move currently deployed units to absent and then move back hero unit (Alear or Veyle)
        player_force.transfer(3, true);

        //Transfer Dead
        if GameVariableManager::get_number("G_DeploymentMode")  == 1 || GameVariableManager::get_number("G_DeploymentMode")  == 2 { Force::get(ForceType::Dead).unwrap().transfer(3, true); }
        let hero_unit = absent_force.get_hero_unit();
        hero_unit.transfer(0, true);
        hero_unit.try_create_actor();
        if !GameUserData::is_encount_map() && GameVariableManager::get_number("G_DeploymentMode") != 3 { hero_unit.set_status(20); }   
        player_count = player_force.get_count();
        // Lowest Rating Deployment
        if GameVariableManager::get_number("G_DeploymentMode") == 1 {
            unit_selection_menu_enable(false);
            emblem_selection_menu_enable(true);
            while player_count < max_player {
                let mut pid: &Il2CppString = "PID_unit".into();
                let mut mpid: &Il2CppString = "MPID_unit".into();
                let mut capability_score = 99999;
                let mut force_iter = Force::iter(absent_force);
                while let Some(unit) = force_iter.next() {
                    let cap = get_unit_rating(unit);
                    if cap < capability_score {
                        capability_score = cap;
                        pid = unit.person.pid;
                        mpid = unit.person.get_name().unwrap();
                    }
                }
                println!("{} is deployed with rating of {}", mpid.get_string().unwrap(), capability_score);
                let move_unit = force_get_unit_from_pid(pid, false, None);
                if move_unit.is_some() {
                    let unit = move_unit.unwrap();
                    unit.transfer(0, true);
                    unit.try_create_actor();
                }
                player_count = player_force.get_count();
            }
        }
        // Random Deployment
        else if GameVariableManager::get_number("G_DeploymentMode") == 2  {
            unit_selection_menu_enable(false);
            emblem_selection_menu_enable(true);
            while player_count < max_player {
                let rng_range = absent_force.get_count();
                let mut index = 0;
                let value = rng.get_value(rng_range);
                let mut force_iter = Force::iter(absent_force);
                while let Some(unit) = force_iter.next() {
                    if index == value {
                        unit.transfer(0, true);
                        unit.try_create_actor();
                        player_count = player_force.get_count();
                        break;
                    }
                    index += 1;
                }
            }
        }
    }
}
// Global Menu Stuff
pub struct DeploymentMod;
impl ConfigBasicMenuItemSwitchMethods for DeploymentMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){ }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = 
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number("G_DeploymentMode") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().deployment_type = result; }
            else {  GameVariableManager::set_number("G_DeploymentMode", result); } 
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else { return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let deploy_type =   
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number("G_DeploymentMode") };
        this.help_text = match deploy_type {
            1 => {"Lowest rating units will be deployed." },
            2 => {"Units will be deployed at random." }
            3 => {"No forced deployment restrictions."},
            _ => { "Normal Deployment"},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let deploy_type =   
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number("G_DeploymentMode") };
        this.command_text = match deploy_type { 
            1 => { "Lowest Rating" },
            2 => { "Random" },
            3 => { "Free"},
            _ => { "Default" },
        }.into();
    }
}

pub struct EmblemMod;
impl ConfigBasicMenuItemSwitchMethods for EmblemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = 
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number("G_EmblemDeployMode") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().emblem_deployment = result; }
            else { GameVariableManager::set_number("G_EmblemDeployMode", result) };
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let emblem_deployment = 
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number("G_EmblemDeployMode") };
        this.help_text = match emblem_deployment {
            1 => { "Emblems will be randomized onto deployed units. (Togglable)" },
            2 => { "Emblems will not be equipped onto units. (Togglable)" },
            _ => { "Emblems are freely selectable in battle preperations. (Togglable)"},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let emblem_deployment = 
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number("G_EmblemDeployMode") };
        this.command_text = match emblem_deployment { 
            1 => { "Random Emblems" },
            2 => { "No Emblems" },
            _ => { "Default"},
        }.into();
    }
}

#[skyline::from_offset(0x01c616f0)]
pub fn remove_all_rings(this: u64, method_info: OptionalMethod);

#[unity::from_offset("App","MapDispos", "GetSortieLimit")]
pub fn get_sortie_limit(method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x01c54fa0)]
pub fn force_get_unit_from_pid(pid: &Il2CppString, relay: bool, method_info: OptionalMethod) -> Option<&'static mut Unit>;

#[unity::class("App", "MapInspectors")]
pub struct MapInspectors {
    parent: [u8; 0x10],
    pub inspectors: &'static mut List<MapInspector>,
    pub kind_inspectors: &'static mut Array<&'static mut List<MapInspector>>,
}

#[unity::class("App", "MapInspector")]
pub struct MapInspector {
    pub kind: i32,
    __: i32,
    pub  m_condition: &'static DynValue,
    pub function: &'static DynValue,
    pub arg: &'static Array<&'static DynValue>,
    pub var1: i32,
    pub var2: i32,
    pub var3: i32,
    pub var4: i32,
    pub var5: i32,
    pub var6: i32,
}
impl MapInspectors {
    fn get_instance() -> &'static mut MapInspectors {
        let idk = get_generic_class!(SingletonClass<MapInspectors>).unwrap();
        let pointer = unsafe { &*(idk.rgctx_data as *const Il2CppRGCTXData as *const u8 as *const [&'static MethodInfo; 6]) };
        let get_instance =
            unsafe { std::mem::transmute::<_, extern "C" fn(OptionalMethod) -> &'static mut MapInspectors>(pointer[5].method_ptr) };
        get_instance(Some(&pointer[5]))
    }
}

pub fn adjust_map_inspectors() {
    if GameVariableManager::get_number("G_EmblemDeployMode") == 2  {
        emblem_selection_menu_enable(false);
        unsafe { remove_all_rings(0, None); }
    }
    else if GameVariableManager::get_number("G_EmblemDeployMode") == 1 {
        unsafe { remove_all_rings(0, None); }
        let emblem_list = get_emblem_list();
        let mut emblem_count = emblem_list.len();
        let mut set_emblems: [bool; 20] = [false; 20];
        let player_force = Force::get(ForceType::Player).unwrap();
        let max_player = player_force.get_count();
        if emblem_count > max_player as usize { emblem_count = max_player as usize; }
        let mut current_emblem_count = 0;
        let mut force_iter = Force::iter(player_force);
        let rng = Random::get_game();
        while let Some(unit) = force_iter.next() {
            let mut value = rng.get_value(emblem_list.len() as i32) as usize;
            while set_emblems[value] == true { value = rng.get_value(emblem_list.len() as i32) as usize;  }
            let god_unit = GodPool::try_get_gid(emblem_list[value], true).unwrap();
            unit.set_god_unit(god_unit);
            current_emblem_count += 1;
            set_emblems[value] = true;
            if current_emblem_count == emblem_count { break; } 
        }
    }
    else { Patch::in_text(0x01d77028).bytes(&[0xc0, 0x00, 0x00, 0x36]).unwrap();}

    if lueur_on_map() && GameVariableManager::get_number("G_DeploymentMode") == 3 { return; } // if alear is on map don't change anything 
    let inspectors = MapInspectors::get_instance();
    for x in 0..inspectors.inspectors.len() {
        let kind = inspectors.inspectors[x].kind;
        if kind == 9 {
            if inspectors.inspectors[x].var6 == 1 { inspectors.inspectors[x].var6 = -1; }
        }
        if kind == 18 || kind == 19 {
            if inspectors.inspectors[x].var1 == 1 { inspectors.inspectors[x].var1 = -1; } //new_person_index; }
        }
    }
    for x in 0..inspectors.kind_inspectors[9].len() {
        if inspectors.kind_inspectors[9].items[x as usize].var6 == 1 { inspectors.kind_inspectors[9].items[x as usize].var6 = -1; }
    }
    for x in 0..inspectors.kind_inspectors[18].len() {
        if inspectors.kind_inspectors[18].items[x as usize].var1 == 1 { inspectors.kind_inspectors[18].items[x as usize].var1 = -1; }
    }
    for x in 0..inspectors.kind_inspectors[19].len() {
        if inspectors.kind_inspectors[19].items[x as usize].var1 == 1 { inspectors.kind_inspectors[19].items[x as usize].var1 = -1; }
    }
}