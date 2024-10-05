pub use unity::prelude::*;
pub use engage::{
    menu::{
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    gamevariable::*, gameuserdata::*, hub::access::*, random::*, mess::*,
    gamedata::{*, item::*, skill::SkillData, dispos::*, unit::*},
};
use crate::{enums::*, utils::*, CONFIG};

pub struct RandomNameMods;
impl ConfigBasicMenuItemSwitchMethods for RandomNameMods {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_names);
        if CONFIG.lock().unwrap().random_names!= result {
            CONFIG.lock().unwrap().random_names  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().random_names {"Characters will have random names and appearances if possible." }
            else { "Characters will have their default name and appearances." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().random_names { "Randomized" }
            else { "Default" }.into();
    }
}

#[unity::from_offset("App", "PersonData", "set_Belong")]
fn set_belong_person(this: &PersonData, value: Option<&Il2CppString>, method_info: OptionalMethod);

#[unity::from_offset("App", "PersonData", "set_Aid")]
fn set_aid_person(this: &PersonData, value: Option<&Il2CppString>, method_info: OptionalMethod);

pub fn set_generic_aid(this: &PersonData) {
    let name = this.get_name().unwrap().get_string().unwrap();
    let bid; 
    let aid;
    match name.as_str() {
        "MPID_FileneVillager"|"MPID_FileneSoldier" => { 
            aid = "AID_一般兵";
            bid = "BID_フィレネ"; 
        },
        "MPID_BrodiaSoldier" => { 
            aid = "AID_一般兵";
            bid = "BID_ブロディア";
         },
        "MPID_MysteriousGroup"|"MPID_IrcionSoldier" => { 
            aid = "AID_一般兵";
            bid = "BID_イルシオン";
        },
        "MPID_SolumVillager"|"MPID_SolumSoldier" => { 
            aid = "AID_一般兵";
            bid = "BID_ソルム"; 
        },
        "MPID_Morph" => { 
            aid = "AID_異形兵";
            bid = "BID_異形"; 
        },
        "MPID_Phantom" => { 
            aid = "AID_幻影兵";
            bid = "BID_幻影"; 
        },
        _ => { return; },
    };
    unsafe {
        set_aid_person(this, Some(aid.into()), None);
        set_belong_person(this, Some(bid.into()), None);
    }

}

pub fn give_names_to_generics() {
    if !crate::utils::can_rand() { return; }
    let mut male_names: Vec<String> = Vec::new();
    let mut female_names: Vec<String> = Vec::new();
    let list = PersonData::get_list().unwrap();
    let rng = get_rng();

    for x in 1..1100 {
        let gender = list[x].get_gender();
        if gender != 1 && gender != 2 { continue; }
        if list[x].get_name().is_none() { continue; }
        if list[x].get_job().is_none() { continue; }
        let name = list[x].get_name().unwrap().get_string().unwrap();
        if name == "MPID_Hide" || name == "MPID_SombreDragon" { continue; }
        if gender == 1 {
            if male_names.iter().find(|&s| *s == name).is_none() { male_names.push(name); }
        }
        else {
            if female_names.iter().find(|&s| *s == name).is_none() { female_names.push(name);  }
        }
    }
    let size_m =  male_names.len() as i32;
    let size_f =  female_names.len() as i32;
    if GameVariableManager::get_bool("G_Random_Names") {
        for x in 2..list.len() {
            if list[x].get_name().is_none() { continue; }
            if list[x].get_job().is_none() { continue; }
            let gender = list[x].get_gender();
            if gender == 1 {
                list[x].set_name( male_names[ rng.get_value(size_m) as usize ].clone().into() );
            }
            else if gender == 2 {
                list[x].set_name(  female_names[ rng.get_value(size_f) as usize ].clone().into() );
            }
            if str_contains(list[x].get_job().unwrap().jid, "JID_紋章士_") {
                unsafe { set_aid_person(list[x], None, None)};
            }
            else { set_generic_aid(list[x]); }
            list[x].on_completed();
        }
        // Selected Alear
        let lueur = unsafe { crate::deployment::force_get_unit_from_pid(PIDS[0].into(), true, None) };
        if lueur.is_some() {
            let gender = lueur.unwrap().edit.gender;
            if gender == 1 {
                list[1].set_name( male_names[ rng.get_value(size_m) as usize ].clone().into() );
            }
            else if gender == 2 {
                list[1].set_name(   female_names[ rng.get_value(size_f) as usize ].clone().into() );
            }
            set_generic_aid(list[1]); 
        }
    }
    else if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        for x in 2..list.len() {
            if list[x].get_name().is_none() { continue; }
            let name = list[x].get_name().unwrap().get_string().unwrap();
            if name != "MPID_Phantom" && name != "MPID_Morph" { continue; }
            unsafe { set_aid_person(list[x], None, None) };
            let gender = list[x].get_gender();
            if gender == 1 {
                list[x].set_name(male_names[ rng.get_value(size_m) as usize ].clone().into() );
            }
            else if gender == 2 {
                list[x].set_name(  female_names[ rng.get_value(size_f) as usize ].clone().into() );
            }
            set_generic_aid(list[x]); 
            list[x].on_completed();
        }
    }
}