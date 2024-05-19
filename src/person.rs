use unity::prelude::*;
use engage::{
    menu::{
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    gamevariable::*,
    gameuserdata::*,
    hub::access::*,
    random::*,
    mess::*,
    gamedata::{*, skill::*, item::*, dispos::*, unit::*},
};

use crate::{emblem::*, skill::{EIRIKA_INDEX}, utils::*, deploy, item, autolevel::*};
use super::CONFIG;
pub static mut RAND_PERSONS: [i32; 82] = [0; 82];

pub const PIDS : &[&str] = &["PID_リュール", "PID_ヴァンドレ", "PID_クラン", "PID_フラン", "PID_アルフレッド", "PID_エーティエ", "PID_ブシュロン", "PID_セリーヌ", "PID_クロエ", "PID_ルイ", "PID_ユナカ", "PID_スタルーク", "PID_シトリニカ", "PID_ラピス", "PID_ディアマンド", "PID_アンバー", "PID_ジェーデ", "PID_アイビー", "PID_カゲツ", "PID_ゼルコバ", "PID_フォガート", "PID_パンドロ", "PID_ボネ", "PID_ミスティラ", "PID_パネトネ", "PID_メリン", "PID_オルテンシア", "PID_セアダス", "PID_ロサード", "PID_ゴルドマリー", "PID_リンデン", "PID_ザフィーア", "PID_ヴェイル", "PID_モーヴ", "PID_アンナ", "PID_ジャン", "PID_エル", "PID_ラファール", "PID_セレスティア", "PID_グレゴリー", "PID_マデリーン"];
const RECRUIT_CID : [&str; 41] = ["M001", "M001", "M002", "M002", "M003", "M003", "M003", "M004", "M004", "M004", "M006", "M007", "M007", "M007", "M008", "M008", "M009", "M011", "M011", "M011", "M012", "M012", "M012", "M013", "M013", "M013", "M014", "M015", "M016", "M016", "M018", "M019", "M022", "M021", "S002", "S001", "E006", "E006", "E006", "E006", "E006"];
pub const RINGS: [&str; 19] = ["Marth", "Siglud", "Celica", "Micaiah", "Roy", "Leaf", "Lucina", "Lin", "Ike", "Byleth", "Kamui", "Eirik", "Edelgard", "Tiki", "Hector", "Veronica", "Senerio", "Camilla", "Chrom" ];
pub static mut SET: i32 = 0;
use std::sync::Mutex;
pub static PLAYABLE: Mutex<Vec<i32>> = Mutex::new(Vec::new());

pub fn get_playable_list() {
    if PLAYABLE.lock().unwrap().len() != 0 { return; }
    let mut list = PLAYABLE.lock().unwrap();
    // Add the 41 units first
    for x in PIDS {
        let index = PersonData::get(&x).unwrap().parent.index;
        list.push(index);
    }
    // Add all others that have non zero SP
    let person_list = PersonData::get_list().unwrap(); 
    for x in 0..person_list.len() { 
        let person = &person_list[x as usize];
        let index = person.parent.index; 
        if person.get_sp() > 0 {
            if list.iter().find(|r| **r == index).is_none() {
                list.push(index);
                let string; 
                if person.get_name().is_some() { string = person.get_name().unwrap().get_string().unwrap(); }
                else { string = " --- ".into(); }
                println!("Added Person #{}: {}: {} has SP > 0", index, person.pid.get_string().unwrap(), string);
            }
        }
    }
    println!("Total of {} Playable Units", list.len());
}

pub fn randomize_person() {
    if !GameVariableManager::get_bool("G_Random_Recruitment") {
        unsafe { 
            for i in 0..41 { 
                RAND_PERSONS[i as usize] = i; 
                RAND_PERSONS[41 + i as usize] = i; 
                let string = format!("G_R_{}",PIDS[i as usize]);
                GameVariableManager::make_entry_str(&string, PIDS[i as usize]);
            } 
        }
    }
    else {
        unsafe {
            let mut emblem_list_size: i32 = 36;
            if has_content(0, None) { emblem_list_size = 41; }
            for i in 0..41 { 
                RAND_PERSONS[i as usize] = i; 
                RAND_PERSONS[41 + i as usize] = i;
                let string = format!("G_R_{}",PIDS[i as usize]);
                GameVariableManager::make_entry_str(&string, PIDS[i as usize]);
            }
            let rng = Random::instantiate().unwrap();
            let seed = GameVariableManager::get_number("G_Random_Seed") as u32;
            rng.ctor(seed);
            let mut emblem_count: i32 = 1;
            let mut set_emblems: [bool; 41] = [false; 41];
            set_emblems[0] = true;
            while emblem_count < emblem_list_size {
                let index = rng.get_value(emblem_list_size);
                if index >= emblem_list_size { continue; }
                if emblem_count == 1 && (index == 2 || index == 3) { continue; }
                if ( emblem_count == 2 && index == 3 ) ||  ( emblem_count == 3 && index == 2 ) { continue; }
                if !set_emblems[index as usize] {
                    let string = format!("G_R_{}",PIDS[emblem_count as usize]);
                    GameVariableManager::set_string(&string, PIDS[index as usize]);
                    RAND_PERSONS[ emblem_count as usize ] = index;
                    RAND_PERSONS[ ( index + 41 ) as usize ] = emblem_count;
                    println!("{} -> {}", PersonData::get(PIDS[ emblem_count as usize]).unwrap().get_name().unwrap().get_string().unwrap(),
                    PersonData::get(PIDS[ index as usize]).unwrap().get_name().unwrap().get_string().unwrap());
                    set_emblems[ index as usize ] = true;
                    emblem_count += 1;
                }
            }
            let aid = ["AID_蚤の市", "AID_筋肉体操", "AID_ドラゴンライド", "AID_釣り", "AID_占い小屋"];
            let locator = ["LocatorSell01", "LocatorTraining01", "LocatorDragon01", "LocatorFish01", "LocatorFortune01"];
            let index = [ 23, 4, 17, 14, 27];
            let hub_dispos = HubDisposData::get_array_mut().unwrap();
            for x in 0..aid.len() {
                let data = HubFacilityData::get_mut(aid[x as usize]);
                let a_index = 41 + index[ x as usize] as usize;
                if data.is_some() {
                    let facility = data.unwrap();
                    facility.condition_cid = format!("CID_{}",   RECRUIT_CID[ RAND_PERSONS[ a_index ] as usize] ).into() ;
                    for y in 0..hub_dispos[1].len() {
                        let hub_locator = hub_dispos[1][y as usize].get_locator();
                        if hub_locator.get_string().unwrap() == locator[ x as usize] {
                            hub_dispos[1][y as usize].set_chapter(RECRUIT_CID[ RAND_PERSONS[ a_index ] as usize].into() );
                            break;
                        }
                    }
                }
            }
        }
        change_hub_dispos(false);
    }
}
pub fn find_pid_replacement(pid: &String, reverse: bool) -> Option<&str>{
    //let mut index = 0;
    let found_pid = PIDS.iter().position(|&x| x == *pid); 
    if found_pid.is_some() {
        let index: usize;
        if reverse { index = found_pid.unwrap() + 41; }
        else { index = found_pid.unwrap(); }
        unsafe {
            return Some(PIDS[ RAND_PERSONS [ index ] as usize ]); 
        }

    }
    let found_gid = deploy::EMBLEM_GIDS.iter().position(|&x| x == *pid);
    if found_gid.is_some() {
        let index: usize;
        if reverse { index = found_gid.unwrap() + 19; }
        else { index = found_gid.unwrap(); }
        unsafe {
            return Some(deploy::EMBLEM_GIDS[ RANDOMIZED_INDEX [ index as usize] as usize ]);
        }

    }
    return None;
    /* 
    for z in PIDS { // Vander
        if *z == pid {
            unsafe {
                if reverse { return Some(PIDS[ RAND_PERSONS [ index + 41 ] as usize ]); }
                else { return Some(PIDS[ RAND_PERSONS [ index ] as usize ]); }
            }
        }
        index += 1;
    }
    index = 0;
    for z in deploy::EMBLEM_GIDS {
        if *z == pid {
            unsafe {
                if reverse { return Some(deploy::EMBLEM_GIDS [ RANDOMIZED_INDEX [ index + 19] as usize ]); }
                else { return Some(deploy::EMBLEM_GIDS[ RANDOMIZED_INDEX [ index as usize] as usize ]); }
            }
        }
        index += 1;
    }
    return None; */
}
pub fn pid_to_mpid(pid: &String) -> String {
    return PersonData::get(&pid).unwrap().get_name().unwrap().get_string().unwrap();
}
pub fn change_hub_dispos(revert: bool) {
    let t_list = HubDisposData::get_array_mut().expect("Me");
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_aid();
            if aid.is_some() { 
                let pid = aid.unwrap().get_string().unwrap();
                let new_pid = find_pid_replacement(&pid, revert);
                if new_pid.is_some() { 
                    let n_pid = new_pid.unwrap();
                    t_list[x][y].set_aid(n_pid.into());
                 }
            }
        }
    }
}
pub fn change_map_dispos() {
    let t_list = DisposData::get_list_mut().expect("Me");
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_pid();
            if aid.is_some() && ( ( t_list[x][y].get_flag().value & 8 ) == 0 && t_list[x][y].get_force() == 0 ) {
                let pid = aid.unwrap().get_string().unwrap();
                let mut index = 0; //Resetting to Normal 
                for z in PIDS {
                    if *z == pid {
                        unsafe {
                            let new_pid = PIDS[ RAND_PERSONS [ 41 + index ] as usize ].into();
                            t_list[x][y].set_pid(new_pid);
                            break;
                        }
                    }
                    index += 1;
                }
                index = 0; // New One
                for z in PIDS {
                    if *z == pid {
                        unsafe {
                            let new_pid = PIDS[ RAND_PERSONS [ index ] as usize ].into();
                            t_list[x][y].set_pid(new_pid);
                            break;
                        }
                    }
                    index += 1;
                }
            }
        }
    }
}
pub fn get_low_class_index(this: &PersonData) -> usize {
    let apt = this.get_aptitude();
    if apt.value & 2 == 2 { return 0;  }
    else if apt.value & 4 == 4 { return 1; }
    else if apt.value & 8 == 8 { return 2; }

    let apt2 = this.get_sub_aptitude();
    if apt2.value & 2 == 2 { return 0; }
    else if apt2.value & 4 == 4 { return 1; }
    else if apt2.value & 8 == 8 { return 2; }
    return 0;
}

pub fn switch_person(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.get_string().unwrap();
    if !GameVariableManager::get_bool("G_Random_Recruitment") { return PersonData::get(&pid).unwrap(); }
    let var_str = format!("G_R_{}", pid);
    let new_pid = GameVariableManager::get_string(&var_str);
    unsafe { if is_null_empty(new_pid, None) { return PersonData::get(&pid).unwrap(); } }
    let new_person = PersonData::get(&new_pid.get_string().unwrap());
    if new_person.is_some() { return new_person.unwrap(); }
    else { return PersonData::get(&pid).unwrap(); }
}
pub fn switch_person_reverse(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.get_string().unwrap();
    unsafe {
        let mut index: usize = 0;
        for x in PIDS {
            if *x == pid { return PersonData::get(PIDS[ RAND_PERSONS [ 41 + index ] as usize ] ).unwrap(); }
            index += 1;
        }
    }
    return PersonData::get(&pid).unwrap();
}
pub fn is_player_unit(person: &PersonData) -> bool {
    let pid = person.pid.get_string().unwrap();
    for x in PIDS { if *x == pid { return true; } }
    return false;
}

#[unity::from_offset("App", "UnitPool", "GetHero")]
fn unit_pool_get_hero(replay :bool, method_info: OptionalMethod) -> Option<&'static Unit>;

pub fn lueur_on_map() -> bool {
    unsafe {
        let lueur_unit = unit_pool_get_hero(true, None);
        if lueur_unit.is_none() {
            return false;
        }
        return  lueur_unit.unwrap().force.unwrap().force_type < 3 ;
    }
}

#[unity::from_offset("App", "Unit", "set_Person")]
pub fn unit_set_person(this: &Unit, person: &PersonData, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "SetSelectedWeaponFromOriginalAptitude")]
fn unit_set_select_weapon_from_original_aptitude(this: &Unit, mask: &WeaponMask, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "AddAptitudeFromWeaponMask")]
fn unit_add_apt_from_weapon_mask(this: &Unit, method_info: OptionalMethod);

// done in Unit$$CreateImpl1
pub fn fixed_unit_weapon_mask(this: &mut Unit){
    this.original_aptitude.value = this.person.get_aptitude().value;
    this.aptitude.value = this.original_aptitude.value | this.person.get_sub_aptitude().value;
    this.selected_weapon_mask.value = 0;
    this.update_weapon_mask();
    this.set_select_weapon_from_original_aptitude(this.original_aptitude);
    this.update_weapon_mask();
    this.add_aptitude_from_weapon_mask();
}
#[unity::hook("App", "Unit", "CreateImpl2")]
pub fn unit_create_impl_2_hook(this: &mut Unit, method_info: OptionalMethod){
    // if Alear or not a playable unit, behave normally
    call_original!(this, method_info);
    if this.person.pid.get_string().unwrap() == "PID_リュール"  { return;  }
    if !is_player_unit(this.person) {
        if this.person.get_sp() > 0 {
            if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
                item::unit_change_to_random_class(this);
                fixed_unit_weapon_mask(this);
            }
            if GameVariableManager::get_bool("G_Random_Recruitment") ||  ( GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3 ) {  adjust_unit_items(this);  }
        }
        return;
    }
    if GameVariableManager::get_bool("G_Random_Recruitment"){
        let mut person = this.get_person();
        let mut new_person = switch_person(person);
         // Hub & Kizuna: person is already the correct person or MapSequence and Alear is not on the Map (Chapter 11)
        if ( GameUserData::get_sequence() == 5  ||  GameUserData::get_sequence() == 4 ) || (GameUserData::get_sequence() == 3 && !lueur_on_map() )  { 
            println!("Hub/Kizuna Recruitment");
            new_person = this.get_person();
            this.set_person( switch_person_reverse(person) );
            person = this.get_person();
            // Add Veyle Weapons
            if new_person.pid.get_string().unwrap() == "PID_ヴェイル" {
                this.item_list.add_item_no_duplicate(ItemData::get("IID_オヴスキュリテ").unwrap()); 
                this.item_list.add_item_no_duplicate(ItemData::get("IID_ミセリコルデ").unwrap());
            }
        }
        // if randomized to the same person
        if new_person.pid.get_string().unwrap() == person.pid.get_string().unwrap() {
            if GameVariableManager::get_number("G_Random_Job") == 1 || GameVariableManager::get_number("G_Random_Job") == 3  {
                item::unit_change_to_random_class(this);
                fixed_unit_weapon_mask(this);
                adjust_unit_items(this);
            }
            return;
        }
        println!("{} -> {}",  person.get_name().unwrap().get_string().unwrap(), new_person.get_name().unwrap().get_string().unwrap());
        let is_low = person.get_job().unwrap().is_low();
        let is_new_low = new_person.get_job().unwrap().is_low();

        let current_level = person.get_level() as i32;
        let mut current_internal_level = person.get_internal_level() as i32;
        if current_internal_level == 0 && !is_low { current_internal_level = 20; }
        let mut original_growth_rates: [u8; 11] = [0; 11];  // storing growth rates of the original person
        let original_gr = person.get_grow();    // growth rate of the original person
        let new_gr = new_person.get_grow(); // growth rate of the new person
        // Switch Growths rates to calculate stats, store the previous person's growths to restore it at the end
        for x in 0..11 { 
            original_growth_rates[x as usize] = original_gr[x as usize];
            original_gr[x as usize] = new_gr[x as usize];  
        }
        if is_low {
            if current_level > 20 { //Old Unit is in a special class so new unit needs to be promoted
                if is_new_low && new_person.get_job().unwrap().has_high() {    // new unpromoted unit can promoted 
                    //let target_level = current_level;
                    let level = current_level - 20;
                    let new_job = &new_person.get_job().unwrap().get_high_jobs()[0];
                    this.set_job(new_job);
                    this.auto_grow_capability( level, current_level);
                    call_original!(this, method_info);
                    this.set_level( level );
                    this.set_internal_level( 20 );
                }
                else if is_new_low && !new_person.get_job().unwrap().has_high() {   // special -> special
                    this.set_job(new_person.get_job().unwrap());
                    this.auto_grow_capability( current_level, current_level);
                    call_original!(this, method_info);
                    this.set_level( current_level );
                    this.set_internal_level( 0 );
                }
                else {  // special -> high
                    this.set_job(new_person.get_job().unwrap());
                    this.auto_grow_capability( current_level-20, current_level);
                    call_original!(this, method_info);
                    this.set_level( current_level - 20 );
                    this.set_internal_level( 20 );
                }
            }
            if is_new_low { // base or special class lvl < 20 -> base class
                this.set_job(new_person.get_job().unwrap());
                this.auto_grow_capability( current_level, current_level);
                call_original!(this, method_info);
                this.set_level( current_level );
                this.set_internal_level( 0 );
            }
            else {
                let new_job_list = new_person.get_job().unwrap().get_low_jobs();
                if new_job_list.len() == 3 {
                    let index = get_low_class_index(new_person);
                    this.set_job(&new_job_list[index as usize]);
                }
                else if new_job_list.len() == 0 { this.set_job(JobData::get("JID_ソードファイター").unwrap()); }    // if promoted class doesn't have a low class, change to sword fighter
                else {  this.set_job(&new_job_list[0]); }
                this.auto_grow_capability(current_level, current_level);
                call_original!(this, method_info);
                this.set_level(current_level);
                this.set_internal_level(0);
            }
        }
        else {  // Promoted
            if is_new_low { // new unit has a base class
                if new_person.get_job().unwrap().has_high() {   // base class -> 1st promoted class
                    let new_high_jobs = new_person.get_job().unwrap().get_high_jobs();
                    if new_high_jobs.len() == 0 { this.set_job(JobData::get("JID_ソードマスター").unwrap());  } // if no high class, change to Swordmaster
                    else { this.set_job(&new_high_jobs[0]); }
                    this.auto_grow_capability(current_level, current_level + 20);
                    call_original!(this, method_info);
                    this.set_level(current_level);
                    this.set_internal_level(current_internal_level);
                }
                else { // Promoted -> Special
                    this.set_job(new_person.get_job().unwrap());
                    let total_level = current_internal_level + current_level;
                    this.auto_grow_capability(total_level, 20+current_level);
                    call_original!(this, method_info);
                    this.set_level(total_level);
                    this.set_internal_level(0);
                    this.set_level( ( person.get_level() + person.get_internal_level() as u8 ).into() );
                }
            }
            else {  // Promoted -> Promoted
                this.set_job(new_person.get_job().unwrap());
                this.auto_grow_capability(current_level, current_level + 20);
                call_original!(this, method_info);
                this.set_level(current_level);
                this.set_internal_level( current_internal_level );
            }
        }
        for x in 0..11 { original_gr[x as usize] = original_growth_rates[x as usize]; } // Change back to original growth rate
        this.set_person(new_person);    // change person
        fixed_unit_weapon_mask(this);   // fixed weapon mask due to class changes
    }
    if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
        item::unit_change_to_random_class(this);
        fixed_unit_weapon_mask(this);
    }
    if GameVariableManager::get_bool("G_Random_Recruitment") ||  ( GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3 ) {  adjust_unit_items(this);  }
}
 #[unity::hook("App", "Unit", "CreateFromDispos")]
 pub fn create_from_dispos_hook(this: &mut Unit, data: &mut DisposData, method_info: OptionalMethod) {
        // Changing Emblems
    if data.gid.is_some() && GameVariableManager::get_number("G_Random_God_Mode") != 0 {
        let string = format!("G_R_{}", data.gid.unwrap().get_string().unwrap());
        let new_gid = GameVariableManager::get_string(&string);
        data.set_gid(new_gid);
    }
        // Changing Person AI
    if GameVariableManager::get_bool("G_Random_Recruitment") {
        if data.ai_action_value.is_some() {
            let string = data.ai_action_value.unwrap().get_string().unwrap();
            let found = PIDS.iter().position(|x| *x == string);
            if found.is_some() {
                let new_string = format!("G_R_{}", data.get_gid().get_string().unwrap());
                let new_pid = GameVariableManager::get_string(&new_string);
                data.ai_action_value = Some(new_pid);
            }
        }
        if data.ai_mind_value.is_some() {
            let string = data.ai_mind_value.unwrap().get_string().unwrap();
            let found = PIDS.iter().position(|x| *x == string);
            if found.is_some() {
                let new_string = format!("G_R_{}", data.get_gid().get_string().unwrap());
                let new_pid = GameVariableManager::get_string(&new_string);
                data.ai_mind_value = Some(new_pid);
            }
        }
        if data.ai_move_value.is_some() {
            let string = data.ai_move_value.unwrap().get_string().unwrap();
            let found = PIDS.iter().position(|x| *x == string);
            if found.is_some() {
                let new_string = format!("G_R_{}", data.get_gid().get_string().unwrap());
                let new_pid = GameVariableManager::get_string(&new_string);
                data.ai_move_value = Some(new_pid);
            }
        }
        if data.ai_attack_value.is_some() {
            let string = data.ai_move_value.unwrap().get_string().unwrap();
            let found = PIDS.iter().position(|x| *x == string);
            if found.is_some() {
                let new_string = format!("G_R_{}", data.get_gid().get_string().unwrap());
                let new_pid = GameVariableManager::get_string(&new_string);
                data.ai_attack_value = Some(new_pid);
            }
        }
    }
    call_original!(this, data, method_info);
    if this.person.get_asset_force() != 0 && !GameUserData::is_evil_map() {
        *CONFIG.lock().unwrap() =  crate::DeploymentConfig::new();
        let rng = Random::get_game();
        if CONFIG.lock().unwrap().autolevel { auto_level_unit(this); }
        if GameVariableManager::get_number("G_Random_Job") >= 2 {
            let rng_rate = CONFIG.lock().unwrap().random_enemy_job_rate;
            unsafe {
                if get_bmap_size(this.person, None) == 1 {
                    if rng.get_value(100) < rng_rate {
                        if item::enemy_unit_change_to_random_class(this){ 
                            adjust_unit_items(this); 
                            adjust_unit_ai(this, data);
                        }
                    }
                }
            }
        }
        if GameVariableManager::get_number("G_Random_Item") >= 2 { item::random_items_drops(this); }
        let mut revival_rate = CONFIG.lock().unwrap().revival_stone_rate;
        while revival_rate > 100 {
            this.hp_stock_count += 1;
            this.hp_stock_count_max += 1;
            revival_rate -= 100;
        }
        if rng.get_value(100) < revival_rate {
            this.hp_stock_count += 1;
            this.hp_stock_count_max += 1;
        }
        let skill_rate = CONFIG.lock().unwrap().random_enemy_skill_rate;
        if GameVariableManager::get_bool("G_Random_Skills") && rng.get_value(100) < skill_rate {
            if GameVariableManager::get_bool("G_Cleared_M004") {
                let diff = GameUserData::get_difficulty(false);
                let mut valid_skill = false;
                let mut count = 0;
                while !valid_skill && count < 5 {
                    let skill = crate::skill::get_random_skill(diff, rng);
                    valid_skill = this.private_skill.add_skill(skill, 10, 0); 
                    count += 1;
                }
            }
        }
        if rng.get_value(200) < CONFIG.lock().unwrap().enemy_emblem_rate && this.get_god_unit().is_none() {
            let current_chapter = GameUserData::get_chapter().cid.get_string().unwrap();
            if ( current_chapter != "CID_M022" && current_chapter != "CID_M011" ) && GameVariableManager::get_bool("G_Cleared_M004") {
                let emblem = rng.get_value(EMBLEMS.len() as i32) as usize;
                if try_equip_emblem(this, emblem) { adjust_emblem_unit_ai(this, data, emblem); }
            }
        } 
    }
    if str_contains(this.person.pid, "PID_M022_紋章士") { 
        this.private_skill.add_sid("SID_死亡回避", 10, 0); 
    }  // Prevent Green Emblems from dying in Chapter 22 if AI is changed
    if !is_player_unit(this.person) { return; }
    if GameVariableManager::get_bool("G_Random_Recruitment") ||  ( GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3 ){ 
        if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
            item::unit_change_to_random_class(this);
            fixed_unit_weapon_mask(this);
        }
        adjust_unit_items(this); 
    }
 }
fn adjust_unit_ai(unit: &Unit, data: &mut DisposData) {
    let job = unit.get_job();
    let m022 = GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M022";
    // Dancer
    if job.jid.get_string().unwrap() == "JID_ダンサー" {
        data.ai_mind_name = "AI_MI_Irregular".into();
        data.ai_action_name = "AI_AC_Everytime".into();
    }
    // staff user, Chapter 22 needs to use Force due to Green Emblem Allies
    else if job.get_weapon_mask().value & ( 1 << 7 ) != 0 {
        if unit.item_list.has_item_iid("IID_ワープ") {
            data.ai_action_name = "AI_AC_Everytime".into();
            data.ai_attack_name = "AI_AT_RodWarp".into();
            data.ai_attack_value = Some("1, 1".into());
            data.ai_move_name = "AI_MV_WeakEnemy".into();
        }
        else if unit.has_interfence_rod() {
            if m022 {  data.ai_attack_name = "AI_AT_InterferenceForceOnly".into();  }
            else {
                data.ai_attack_name = "AI_AT_Interference".into();
                data.ai_move_name =  "AI_MV_WeakEnemy".into();
            }
            if str_contains(data.ai_action_name, "AI_AC_TurnAttackRange") {
                data.ai_action_name =  "AI_AC_InterferenceRange".into();
                data.ai_action_value =  Some("".into());
            }
            else {
                data.ai_action_name =  "AI_AC_InterferenceRange".into();
                data.ai_action_value =  Some("".into());
            }
        }
        else if unit.has_heal_rod() {
            if m022 { data.ai_attack_name =  "AI_AT_AttackToHealForceOnly".into(); }
            else {
                data.ai_attack_name =  "AI_AT_HealToAttack".into();
                data.ai_move_name =  "AI_MV_WeakEnemy".into();
            }
        }
        else {
            data.ai_action_name =  "AI_AC_Everytime".into();
            data.ai_move_name =  "AI_MV_WeakEnemy".into();
        }
    }
    else {
        if str_contains(data.ai_action_name, "Guard") || str_contains(data.ai_mind_name, "Guard") { //Chain Guarder Unit
            unit.private_skill.add_sid("SID_チェインガード許可", 10, 0); 
        }
        // Healer turned non healer
        if str_contains(data.ai_action_name, "Heal") { data.ai_action_name = "AI_AC_AttackRange".into(); }
        if str_contains(data.ai_attack_name, "Heal") {  
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
        }
        if str_contains(data.ai_move_name, "Heal") {  data.ai_move_name = "AI_MV_WeakEnemy".into(); }
    }
    if m022 {
        data.ai_move_name = "AI_MV_ForceOnly".into();
        data.ai_move_value = Some("FORCE_PLAYER".into());
        data.ai_attack_value = Some("FORCE_PLAYER".into());
    }
    unsafe { unit_set_dispos_ai(unit, data, None); }
}

pub fn adjust_unit_items(unit: &Unit) {
    let job = unit.get_job();
    let mut weapon_mask = job.get_weapon_mask().value | unit.selected_weapon_mask.value;
    if weapon_mask == 0 {  weapon_mask = unit.selected_weapon_mask.value; }
    let list_count = unit.item_list.get_count();
    let name =  unit.person.get_name().unwrap().get_string().unwrap();
    if list_count == 0 {
        return;
    }
    let mut slot = 0;
    let mut weapon_mask_array: [i32; 4] = [0; 4];
    let mut weapon_level: [i32; 4] = [0; 4];
    for x in 1..9 {
        if x == 7 { continue; }
        if weapon_mask & (1 << x) != 0 {
            weapon_mask_array[slot as usize] =  weapon_mask & (1 << x);
            weapon_level[slot as usize] = job.get_max_weapon_level(x);
            println!("Job has weapon type: {}, max level: {}", x, weapon_level[slot as usize]);
            slot += 1;
        }
        if slot == 4 { break; }
    }
    let n_weapons = slot;
    slot = 0;
    let jid = unit.get_job().jid.get_string().unwrap();
    for x in 0..list_count+3 {
        let item = unit.item_list.get_item(x);
        if item.is_some() {
            let weapon = &item.unwrap();
            //if weapon.is_drop() { continue; }
            let kind = weapon.item.get_kind(); 
            if kind > 8 || kind == 0 { continue; }
            if kind == 7 { continue; }
            let rank = weapon.item.get_weapon_level();
            println!("{}: Weapon Mask {} & {} (kind = {}, rank {} ) = {} for {} ", name, weapon_mask, 1 << kind, kind, rank, weapon_mask & ( 1 <<  kind ), weapon.item.name.get_string().unwrap());
            if weapon_mask & ( 1 <<  kind ) == 0 {
                // For Veyle
                if name == "MPID_Veyre" && weapon_mask_array[slot] == 64 {
                    if slot == 0 { weapon.ctor(ItemData::get("IID_オヴスキュリテ").unwrap());  }
                    else if slot == 1 { weapon.ctor(ItemData::get("IID_ミセリコルデ").unwrap());  }
                }
                else if jid == "JID_マージカノン" { // mage cannon
                    if slot == 0 { 
                        weapon.ctor_str("IID_弾_物理"); 
                        slot +=1;
                    }
                    if slot == 1 {
                        weapon.ctor_str("IID_弾_魔法");
                        slot += 1;
                    }
                }
                else if jid == "JID_異形狼" || jid == "JID_幻影狼" {
                    if slot == 0 {
                        weapon.ctor_str("IID_牙");
                        slot +=1;
                    }
                    else if slot == 1 {
                        weapon.ctor_str("IID_HPの薬");
                        slot +=1;
                    }
                }
                else {
                    if slot < n_weapons {
                        item::replace_weapon(weapon, weapon_mask_array[slot as usize], weapon_level[slot as usize]);
                        if n_weapons > 1 { slot += 1; }
                    }
                    else if slot < 4 && slot >= 1 {
                        item::replace_weapon(weapon, weapon_mask_array[slot - 1 as usize], weapon_level[slot - 1 as usize]);
                    }
                }
            }
        }
    }
    item::adjust_staffs(unit);
    unsafe { unit_update_auto_equip(unit, None); }
    println!("item adjustment for {} complete", name);
}

#[unity::from_offset("App", "Unit", "UpdateStateWithAutoEquip")]
pub fn unit_update_auto_equip(this: &Unit, method_info: OptionalMethod);

#[skyline::from_offset(0x01a37990)]
pub fn unit_add_private_skill(this: &Unit, skill: &SkillData, method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x01a35f80)]
pub fn unit_add_equip_skill(this: &Unit, skill: &SkillData, method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x01f25ec0)]
fn get_bmap_size(this: &PersonData, method_info: OptionalMethod) -> u8;

#[unity::from_offset("App", "Unit", "SetDisposAi")]
pub fn unit_set_dispos_ai(this: &Unit, data: &mut DisposData, method_info: OptionalMethod);

pub struct RandomPersonMod;
impl ConfigBasicMenuItemSwitchMethods for RandomPersonMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_recruitment);
        if CONFIG.lock().unwrap().random_recruitment != result {
            CONFIG.lock().unwrap().random_recruitment = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_recruitment { this.help_text = "Characters will be recruited in a random order.".into(); }
        else { this.help_text = "Standard recruitment order.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_recruitment { this.command_text = "Random".into();  }
        else { this.command_text = "Standard".into(); }
    }
}

#[unity::hook("App", "Mess", "GetImpl")]
pub fn mess_get_impl_hook(label: Option<&Il2CppString>, is_replaced: bool, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(label, is_replaced, method_info);
    unsafe {
        if label.is_some() {
            let mess_label = label.unwrap().get_string().unwrap();
            if mess_label == "MSID_H_EirikEngage" {
                let eirika_replacement = GodData::get( deploy::EMBLEM_GIDS[ EIRIKA_INDEX ] ).unwrap().mid;
                return replace_str(result, Mess::get("MGID_Eirik"), Mess::get(eirika_replacement), None);
            }
            if mess_label == "MID_RULE_M006_LOSE" {
                return replace_str(result, Mess::get("MPID_Yunaka"), Mess::get(PersonData::get( PIDS [ RAND_PERSONS[10] as usize ]).unwrap().get_name().unwrap()), None);
            }
            if mess_label == "MID_RULE_M015_WIN" {
                return replace_str(result, Mess::get("MPID_Seadas"), Mess::get(PersonData::get( PIDS [ RAND_PERSONS[27] as usize ]).unwrap().get_name().unwrap()), None);
            }
            if mess_label == "MID_RULE_M015_LOSE" {
                return replace_str(result, Mess::get("MPID_Seadas"), Mess::get(PersonData::get( PIDS [ RAND_PERSONS[27] as usize ]).unwrap().get_name().unwrap()), None);
            }
            if mess_label == "MID_TUT_NAVI_M015_ESCAPE" {
                return replace_str(result, Mess::get("MPID_Seadas"), Mess::get(PersonData::get( PIDS [ RAND_PERSONS[27] as usize ]).unwrap().get_name().unwrap()), None);
            }
            if string_start_with(label.unwrap(), "MTID_Ring_".into(), None) {
                for x in 1..12 {
                    let tid_label = format!("MTID_Ring_{}",  RINGS[x as usize ]);
                    if mess_label == tid_label { return Mess::get(format!("MGID_Ring_{}", RINGS[ RANDOMIZED_INDEX[ x as usize ] as usize ])); }
                }
            }
            if string_start_with(label.unwrap(), "MIID_H_".into(), None) && GameVariableManager::get_bool("G_Random_Engage_Weps") {
                let found = item::ENGAGE_ITEMS.lock().unwrap().item_list.iter().position(|x| x.miid == mess_label);
                if found.is_some() {
                    let new_emblem = item::ENGAGE_ITEMS.lock().unwrap().item_list[found.unwrap()].new_emblem;
                    let old_emblem = item::ENGAGE_ITEMS.lock().unwrap().item_list[found.unwrap()].original_emblem;
                    if new_emblem == -1 { return result; }
                    let emblem_name = Mess::get( GodData::get(&format!("GID_{}", crate::skill::EMBLEM_ASSET[old_emblem as usize])).unwrap().mid);
                    let new_emblem_name = Mess::get( GodData::get(&format!("GID_{}", crate::skill::EMBLEM_ASSET[new_emblem as usize])).unwrap().mid);
                    return replace_str(result, emblem_name, new_emblem_name, None);
                }
                return result;
            }
            if string_start_with(label.unwrap(), "MID_TUT_NAVI_M022_GET_".into(), None){
                if GameVariableManager::get_number("G_Emblem_Mode") != 0 {
                    let mock_text = call_original!(Some("MID_TUT_NAVI_M022_GET_Siglud".into()), is_replaced, method_info);
                    for x in RINGS {
                        if str_contains(label.unwrap(), x) {
                            let new_ring = format!("MGID_Ring_{}", x);
                            return replace_str(mock_text, Mess::get("MGID_Ring_Siglud"),  Mess::get(new_ring), None);
                        }
                    }
                }
            }
        }
    }
    return result;
}
