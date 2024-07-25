use unity::prelude::*;
use engage::{
    menu::{
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    gamevariable::*, gameuserdata::*, hub::access::*, random::*, mess::*,
    gamedata::{*, item::*, skill::SkillData, dispos::*, unit::*},
};
use skyline::patching::Patch;
use unity::il2cpp::object::Array;
use crate::{
    enums::*,
    utils::*, item, autolevel::*,
};
use super::CONFIG;
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
                if person.get_name().is_some() { string = person.get_name().expect(format!("Person #{} has nonzero SP but no name", x).as_str()).get_string().unwrap(); }
                else { string = " --- ".into(); }
                println!("Added Person #{}: {}: {} has SP > 0", index, person.pid.get_string().unwrap(), string);
            }
        }
    }
    println!("Total of {} Playable Units", list.len());
}
fn create_reverse() {
    for x in 0..41 {
        let key = format!("G_R_{}",PIDS[x as usize]);
        let pid = GameVariableManager::get_string(&key).get_string().unwrap();
        for y in 0..41 {
            if pid == PIDS[y as usize] {
                GameVariableManager::make_entry_str(&format!("G_R2_{}",PIDS[y as usize]), PIDS[x as usize]);
            }
        }
    }
}
fn get_custom_recruitment_list() -> [i32; 41] {
    let mut output: [i32; 41] = [-1; 41];
    let mut set: [bool; 41] = [false; 41];
    let length = crate::enums::SET_RECRUITMENT.lock().unwrap().len();
    for x in 0..length {
        let value = crate::enums::SET_RECRUITMENT.lock().unwrap()[x as usize];
        if value.2 { continue; } // emblem
        let index_1 = value.0;
        let index_2 = value.1;
        if output[index_1 as usize] == -1 && !set[index_2 as usize] { 
            output[index_1 as usize] = index_2; 
            set[index_2 as usize] = true;
        }
    }
    let random = unsafe { UNIT_RANDOM };
    if !random { 
        for x in 0..41 {
            if output[x as usize] != -1 {
                let position = output.iter().find(|y| **y == x);
                if position.is_none() {
                    let mut index = output[ x as usize ];
                    while output[ index as usize] != -1 { index = output[ index as usize]; }
                    output[ index as usize ] = x;
                }
            }
        }
     }
     else {
        println!("Adding randomization to custom unit recruitment");
        let person_pool = if dlc_check() { 41 } else { 36 };
        let rng = get_rng();
        for x in 0..person_pool {
            if output[ x as usize ] != -1 { continue; }
            let mut index;
            loop {
                index = rng.get_value(person_pool); 
                if x == 0 && index > 35 { continue; }
                if ( index == 2 || index == 3 ) && x == 1 { continue; }
                if ( x == 2 && index == 3) || ( x == 3 && index == 2 ) { continue; }
                if !set[ index as usize] { break; }
            }
            set[index as usize] = true;
            output[x as usize] = index; 
        }
    }
    output
}
fn set_hub_facilities() {
    let aid = ["AID_蚤の市", "AID_筋肉体操", "AID_ドラゴンライド", "AID_釣り", "AID_占い小屋"];
    let locator = ["LocatorSell01", "LocatorTraining01", "LocatorDragon01", "LocatorFish01", "LocatorFortune01"];
    let index = [ 23, 4, 17, 14, 27];
    let hub_dispos = HubDisposData::get_array_mut().unwrap();
    for x in 0..aid.len() {
        let data = HubFacilityData::get_mut(aid[x as usize]);
        let pid = PIDS[index[x] as usize];
        let a_index = pid_to_index(&pid.to_string(), true) as usize;
        if data.is_some() {
            let facility = data.unwrap();
            facility.condition_cid = format!("CID_{}", RECRUIT_CID[ a_index as usize] ).into() ;
            for y in 0..hub_dispos[1].len() {
                let hub_locator = hub_dispos[1][y as usize].get_locator();
                if hub_locator.get_string().unwrap() == locator[ x as usize] {
                    hub_dispos[1][y as usize].set_chapter(RECRUIT_CID[ a_index as usize].into() );
                    break;
                }
            }
        }
    }
}
pub fn randomize_person() {
    if !can_rand() { return; }
    println!("Checking Randomized Recruitment Status...");
    if !GameVariableManager::exist("G_Random_Person_Set") {  GameVariableManager::make_entry("G_Random_Person_Set", 0);  }
    if GameVariableManager::get_bool("G_Random_Person_Set") { 
        if GameVariableManager::get_number("G_Random_Recruitment") != 0 { 
            set_hub_facilities(); 
            change_hub_dispos(false);
        }
        return; 
    }
    else if GameVariableManager::exist("G_R_PID_リュール") && !GameVariableManager::exist("G_R2_PID_リュール") { create_reverse();}
    else {
        let emblem_list_size = if dlc_check() { 41 } else { 36};
        for i in 0..41 { 
            GameVariableManager::make_entry_str(&format!("G_R_{}",PIDS[i as usize]), PIDS[i as usize]);
            GameVariableManager::make_entry_str(&format!("G_R2_{}",PIDS[i as usize]), PIDS[i as usize]);
        }
        let rng = get_rng();
        let mut emblem_count: i32 = 0;
        let mut set_emblems: [bool; 41] = [false; 41];
        match GameVariableManager::get_number("G_Random_Recruitment") {
            1 => {
                while emblem_count < emblem_list_size {
                    let index = rng.get_value(emblem_list_size);
                    if index >= emblem_list_size { continue; }
                    if emblem_count == 0 && index > 35 { continue; }
                    if emblem_count == 1 && (index == 2 || index == 3) { continue; }
                    if ( emblem_count == 2 && index == 3 ) ||  ( emblem_count == 3 && index == 2 ) { continue; }
                    if !set_emblems[index as usize] {
                        let string = format!("G_R_{}",PIDS[emblem_count as usize]);
                        GameVariableManager::set_string(&string, PIDS[index as usize]);
                        let string2 = format!("G_R2_{}",PIDS[index as usize]);
                        GameVariableManager::set_string(&string2, PIDS[emblem_count as usize]);
                        println!("{} -> {}", PersonData::get(PIDS[ emblem_count as usize]).unwrap().get_name().unwrap().get_string().unwrap(),
                        PersonData::get(PIDS[ index as usize]).unwrap().get_name().unwrap().get_string().unwrap());
                        set_emblems[ index as usize ] = true;
                        emblem_count += 1;
                    }
                }
            },
            2 => {   //Reverse
                for x in 0..41 {
                    let index = RR_ORDER[x as usize] as usize;
                    GameVariableManager::set_string(&format!("G_R_{}",PIDS[x as usize]), PIDS[index]);
                    GameVariableManager::set_string(&format!("G_R2_{}",PIDS[index]), PIDS[x as usize]);
                    println!("{} -> {}", pid_to_mpid(&PIDS[x as usize].to_string()), pid_to_mpid(&PIDS[index].to_string()));
                }
            },
            3 => { // Custom
                let custom = get_custom_recruitment_list();
                println!("Custom Recruitment List");
                for x in 0..41 {
                    let mut index = x as usize; 
                    if set_emblems[index] { continue; }
                    while custom[index] != -1 {
                        if set_emblems[index] { break; }
                        GameVariableManager::set_string(&format!("G_R_{}",PIDS[index]), PIDS[custom[index] as usize]);
                        GameVariableManager::set_string(&format!("G_R2_{}",PIDS[custom[index] as usize]), PIDS[index]);
                        set_emblems[index] = true;
                        println!("Loop {}, {} -> {}", x, pid_to_mpid(&PIDS[index].to_string()), pid_to_mpid(&PIDS[custom[index] as usize].to_string()));
                        index = custom[index] as usize;
                    }
                    if set_emblems[index] { continue; }
                    GameVariableManager::set_string(&format!("G_R_{}",PIDS[index]), PIDS[x as usize]);
                    GameVariableManager::set_string(&format!("G_R2_{}",PIDS[x as usize]), PIDS[index]);
                    println!("{} -> {}", pid_to_mpid(&PIDS[index].to_string()), pid_to_mpid(&PIDS[x as usize].to_string()));
                    set_emblems[index] = true;
                }
            },
            _ => {},
        }
    }
    GameVariableManager::set_bool("G_Random_Person_Set", true);
    set_hub_facilities(); 
    change_hub_dispos(false);
}
pub fn pid_to_index(pid: &String, reverse: bool) -> i32 {
    let replacement = find_pid_replacement(pid, reverse);
    if replacement.is_some() {
        let found = replacement.unwrap();
        let found_pid = PIDS.iter().position(|&x| x == found); 
        if found_pid.is_some() { return found_pid.unwrap() as i32; }
        let found_gid = EMBLEM_GIDS.iter().position(|&x| x == found); 
        if found_gid.is_some() { return found_gid.unwrap() as i32; }
    }
    return -1;  // to cause crashes
}

pub fn find_pid_replacement(pid: &String, reverse: bool) -> Option<String>{
    let found_pid = PIDS.iter().position(|&x| x == *pid); 
    if found_pid.is_some() {
        if reverse { return Some( GameVariableManager::get_string(&format!("G_R2_{}", pid)).get_string().unwrap()); }   
        else { return Some( GameVariableManager::get_string(&format!("G_R_{}", pid)).get_string().unwrap()); }
    }
    let found_gid = EMBLEM_GIDS.iter().position(|&x| x == *pid);
    if found_gid.is_some() {
        if reverse { return Some( GameVariableManager::get_string(&format!("G_R2_{}", pid)).get_string().unwrap()); }   
        else { return Some( GameVariableManager::get_string(&format!("G_R_{}", pid)).get_string().unwrap()); }
    }
    return None;
}
pub fn pid_to_mpid(pid: &String) -> String { return PersonData::get(&pid).unwrap().get_name().unwrap().get_string().unwrap(); }

pub fn change_hub_dispos(revert: bool) {
    let t_list = HubDisposData::get_array_mut().expect("Me");
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_aid();
            if aid.is_some() { 
                if str_contains(aid.unwrap(), "GID_") && str_contains(t_list[x][y].parent.array_name, "Fld_S0") { continue; }
                let pid = aid.unwrap().get_string().unwrap();
                let new_pid = find_pid_replacement(&pid, revert);
                if new_pid.is_some() { 
                    let n_pid = new_pid.unwrap();
                    t_list[x][y].set_aid(n_pid.into());
                 }
            }
        }
    }
    if GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap() == "PID_リュール" { return;  }
    let replacement = GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap();
    let hublist = crate::shop::HubRandomSet::get_list_mut().unwrap();
    for x in 0..hublist.len() {
        let list = &mut hublist[x]; 
        for y in 0..list.len() {
            if list.parent.list[y].iid.get_string().unwrap() == replacement {
                list.parent.list[y].iid = "PID_リュール".into();
            }
        }
    }
}
pub fn change_map_dispos() {
    let t_list = DisposData::get_list_mut().expect("Me");
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_pid();
            if aid.is_none() { continue; }
            let pid = aid.unwrap().get_string().unwrap();
            if pid == "PID_リュール" { 
                let new_pif = GameVariableManager::get_string("G_R_PID_リュール");
                t_list[x][y].set_pid(new_pif); 
            }
            else if ( t_list[x][y].get_flag().value & 8 == 0 ) && t_list[x][y].get_force() == 0  {
                for z in PIDS {
                    if *z == pid {
                        let new_pid = GameVariableManager::get_string(&format!("G_R2_{}", z));
                        t_list[x][y].set_pid(new_pid);
                        break;
                    }
                }
                for z in PIDS {
                    if *z == pid {
                        let new_pid = GameVariableManager::get_string(&format!("G_R_{}", z));
                        t_list[x][y].set_pid(new_pid);
                        break;
                    }
                }
            }
        }
    }
}

pub fn get_low_class_index(this: &PersonData) -> usize {
    let apt = this.get_aptitude().value | this.get_sub_aptitude().value;
    if apt & 2 == 2 { return 0;  }
    else if apt & 4 == 4 { return 1; }
    else if apt & 8 == 8 { return 2; }
    return 0;
}
pub fn switch_person(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.get_string().unwrap();
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return PersonData::get(&pid).unwrap(); }
    let var_str = format!("G_R_{}", pid);
    let new_pid = GameVariableManager::get_string(&var_str);
    unsafe { if is_null_empty(new_pid, None) { return PersonData::get(&pid).unwrap(); } }
    let new_person = PersonData::get(&new_pid.get_string().unwrap());
    if new_person.is_some() { return new_person.unwrap(); }
    else { return PersonData::get(&pid).unwrap(); }
}
pub fn switch_person_reverse(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.get_string().unwrap();
    let reverse = GameVariableManager::get_string(&format!("G_R2_{}", pid)).get_string().unwrap();
    return PersonData::get(&reverse).unwrap();
}
pub fn is_player_unit(person: &PersonData) -> bool {
    let pid = person.pid.get_string().unwrap();
    for x in PIDS { if *x == pid { return true; } }
    return false;
}
pub fn set_unit_edit_name(unit: &Unit) {
    if unit.person.pid.get_string().unwrap() == "PID_リュール" || unit.person.get_flag().value & 1024 != 0 {
        if GameVariableManager::get_number("G_Lueur_Gender2") != 0 { unit.edit.set_gender( GameVariableManager::get_number("G_Lueur_Gender2") ); }
        else {unit.edit.set_gender( 1 );  }
        if GameVariableManager::exist("G_Lueur_Name") { unit.edit.set_name( GameVariableManager::get_string("G_Lueur_Name") ); }
        unit.person.set_gender( unit.edit.gender );
    }
    if unit.person.get_flag().value & 128 != 0 {
        unit.edit.set_name( GameVariableManager::get_string("G_Lueur_Name") );
        if GameVariableManager::get_number("G_Lueur_Gender2") != 0 { unit.edit.set_gender( GameVariableManager::get_number("G_Lueur_Gender2") ); }
        else {unit.edit.set_gender( 1 );  }
    } 
}
pub fn change_lueur_for_recruitment(is_start: bool) {
    if GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap() == "PID_リュール" { return;  }
    if !crate::utils::can_rand() { return; }
        // remove hero status on alear and place it on the replacement and add alear skills on the replacement
    let person_lueur = PersonData::get(PIDS[0]).unwrap();
    let lueur_sids = person_lueur.get_common_sids().unwrap();
    for x in 0..lueur_sids.len() {
        if lueur_sids[x].get_string().unwrap() == "SID_主人公" { lueur_sids[x] = "SID_無し".into(); }
    }
    person_lueur.on_complete();
    let new_hero = switch_person(person_lueur);
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
        let lueur = unsafe { crate::deploy::force_get_unit_from_pid(PIDS[0].into(), true, None) };
        if lueur.is_some() { 
            let lueur_unit = lueur.unwrap();
            crate::grow::change_unit_autolevel(lueur_unit, true);
            if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
                item::unit_change_to_random_class(lueur_unit);
                fixed_unit_weapon_mask(lueur_unit);
                adjust_unit_items(lueur_unit);
            }
            lueur_unit.transfer(5, false); 
            println!("Lueur transfer to force 5");
            crate::utils::get_lueur_name_gender(); // grab gender and name
            GameVariableManager::make_entry("G_Lueur_Gender2", lueur_unit.edit.gender);
            println!("Lueur Gender: {}",  GameVariableManager::get_number("G_Lueur_Gender2"));
        }
        let new_unit =  unsafe { join_unit(new_hero, None) };
        if new_unit.is_some() {
            println!("{} joined.", new_hero.get_name().unwrap().get_string().unwrap());
            let unit = new_unit.unwrap();
            unit.edit.set_name( Mess::get( new_hero.get_name().unwrap()) );
            unit.edit.set_gender( new_hero.get_gender() );
            println!("{} unit edit set", new_hero.get_name().unwrap().get_string().unwrap());
            unit.transfer(3, false);
        }
    }
    Patch::in_text(0x02d524e0).nop().unwrap();
    Patch::in_text(0x02d524e4).nop().unwrap();
    // LueurW_God or Lueur_God in GetPath 
    if GameVariableManager::get_number("G_Lueur_Gender2") == 2 {  
        Patch::in_text(0x02d524e8).bytes(&[0x48, 0x00, 0x80, 0x52]).unwrap(); 
        person_lueur.set_ascii_name("LueurW".into());
    }
    else { 
        Patch::in_text(0x02d524e8).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap();
        person_lueur.set_ascii_name("Lueur".into());
    }

    Patch::in_text(0x0233f104).bytes(&[0x01,0x10, 0x80, 0x52]).unwrap(); // GodUnit$$GetName ignore hero flag on Emblem Alear
    let lueur_god_offsets = [0x02d51dec, 0x021e12ac, 0x02915844, 0x02915844, 0x02915694, 0x01c666ac,0x02081edc, 0x01c69d60, 0x01c66588];
    for x in lueur_god_offsets { mov_x0_0(x); }

    // For Hub-Related Activites
    let offsets = [0x02ae8d28, 0x02ae9000, 0x02a5d0f4, 0x01cfd4c4, 0x01d03184, 0x01e5fe00, 0x01e5ff4c, 0x027049c8];
    let new_hero_gender = if new_hero.get_gender() == 2 || (new_hero.get_gender() == 1 && new_hero.get_flag().value & 32 != 0 ) { 2 } else { 1 };
    for x in offsets {
        if new_hero_gender == 1 {  crate::utils::mov_x0_0(x); }
        else { crate::utils::mov_1(x); }
    }
    unsafe { LUEUR_CHANGE = true; }
}

#[unity::from_offset("App", "UnitPool", "GetHero")]
pub fn unit_pool_get_hero(replay :bool, method_info: OptionalMethod) -> Option<&'static Unit>;

// Handle the case of Chapter 11 ends with not escape
pub fn m011_ivy_recruitment_check(){
    if !crate::utils::can_rand() { return; }
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return; }
    if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M011" && lueur_on_map() {
        GameVariableManager::make_entry("MapRecruit", 1);
        GameVariableManager::set_bool("MapRecruit", true);
    }
}

pub fn lueur_on_map() -> bool {
    let lueur_unit = unsafe { unit_pool_get_hero(true, None) };
    if lueur_unit.is_none() { return false;  }
    return lueur_unit.unwrap().force.unwrap().force_type < 3 ;
}

#[skyline::from_offset(0x01c73960)]
fn join_unit(person: &PersonData, method_info: OptionalMethod) -> Option<&'static mut Unit>;

#[unity::from_offset("App", "Unit", "set_Person")]
pub fn unit_set_person(this: &Unit, person: &PersonData, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "SetSelectedWeaponFromOriginalAptitude")]
fn unit_set_select_weapon_from_original_aptitude(this: &Unit, mask: &WeaponMask, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "AddAptitudeFromWeaponMask")]
fn unit_add_apt_from_weapon_mask(this: &Unit, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "GetEngageAttack")]
fn unit_get_engage_atk(this: &Unit, method_info: OptionalMethod) -> Option<&'static SkillData>;

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
    let can_lueur_change = unsafe { LUEUR_CHANGE };
    call_original!(this, method_info);
    println!("Create Impl 2 on {} - #{}", this.person.get_name().unwrap().get_string().unwrap(),  this.person.parent.index);
    if !can_lueur_change {
        if this.person.pid.get_string().unwrap() == "PID_リュール" {  unsafe { LUEUR_CHANGE = true; } }
        return;
    }
    if !can_rand() { return; }
    if !is_player_unit(this.person) {
        if this.person.get_sp() > 0 {
            if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
                item::unit_change_to_random_class(this);
                fixed_unit_weapon_mask(this);
                println!("CreateFromDispos Adjust Unit Items for {}", Mess::get(this.person.get_name().unwrap()).get_string().unwrap());
            }
            if GameVariableManager::get_number("G_Random_Recruitment") != 0 ||  ( GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3 ) {  adjust_unit_items(this);  }
        }
        return;
    }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        if GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap() == this.person.pid.get_string().unwrap() && GameUserData::get_sequence() == 0 {
            crate::grow::change_unit_autolevel(this, true);
            this.item_list.put_off_all_item();
            this.item_list.add_item_no_duplicate(ItemData::get("IID_鉄の剣").unwrap()); 
            this.item_list.add_item_no_duplicate(ItemData::get("IID_傷薬").unwrap());
        }
         // Hub & Kizuna: person is already the correct person or MapSequence and Alear is not on the Map (Chapter 11)
        else if ( GameUserData::get_sequence() == 5  ||  GameUserData::get_sequence() == 4 ) || 
            (GameUserData::get_sequence() == 3 && ( GameVariableManager::get_bool("MapRecruit") || ( GameVariableManager::get_number("G_DeploymentMode") != 3 && !lueur_on_map() ) ) )  { 
            println!("Hub/Kizuna Recruitment");
            crate::grow::change_unit_autolevel(this, true);
            if this.person.pid.get_string().unwrap() == "PID_ヴェイル" {
                this.item_list.put_off_all_item();
                this.item_list.add_item_no_duplicate(ItemData::get("IID_オヴスキュリテ").unwrap()); 
                this.item_list.add_item_no_duplicate(ItemData::get("IID_ミセリコルデ").unwrap());
            }
        }
        // if randomized to the same person
        else if switch_person(this.person).pid.get_string().unwrap() ==  this.person.pid.get_string().unwrap() {
            println!("Same Person");
            if GameVariableManager::get_number("G_Random_Job") == 1 || GameVariableManager::get_number("G_Random_Job") == 3  {
                item::unit_change_to_random_class(this);
                fixed_unit_weapon_mask(this);
                adjust_unit_items(this);
            }
            return;
        }
        else { crate::grow::change_unit_autolevel(this, false);  }
    }
    if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
        item::unit_change_to_random_class(this);
        fixed_unit_weapon_mask(this);
    }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 ||  ( GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3 ) {  adjust_unit_items(this);  }
    item::remove_duplicates(this.item_list);
    println!("Finish with Create2Impl for {}", this.person.get_name().unwrap().get_string().unwrap());
    set_unit_edit_name(this);
    for x in 0..8 {
        let item = this.item_list.get_item(x);
        if item.is_some() {
            let weapon = &item.unwrap();
            println!("Item {}: {}", x, weapon.item.name.get_string().unwrap());
        }
    }
}
 #[unity::hook("App", "Unit", "CreateFromDispos")]
 pub fn create_from_dispos_hook(this: &mut Unit, data: &mut DisposData, method_info: OptionalMethod) {
 // Changing Emblems
    if data.gid.is_some() && GameVariableManager::get_number("G_Emblem_Mode") != 0 {
        let string = data.gid.unwrap().get_string().unwrap();
        if EMBLEM_GIDS.iter().position(|x| *x == string).is_some() {
            let new_string = format!("G_R_{}", string);
            let new_gid = GameVariableManager::get_string(&new_string);
            data.set_gid(new_gid);
        }
    }
// Changing Person AI
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        if data.ai_action_value.is_some() {
            let string = data.ai_action_value.unwrap().get_string().unwrap();
            let found = PIDS.iter().position(|x| *x == string);
            if found.is_some() {
                let new_string = format!("G_R_{}", string);
                let new_pid = GameVariableManager::get_string(&new_string);
                data.ai_action_value = Some(new_pid);
            }
        }
        if data.ai_mind_value.is_some() {
            let string = data.ai_mind_value.unwrap().get_string().unwrap();
            let found = PIDS.iter().position(|x| *x == string);
            if found.is_some() {
                let new_string = format!("G_R_{}", string);
                let new_pid = GameVariableManager::get_string(&new_string);
                data.ai_mind_value = Some(new_pid);
            }
        }
        if data.ai_move_value.is_some() {
            let string = data.ai_move_value.unwrap().get_string().unwrap();
            let found = PIDS.iter().position(|x| *x == string);
            if found.is_some() {
                let new_string = format!("G_R_{}", string);
                let new_pid = GameVariableManager::get_string(&new_string);
                data.ai_move_value = Some(new_pid);
            }
        }
        if data.ai_attack_value.is_some() {
            let string = data.ai_attack_value.unwrap().get_string().unwrap();
            let found = PIDS.iter().position(|x| *x == string);
            if found.is_some() {
                let new_string = format!("G_R_{}", string);
                let new_pid = GameVariableManager::get_string(&new_string);
                data.ai_attack_value = Some(new_pid);
            }
        }
    }
    call_original!(this, data, method_info);
    set_unit_edit_name(this);
    if this.person.get_flag().value & 512 == 512 {
        fixed_unit_weapon_mask(this);
        adjust_unit_items(this); 
        adjust_unit_ai(this, data);
    }
    if this.person.get_asset_force() != 0 && !GameUserData::is_evil_map() {
        *CONFIG.lock().unwrap() =  crate::DeploymentConfig::new();
        let rng = Random::get_game();
        if str_contains(GameUserData::get_chapter().cid, "CID_S0") && GameVariableManager::get_number("G_Emblem_Mode") != 0 { emblem_paralogue_level_adjustment(this); } 
        if GameVariableManager::get_bool("G_DVC_Autolevel") { auto_level_unit(this); }
        if !crate::utils::can_rand() { return; }
        if GameVariableManager::get_number("G_Random_Job") >= 2 {
            let rng_rate = GameVariableManager::get_number("G_EnemyJobGauge");
            if unsafe { get_bmap_size(this.person, None) } == 1 {
                if rng.get_value(100) < rng_rate {
                    if item::enemy_unit_change_to_random_class(this){ 
                        fixed_unit_weapon_mask(this);
                        adjust_unit_items(this); 
                        adjust_unit_ai(this, data);
                    }
                }
            }
        }
        if GameVariableManager::get_number("G_Random_Item") >= 2 { item::random_items_drops(this); }
        let revival_rate = GameVariableManager::get_number("G_EnemyRevivalStone");
        if rng.get_value(100) < revival_rate {
            this.hp_stock_count += 1;
            this.hp_stock_count_max += 1;
        }
        let skill_rate = GameVariableManager::get_number("G_EnemySkillGauge");
        if GameVariableManager::get_bool("G_Random_Skills") && rng.get_value(100) < skill_rate {
            if GameVariableManager::get_bool("G_Cleared_M004") {
                let diff = GameUserData::get_difficulty(false);
                let mut valid_skill = false;
                let mut count = 0;
                while !valid_skill && count < 5 {
                    let skill = crate::skill::get_random_skill(diff, rng);
                    if !has_skill(this, skill) {
                        valid_skill = this.private_skill.add_skill(skill, 10, 0); 
                        count += 1;
                    }
                }
            }
        }
        if rng.get_value(200) < GameVariableManager::get_number("G_EnemyEmblemGauge") && this.get_god_unit().is_none() {
            let current_chapter = GameUserData::get_chapter().cid.get_string().unwrap();
            if ( current_chapter != "CID_M022" && current_chapter != "CID_M011" ) && GameVariableManager::get_bool("G_Cleared_M004") {
                let emblem = rng.get_value(EMBLEMS.len() as i32) as usize;
                if try_equip_emblem(this, emblem) { adjust_emblem_unit_ai(this, data, emblem); }
            }
        } 
    }
    if str_contains(this.person.pid, "PID_M022_紋章士") { this.private_skill.add_sid("SID_死亡回避", 10, 0);  }  // Prevent Green Emblems from dying in Chapter 22 if AI is changed
    return;
 }
fn adjust_unit_ai(unit: &Unit, data: &mut DisposData) {
    let job = unit.get_job();
    let m022 = GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M022";
    let jid = job.jid.get_string().unwrap();
    let old_ai_names: [&Il2CppString; 4] = [data.ai_action_name, data.ai_mind_name, data.ai_attack_name, data.ai_move_name];
    let old_ai_values: [Option<&Il2CppString>; 4] = [data.ai_action_value, data.ai_mind_value, data.ai_attack_value, data.ai_move_value];
    if jid == "JID_ダンサー" {
        data.ai_mind_name = "AI_MI_Irregular".into();
        data.ai_action_name = "AI_AC_Everytime".into();
    }
    else if jid == "JID_エンチャント" {
        data.ai_attack_name = "AI_AT_Enchant".into();
        data.ai_attack_value = Some("".into());
    }
    // staff user, Chapter 22 needs to use Force due to Green Emblem Allies
    else if job.get_weapon_mask().value & ( 1 << 7 ) != 0 {
        if unit.item_list.has_item_iid("IID_ワープ") {
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
            if str_contains(data.ai_action_name, "AI_AC_AttackRange") {
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
            data.ai_attack_name =  "AI_AT_Attack".into();
            data.ai_attack_value = None;
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
        // No offensive staffs
        if str_contains(data.ai_action_name, "Interference") || str_contains(data.ai_attack_name, "Interference") {
            data.ai_action_name =  "AI_AC_Everytime".into();
            data.ai_action_value = None;
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
            data.ai_attack_value = None;
            data.ai_move_name =  "AI_MV_WeakEnemy".into();
        }
        if str_contains(data.ai_attack_name, "RodWarp") { 
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
            data.ai_attack_value = None;
        }
    }
    if m022 {
        data.ai_move_name = "AI_MV_ForceOnly".into();
        data.ai_move_value = Some("FORCE_PLAYER".into());
        data.ai_attack_value = Some("FORCE_PLAYER".into());
    }
    if data.get_flag().value & 16 != 0 ||  str_contains(old_ai_names[0], "Turn") { 
        data.ai_action_name = old_ai_names[0]; 
        data.ai_action_value = old_ai_values[0];
    }
    unsafe {
        let engage_atk_ai = crate::emblem::get_engage_attack_type(unit_get_engage_atk(unit, None));
        if engage_atk_ai != -1 {
            data.ai_attack_name = ENGAGE_ATK_AI[engage_atk_ai as usize].into();
            if engage_atk_ai == 4 { data.ai_attack_value = Some("255, 255, 3, 3".into()); }
            else if engage_atk_ai == 8 { data.ai_attack_value = Some("2, 2, 255, 255".into()); }
            else { data.ai_attack_value = Some("2,2".into()); }
            if str_contains(data.ai_action_name, "AC_Null") {  data.ai_action_name = "AI_AC_AttackRange".into(); }
        }
        unit_set_dispos_ai(unit, data, None); 
    }
    data.ai_action_name = old_ai_names[0];
    data.ai_mind_name = old_ai_names[1];
    data.ai_attack_name = old_ai_names[2];
    data.ai_move_name = old_ai_names[3];
    data.ai_action_value = old_ai_values[0];
    data.ai_mind_value = old_ai_values[1];
    data.ai_attack_value = old_ai_values[2];
    data.ai_move_value = old_ai_values[3];
}

pub fn adjust_unit_items(unit: &Unit) {
    let job = unit.get_job();
    let mut weapon_mask = job.get_weapon_mask().value | unit.selected_weapon_mask.value;
    if weapon_mask == 0 {  weapon_mask = unit.selected_weapon_mask.value; }
    let list_count = unit.item_list.get_count();
    let name =  unit.person.get_name().unwrap().get_string().unwrap();
    if list_count == 0 { return; }
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
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_some() {
            let weapon = &item.unwrap();
            //if weapon.is_drop() { continue; }
            let kind = weapon.item.get_kind(); 
            if kind > 8 || kind == 0 { continue; }
            if kind == 7 { continue; }
            if weapon.item.get_flag().value & 128 != 0 || weapon.item.get_flag().value & 2 != 0 { continue;  }
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
    item::remove_duplicates(unit.item_list);
}

#[unity::from_offset("App", "Unit", "UpdateStateWithAutoEquip")]
pub fn unit_update_auto_equip(this: &Unit, method_info: OptionalMethod);

#[skyline::from_offset(0x01f25ec0)]
fn get_bmap_size(this: &PersonData, method_info: OptionalMethod) -> u8;

#[unity::from_offset("App", "Unit", "SetDisposAi")]
pub fn unit_set_dispos_ai(this: &Unit, data: &mut DisposData, method_info: OptionalMethod);

fn has_skill(this: &Unit, skill: &SkillData) -> bool {
    return this.mask_skill.unwrap().find_sid(skill.sid).is_some() | this.private_skill.find_sid(skill.sid).is_some()| this.equip_skill.find_sid(skill.sid).is_some();
}
pub fn has_sid(this: &Unit, sid: &str) -> bool {
    return this.mask_skill.unwrap().find_sid(sid.into()).is_some() | this.private_skill.find_sid(sid.into()).is_some()| this.equip_skill.find_sid(sid.into()).is_some();
}

pub fn emblem_paralogue_level_adjustment(this: &Unit){
    if !crate::utils::can_rand() { return; }
    let level_difference = GameVariableManager::get_number("G_Paralogue_Level");
    if level_difference == 0 { return; }
    if level_difference < 0 {
        let old_level = this.level as i32;
        let level;
        if this.level == 1 { level = 2; }
        else { level = this.level as i32 ;  }
        let mut count = 0;
        let count_max = -1*(level_difference + level_difference / 3 );
        loop {
            this.level_down();
            this.set_level(level);
            count += 1;
            if count == count_max { break; }
        }
        if old_level + level_difference <= 0 { this.set_level(1); }
        else { this.set_level( (old_level + level_difference) as i32)}
    }
    else {
        let mut count = 0;
        loop {
            this.level_up(3);
            count += 1;
            if count == level_difference { break; }
        }
        let job_max_level = this.job.max_level as i32; 
        let level = this.level as i32; 
        if level > job_max_level {
            let new_internal_level = this.internal_level as i32 + job_max_level - level; 
            this.set_level(job_max_level);
            this.set_internal_level(new_internal_level);
        }
    }
    this.set_hp(this.get_capability(0, true));
}

pub struct RandomPersonMod;
impl ConfigBasicMenuItemSwitchMethods for RandomPersonMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_recruitment, 0, 3, 1);
        if CONFIG.lock().unwrap().random_recruitment != result {
            CONFIG.lock().unwrap().random_recruitment = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_recruitment {
            1 => { "Characters will be recruited in a random order." }, 
            3 => { "Unit recruitment order is determined by list."},
            2 => { "Characters will be recruited in reversed order."}
            _ => { "Standard recruitment order." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_recruitment {
            1 => { "Random"},
            3 => { "Custom"},
            2 => { "Reverse"},
            _ => { "Standard"},
        }.into();
    }
}
