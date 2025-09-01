use std::sync::OnceLock;
use skyline::patching::Patch;
pub use engage::{
    resourcemanager::ResourceManager,
    unitpool::UnitPool,
    menu::{
        BasicMenuItemAttribute,
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    mess::*,
    gamevariable::*, gameuserdata::*, hub::access::*, random::*,
    gamedata::{*, item::*, skill::SkillData, dispos::*, unit::*},
    spriteatlasmanager::SpriteAtlasManager,
};
use unity::il2cpp::object::Array;
use crate::{enums::*, utils::*, autolevel::*};
use super::{DVCVariables, CONFIG, RANDOMIZER_STATUS};

pub mod ai;
pub mod unit; 
pub mod hub;
pub mod custom;

pub static mut SET: i32 = 0;
pub static PLAYABLE: OnceLock<Vec<i32>> = OnceLock::new();
pub static ENEMY_PERSONS: OnceLock<Vec<(i32, i32)>> = OnceLock::new();
pub static PERSONS_LIST: OnceLock<Vec<(i32, i32, String)>> = OnceLock::new();

pub struct RandomPersonMod;
pub struct RandomBosses;

impl ConfigBasicMenuItemSwitchMethods for RandomPersonMod {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.get_class_mut()
            .get_virtual_method_mut("ACall")
            .map(|method| method.method_ptr = custom::crecruitment_menu_a_call as _).unwrap();
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().random_recruitment;
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            CONFIG.lock().unwrap().random_recruitment = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_recruitment {
            1 => { "Random"},
            3 => { "Custom Order (A)"},
            2 => { "Reverse"},
            _ => { "Standard"},
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_recruitment {
            1 => { "Characters will be recruited in a random order." },
            3 => { "Unit recruitment order is determined by list. (Press A)"},
            2 => { "Characters will be recruited in reversed order."}
            _ => { "Standard recruitment order." },
        }.into();
    }
}

impl ConfigBasicMenuItemSwitchMethods for RandomBosses  {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry(DVCVariables::RANDOM_BOSS_KEY, 0);
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_main = DVCVariables::is_main_menu();
        let value = if is_main { CONFIG.lock().unwrap().bosses }
        else { GameVariableManager::get_bool(DVCVariables::RANDOM_BOSS_KEY) };

        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if is_main { CONFIG.lock().unwrap().bosses = result }
            else { GameVariableManager::set_bool(DVCVariables::RANDOM_BOSS_KEY, result) };

            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().bosses }
        else { GameVariableManager::get_bool(DVCVariables::RANDOM_BOSS_KEY) };
        this.command_text = if value { "Random" } else { "Default"}.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().bosses }
        else { GameVariableManager::get_bool(DVCVariables::RANDOM_BOSS_KEY) };
        this.help_text = if value { "NPCs and enemy bosses may look a bit different." }
        else { "Default behavior."}.into();
    }
}
// Custom Person Mod
pub struct CustomPersonMod;
impl ConfigBasicMenuItemSwitchMethods for CustomPersonMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().custom_units);
        if CONFIG.lock().unwrap().custom_units != result {
            CONFIG.lock().unwrap().custom_units = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().custom_units  { "Include" }
            else { "Default"}.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().custom_units { "Custom units are include in random recruitment order." }
            else { "Custom units will excluded from random recruitment order." }.into();
    }
}
fn build_attribute_custom_units(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if PLAYABLE.get().unwrap().len() == 41 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

pub struct CustomPersonRecruitDisable;
impl ConfigBasicMenuItemSwitchMethods for CustomPersonRecruitDisable {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if PLAYABLE.get().unwrap().len() > 94 {
            this.help_text = "Added recruitment slots are disabled. (Exceeds unit limit)".into();
            this.command_text = "Disable".into();
            this.update_text();
            return BasicMenuResult::new();
        }
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().custom_unit_recruitment_disable);
        if CONFIG.lock().unwrap().custom_unit_recruitment_disable != result {
            CONFIG.lock().unwrap().custom_unit_recruitment_disable = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else {BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().custom_unit_recruitment_disable  { "Disable" }
            else { "Enable"}.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().custom_unit_recruitment_disable { "Added Somniel recruitment slots will be disabled." }
            else { "Added Somniel recruitment slots will be enabled." }.into();
    }
}

pub extern "C" fn vibe_custom_units() -> &'static mut ConfigBasicMenuItem { 
    let item = ConfigBasicMenuItem::new_switch::<CustomPersonMod>("Custom Units");
    item.get_class_mut().get_virtual_method_mut("BuildAttribute").
        map(|method| method.method_ptr = build_attribute_custom_units as _);
    item
}
pub extern "C" fn vibe_custom_slot_disable() -> &'static mut ConfigBasicMenuItem { 
    let item = ConfigBasicMenuItem::new_switch::<CustomPersonRecruitDisable>("Added Recruitment Slots");
    item.get_class_mut().get_virtual_method_mut("BuildAttribute")
        .map(|method| method.method_ptr = build_attribute_custom_units as _);
    item
}
pub fn get_playable_list() {
    // Add the 41 units first
    PLAYABLE.get_or_init(||{
        let mut list: Vec<i32> = Vec::new();
        let mut hashes: Vec<i32> = Vec::new();
        PIDS.iter().for_each(|&pid| {
            let person = PersonData::get(pid).unwrap();
            list.push(person.parent.index);
            hashes.push(person.parent.hash);
        });
        // Add all others that have non-zero SP
        let person_list = PersonData::get_list().unwrap(); 
        let count = person_list.iter().filter(|p| p.get_sp() > 0 ).count();
        if count < 200 { 
            person_list.iter().filter(|p| 
                !p.pid.str_contains("_竜化") &&
                p.parent.index > 1 && 
                p.get_sp() > 0 && 
                p.get_asset_force() == 0
            )
            .for_each(|person|{
                if !hashes.iter().any(|&hash| hash == person.parent.hash ) {
                    list.push(person.parent.index);
                    hashes.push(person.parent.hash);
                }
            });
        }
        if count < 95 { println!("Total of {} Playable Units", list.len()); }
        else { println!("Total of {} Possible Playable Units", list.len()); }

        list
    });
    let extras = [
        "PID_道具屋", "PID_アクセ屋", "PID_イヴ", "PID_モリオン", "PID_スフォリア", "PID_S001_ジャン_母親",
        "PID_S001_ジャン_父親", "PID_E003_召喚_マルス", "PID_錬成屋", "PID_武器屋",
        "PID_M010_異形兵_モリオン", "PID_M017_異形兵_ハイアシンス", "PID_M003_イルシオン兵_ボス", "PID_M005_Irc_ボス",
        "PID_M006_ボス", "PID_M013_蛮族_お頭Ａ", "PID_M013_蛮族_お頭Ｂ", "PID_S002_蛮族_お頭", "PID_E001_イル",
        "PID_E001_Boss",
    ];

    let filter = ["モーヴ", "_竜化", "M0", "E00", "G00", "村人", "召喚_", "S0", "幻影兵", "ン兵", "異形兵", "PID_残像"];
    PERSONS_LIST.get_or_init(||{
        PersonData::get_list().unwrap().iter()
           .filter(|p|{
               let pid = p.pid.to_string();
               !PIDS.iter().any(|&p| pid.contains(&p[3..])) && p.get_bmap_size() == 1 &&
               p.gender & 3 != 0 && (
                   extras.iter().any(|ex| &pid == ex ) ||  (
                       !filter.iter().any(|&filter| pid.contains(filter)) &&
                       !p.name.is_some_and(|c| {
                           let name = c.to_string();
                           name.contains("Soldier") || name.contains("Villager") || name.contains("Phantom")
                       || name.contains("Bandits") || name.contains("Morph")})
                           && !p.get_job().is_some_and(|j| j.jid.str_contains("JID_紋章士")) &&
                       !extras.iter().any(|&ex| pid.contains(&ex[4..])) && (
                           p.name.is_some_and(|name| name.str_contains("Boss")) ||
                           p.get_combat_bgm().is_some_and(|bgm| bgm.str_contains("Boss"))
                       )
                   )
               )
           })
           .map(|p| (p.parent.hash, p.gender, p.name.unwrap().to_string()))
           .collect()
    });
    println!("NPC Persons List: {}", PERSONS_LIST.get().unwrap().len());
}
pub fn is_playable_person(person: &PersonData) -> bool { PLAYABLE.get().unwrap().iter().any(|&x| person.parent.index == x) }
pub fn check_playable_classes() {
    let list = PLAYABLE.get().unwrap();
    list.iter().for_each(|&index|{
        if let Some(person) = PersonData::try_index_get_mut(index) {
            if person.get_job().is_none() {
                if person.get_sp() >= 1000 || person.get_internal_level() > 0 { person.set_jid("JID_ソードマスター".into()); }
                else {  person.set_jid("JID_ソードファイター".into()); }
                person.on_completed();
            }
        }
    });
}

fn get_custom_recruitment_list() -> Vec<(i32, i32)> {   // person_x to person_y
    let mut output: Vec<(i32, i32)> = Vec::new();
    let table = custom::CUSTOM_RECRUITMENT_TABLE.lock().unwrap();
    let limit = if dlc_check() { 41 } else { 36 };
    let mut available: Vec<i32> = (0..limit).collect();
    let mut pool: Vec<i32> = Vec::new();
    for x in 0..limit {
        let value = table[x as usize];
        if table[x as usize] != 0 {
            output.push( (x, value - 1) );
            if let Some(pos) = available.iter().position(|&y| value - 1 == y) {
                available.remove(pos);
            }
        }
        else { pool.push(x); }
    }
    let rng = get_rng();
    [0, 4, 14, 17, 23, 27 ].iter().for_each(|&x_lord|{ 
        if let Some(pos) = pool.iter().position(|&xi| xi == x_lord){
            if available.len() > 0 {
                loop {  // Making sure lords do not get randomized with DLC Characters
                    let index = rng.get_value( available.len() as i32) as usize;
                    let xj = available[index];
                    if xj < 36 {
                        output.push( (x_lord, xj) );
                        available.remove(index);
                        break;
                    }
                }
                pool.remove(pos);
            }
        }
    });
    pool.iter().for_each(|&xi|{
        if available.len() > 0 {
            let index = rng.get_value( available.len() as i32) as usize;
            let xj = available[index];
            output.push( (xi, xj) );
            available.remove(index);
        }
    });
    output
}

fn set_hub_facilities() {
    let aid = ["AID_蚤の市", "AID_筋肉体操", "AID_ドラゴンライド", "AID_釣り", "AID_占い小屋"];
    let locator = ["LocatorSell01", "LocatorTraining01", "LocatorDragon01", "LocatorFish01", "LocatorFortune01"];
    let index = [ 23, 4, 17, 14, 27];
    let hub_dispos = HubDisposData::get_array_mut().unwrap();
    for x in 0..aid.len() {
        let data = HubFacilityData::get_mut(aid[x]);
        let pid = PIDS[index[x] as usize];
        let a_index = pid_to_index(&pid.to_string(), true) as usize;
        if data.is_some() {
            let facility = data.unwrap();
            facility.condition_cid = format!("CID_{}", RECRUIT_CID[a_index] ).into() ;
            for y in 0..hub_dispos[1].len() {
                let hub_locator = hub_dispos[1][y].get_locator();
                if hub_locator.to_string() == locator[x] {
                    hub_dispos[1][y].set_chapter(RECRUIT_CID[a_index].into() );
                    break;
                }
            }
        }
    }
}
pub fn randomize_person() {
    if !can_rand() { return; }
    if !GameVariableManager::exist("G_Random_Person_Set") {  GameVariableManager::make_entry("G_Random_Person_Set", 0);  }
    if GameVariableManager::get_bool("G_Random_Person_Set") { 
        if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 { 
            set_hub_facilities(); 
            hub::change_somniel_hub_dispos();
        }
        return; 
    }
    else {
        let rng = get_rng();
        match GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) {
            1 => {
                let playable_size = if CONFIG.lock().unwrap().custom_units && PLAYABLE.get().unwrap().len() > 41 { PLAYABLE.get().unwrap().len() } else { 41 };
        
                let list = PLAYABLE.get().unwrap();
                let mut playable_list: Vec<usize> = (0..playable_size).collect();
                let mut to_replace_list: Vec<usize> = (0..playable_size).collect();
                if !dlc_check() || CONFIG.lock().unwrap().dlc & 2 != 0 { 
                    for x in 36..41 {   // Remove DLC characters in the pool
                        if let Some(index) = playable_list.iter().position(|&i| i == x) {  playable_list.remove(index);  }
                        if let Some(index) = to_replace_list.iter().position(|&i| i == x) {  to_replace_list.remove(index);  }
                    }
                }
                let person_list = PersonData::get_list().unwrap();
                let pids: Vec<String> = list.iter().map(|&x| person_list[x as usize].pid.to_string() ).collect();
                pids.iter().for_each(|pid|{
                    let key = format!("G_R_{}", pid.as_str());
                    GameVariableManager::make_entry_str(key.as_str(), pid.as_str());
                    let key2 = format!("G_R2_{}", pid.as_str());
                    GameVariableManager::make_entry_str(key2.as_str(), pid.as_str());
                });

                let set_recruitment = SET_RECRUITMENT.lock().unwrap();                
                set_recruitment.iter().filter(|x| x.1 == -1).map(|x| x.0 as usize).for_each(|index|{
                    if let Some(remove) = playable_list.iter().position(|&i| i == index) {  playable_list.remove(remove);  }
                    if let Some(remove) = to_replace_list.iter().position(|&i| i == index) {  to_replace_list.remove(remove);  }
                });

                println!("Playable Unit Size: {}", playable_list.len());
            // Alear and somniel royals must be switched with non-dlc units 
            //  x_i in to_replace, x_j in playable_list, royals are x_i
            //  x_j -> x_i, remove royal (x_i) from to_replace and remove x_j from playable_list
                let royals = [0, 23, 4, 17, 14, 27];
                for x_royal in royals {
                    if let Some(index_royal) = playable_list.iter().position(|&i| i == x_royal ){  // royal is no longer in the available pool, skip
                        loop {
                            let index_j =  rng.get_value(to_replace_list.len() as i32) as usize;
                            let x_j = to_replace_list[ index_j ];
                            if x_j > 35 || x_j == 30 { continue; }  // If DLC/Linden try again
                            DVCVariables::set_person_recruitment(x_j as i32, x_royal as i32);
                            to_replace_list.remove(index_j);  // Remove the unit who becomes the royal from the list of units to replace
                            playable_list.remove(index_royal); // Remove Royal from Pool from the playable list to replace to
                            println!("#{}: {} -> {}", x_j, Mess::get_name(PIDS[x_j]),  Mess::get_name(PIDS[x_royal]));
                            break;
                        }
                    }
                }
                to_replace_list.iter().for_each(|&x_i|{
                    let key_pid_x = format!("G_R_{}", pids[x_i].as_str());
                    let pool_size = playable_list.len() as i32;
                    if pool_size > 0 {
                        let index_j = rng.get_value(pool_size) as usize;
                        let x_j = playable_list[ index_j ];
                        GameVariableManager::set_string(key_pid_x.as_str(), pids[x_j].as_str() );
                        let key_pid_j = format!("G_R2_{}", pids[x_j]);
                        GameVariableManager::set_string(key_pid_j.as_str(), pids[x_i].as_str());
                        playable_list.remove(index_j);
                        println!("#{}: {} -> {}", x_i, Mess::get_name(pids[x_i].as_str()),  Mess::get_name(pids[x_j].as_str()));
                    }
                }); 
            },
            2 => {   //Reverse
                for x in 0..41 { DVCVariables::set_person_recruitment(x, RR_ORDER[x as usize] as i32); }
            },
            3 => { // Custom
                get_custom_recruitment_list().iter().for_each(|&x|{
                    DVCVariables::set_person_recruitment(x.0, x.1);
                    println!("{} -> {}", Mess::get_name(PIDS[x.0 as usize]), Mess::get_name(PIDS[x.1 as usize]));
                });
            },
            _ => {},
        }
    }
    GameVariableManager::set_bool("G_Random_Person_Set", true);
    set_hub_facilities(); 
    hub::change_somniel_hub_dispos();
}
pub fn find_pid_replacement(pid: &String, reverse: bool) -> Option<String>{
    if PIDS.iter().position(|&x| x == *pid).is_some() || EMBLEM_GIDS.iter().position(|&x| x == *pid).is_some() {
        if reverse { Some(GameVariableManager::get_string(&format!("G_R2_{}", pid)).to_string()) }
        else { Some(GameVariableManager::get_string(&format!("G_R_{}", pid)).to_string()) }
    }
    else { None }
}

pub fn change_map_dispos() {
    let list = DisposData::get_list_mut();
    if list.is_none() || !can_rand() { return; }
    let t_list = list.unwrap();
    println!("[DVC] Map Dispos Check");
    // Framme and Clanne Replacement
    if !DVCVariables::is_main_chapter_complete(3) { GameVariableManager::make_entry("DDFanClub", 1); }
    t_list.iter_mut().flat_map(|array| array.iter()).for_each(|dispos| {
        if let Some(gid) = dispos.gid {
            let key = format!("G_R_{}", gid);
            if GameVariableManager::exist(key.as_str()) { dispos.set_gid(GameVariableManager::get_string(key.as_str())); }
        }
    });
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return; }
    t_list.iter_mut().flat_map(|array| array.iter())
        .filter(|dispos|
            ((1 << dispos.force) & 5 != 0) &&
                dispos.get_pid().is_some_and(|pid| pid.str_contains("PID_"))
        )
        .for_each(|dispos| {
            let pid = dispos.pid.to_string();
            if pid == PIDS[0] { dispos.set_pid(DVCVariables::get_dvc_person(0, false)); }
            else if GameVariableManager::get_bool("DDFanClub") && GameVariableManager::exist(&format!("G_R_{}", pid))
            {
                dispos.set_pid(GameVariableManager::get_string(&format!("G_R_{}", pid)));
            }
        });
}



pub fn change_lueur_for_recruitment(is_start: bool) {
    if !DVCVariables::random_enabled() || RANDOMIZER_STATUS.read().unwrap().alear_person_set { return; }
    if !GameVariableManager::exist("G_R_PID_リュール") ||GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return; }
    if DVCVariables::get_dvc_person(0, false).to_string() == PIDS[0] {
        let _ = RANDOMIZER_STATUS.try_write().map(|mut lock| lock.alear_person_set = true);
        return;
     }
    // remove hero status on alear and place it on the replacement and add alear skills on the replacement
    let person_lueur = PersonData::get(PIDS[0]).unwrap();
    let lueur_sids = person_lueur.get_common_sids().unwrap();
    if let Some(hero_sid) = lueur_sids.iter_mut().find(|x| x.to_string().contains("SID_主人公")) {
        *hero_sid =  "SID_無し".into();
    }
    person_lueur.on_complete();
    let new_hero = switch_person(person_lueur);
    if let Some(hero) = UnitPool::get_from_person_force_mask(&new_hero, -1) {
        hero.private_skill.add_sid("SID_主人公", 10, 0);
        hero.private_skill.add_sid("SID_王族", 10, 0);
        hero.private_skill.add_sid("SID_リベラシオン装備可能", 10, 0);
        hero.private_skill.add_sid("SID_ヴィレグランツ装備可能", 10, 0);
    }
    let sids = new_hero.get_common_sids().unwrap();
    let new_sids = Array::<&Il2CppString>::new_specific( sids.get_class(), sids.len()+4).unwrap();
    for x in 0..sids.len() { new_sids[x] = sids[x]; }
    new_sids[sids.len() ] = "SID_主人公".into();
    new_sids[sids.len() + 1 ] = "SID_リベラシオン装備可能".into();
    new_sids[sids.len() + 2 ] = "SID_ヴィレグランツ装備可能".into();
    new_sids[sids.len() + 3 ] = "SID_王族".into();
    new_hero.set_common_sids(new_sids);
    new_hero.on_complete();
    if is_start {   // Move alear to force 5
        if let Some(lueur_unit) = UnitPool::get_from_person_mut(PIDS[0].into(), false) {
            unit::change_unit_autolevel(lueur_unit, true);
            if GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0 {
                super::job::unit_change_to_random_class(lueur_unit);
                unit::fixed_unit_weapon_mask(lueur_unit);
                unit::adjust_unit_items(lueur_unit);
            }
            lueur_unit.transfer(5, false); 
            get_lueur_name_gender(); // grab gender and name
            GameVariableManager::make_entry(DVCVariables::LUEUR_GENDER, lueur_unit.edit.gender);
        }
        if let Some(unit) = unsafe { join_unit(new_hero, None) }{
            unit.edit.set_name( Mess::get( new_hero.get_name().unwrap()) );
            unit.edit.set_gender( new_hero.get_gender() );
            unit.private_skill.add_sid("SID_主人公", 10, 0);
            unit.private_skill.add_sid("SID_王族", 10, 0);
            unit.private_skill.add_sid("SID_リベラシオン装備可能", 10, 0);
            unit.private_skill.add_sid("SID_ヴィレグランツ装備可能", 10, 0);
            unit.transfer(3, false);
        }
    }
    Patch::in_text(0x02d524e0).nop().unwrap();
    Patch::in_text(0x02d524e4).nop().unwrap();

    // LueurW_God or Lueur_God in GetPath 
    if GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 {  
        Patch::in_text(0x02d524e8).bytes(&[0x48, 0x00, 0x80, 0x52]).unwrap(); 
       person_lueur.set_gender(2);
    }
    else { 
        Patch::in_text(0x02d524e8).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap();
        person_lueur.set_gender(1);
    }

    Patch::in_text(0x0233f104).bytes(&[0x01,0x10, 0x80, 0x52]).unwrap(); // GodUnit$$GetName ignore hero flag on Emblem Alear
    let lueur_god_offsets = [0x02d51dec, 0x021e12ac, 0x02915844, 0x02915844, 0x02915694, 0x01c666ac,0x02081edc, 0x01c69d60, 0x01c66588];
    for x in lueur_god_offsets { mov_x0_0(x); }

    // For Hub-Related Activities
    let offsets = [0x02ae8d28, 0x02ae9000, 0x02a5d0f4, 0x01cfd4c4, 0x01d03184, 0x01e5fe00, 0x01e5ff4c, 0x027049c8];
    let new_hero_gender = if new_hero.get_gender() == 2 || (new_hero.get_gender() == 1 && new_hero.get_flag().value & 32 != 0 ) { 2 } else { 1 };
    for x in offsets {
        if new_hero_gender == 1 {  mov_x0_0(x); }
        else { mov_1(x); }
    }
    if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() { 
        lock.alear_person_set = true; 
        lock.set_enable();
    }
}


pub fn pid_to_index(pid: &String, reverse: bool) -> i32 {
    if let Some(replacement) = find_pid_replacement(pid, reverse) {
        if let Some(found_pid) = PIDS.iter().position(|&x| x == replacement) { return found_pid as i32; }
        if let Some(found_gid) = EMBLEM_GIDS.iter().position(|&x| x == replacement).filter(|x| *x < 19) { return found_gid as i32;  }
    }
    -1  // to cause crashes
}

pub fn get_low_class_index(this: &PersonData) -> usize {
    let apt = this.get_aptitude().value;
    for x in 0..3 { if apt & (1 << (x+1) ) != 0 { return x; } }
    let apt2 = this.get_sub_aptitude().value;
    for x in 0..3 { if apt2 & (1 << (x+1) ) != 0 { return x; } }
    0
}

pub fn switch_person(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.to_string();
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return PersonData::get(&pid).unwrap(); }
    let var_str = format!("G_R_{}", pid);
    let new_pid = GameVariableManager::get_string(&var_str);
    unsafe { if is_null_empty(new_pid, None) { return PersonData::get(&pid).unwrap(); } }
    if let Some(new_person) = PersonData::get(&new_pid.to_string()) { new_person } else { PersonData::get(&pid).unwrap() }
}
pub fn switch_person_reverse(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.to_string();
    let reverse = GameVariableManager::get_string(&format!("G_R2_{}", pid));
    PersonData::get(reverse).unwrap()
}

// Handle the case of Chapter 11 ends with not escape
pub fn m011_ivy_recruitment_check(){
    if !DVCVariables::random_enabled() || GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return; }
    if GameUserData::get_chapter().cid.to_string() == "CID_M011" && lueur_on_map() {
        GameVariableManager::make_entry("MapRecruit", 1);
        GameVariableManager::set_bool("MapRecruit", true);
    }
}
pub fn lueur_recruitment_check() {
    if let Some(lueur) = UnitPool::get_from_person_force_mask(PersonData::get(PIDS[0]).unwrap(), 6){
        if lueur.force.is_some_and(|f| ( GameUserData::get_chapter().cid.str_contains("M018") && f.force_type == 1 ) || f.force_type == 2) {
            if GameUserData::get_sequence() == 3 { lueur.transfer(0, true); }
            else if GameUserData::get_sequence() == 5 { lueur.transfer(3, true); }
        }
    }
}

pub fn get_all_enemy_persons() {
    ENEMY_PERSONS.get_or_init(|| {
        include_str!("./person/persons.txt").lines()
            .map(|l|
                {
                    let s: Vec<_> = l.split_ascii_whitespace().collect();
                    (s[0].parse::<i32>().unwrap_or(0), PersonData::get_index(s[1]))
                }
            ).collect::<Vec<(i32, i32)>>()
    });
    println!("Person Enemy Count: {}", ENEMY_PERSONS.get().unwrap().len());
}

#[skyline::from_offset(0x01c73960)]
fn join_unit(person: &PersonData, method_info: OptionalMethod) -> Option<&'static mut Unit>;



