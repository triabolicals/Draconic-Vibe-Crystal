use unity::prelude::*;
use engage::{
    dialog::yesno::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gamedata::{unit::*, *},
    gameuserdata::GameUserData,
};
use std::sync::Mutex;
use super::CONFIG;
pub static INTERACT_DEFAULT: Mutex<[i32; 30]> = Mutex::new([0; 30]);
pub static BATTLE_STYLES_DEFAULT: Mutex<[i32; 256]> = Mutex::new([0; 256]); 

#[unity::class("App", "InteractData")]
pub struct InteractData {
    pub parent: StructBaseFields,
    pub kind: &'static Il2CppString,
    pub flag: &'static mut WeaponMask,
}
impl Gamedata for InteractData {}

pub fn get_style_interact_default_values() {
    for x in 1..200 { if BATTLE_STYLES_DEFAULT.lock().unwrap()[x] > 0 { return; }  } // already set
    let job_list = JobData::get_list_mut().unwrap();
    for x in 1..job_list.len() {
        let style_name = job_list[x].style_name.get_string().unwrap();
        let pos = crate::enums::STYLE_NAMES.iter().position(|&x| x == style_name);
        if pos.is_some() { BATTLE_STYLES_DEFAULT.lock().unwrap()[x] = pos.unwrap() as i32; }
        else { BATTLE_STYLES_DEFAULT.lock().unwrap()[x] = -1; }
    }
    let interact_data = InteractData::get_list().unwrap();
    for x in 0..interact_data.len() { INTERACT_DEFAULT.lock().unwrap()[x] = interact_data[x].flag.value; }

}
pub fn randomize_person_grow(){
    let mut max: [u8; 11] = [0; 11];
    let mut min: [u8; 11] = [100; 11];
    let person_list = PersonData::get_list_mut().unwrap();
    for x in 0..person_list.len() {
        let grow = person_list[x].get_grow();
        if grow.is_zero() { continue; }
        for y in 0..11 {
            if grow[y as usize] > max[y as usize] { max[ y as usize ] = grow[y as usize]; }
            if grow[y as usize] < min[y as usize] { min[ y as usize] = grow[y as usize]; }
        }
    }
    for y in 0..11 {
        max[y as usize] = max[y as usize] / 5;
        min[y as usize] = min[y as usize] / 5;
    }
    let rng = crate::utils::get_rng();
    for x in 0..person_list.len() {
        let grow = person_list[x].get_grow();
        if grow.is_zero() { continue; } 
        for y in 0..11 {
            let v = rng.get_min_max(min[y as usize] as i32, max[y as usize] as i32) as u8;
            if person_list[x].get_asset_force() != 0 {
                grow[y as usize] = (v+2)*5;
            }
            else { grow[y as usize] = v*5; }
        }
    }
}

pub fn randomize_job_grow(){
    let mut max: [i8; 11] = [0; 11];
    let mut min: [i8; 11] = [100; 11];
    let job_list = JobData::get_list_mut().unwrap();
    for x in 0..job_list.len() {
        let grow = job_list[x].get_diff_grow();
        if grow.is_zero() { continue; } 
        for y in 0..11 {
            if grow[y as usize] > max[y as usize] { max[ y as usize ] = grow[y as usize]; }
            if grow[y as usize] < min[y as usize] { min[ y as usize] = grow[y as usize]; }
        }
    }
    for y in 0..11 {
        max[y as usize] = max[y as usize] / 5 + 1;
        min[y as usize] = min[y as usize] / 5;
    }
    let rng = crate::utils::get_rng();
    for x in 0..job_list.len() {
        let grow = job_list[x].get_diff_grow();
        if grow.is_zero() { continue; } 
        for y in 0..11 {
            let v = rng.get_min_max(min[y as usize] as i32, (max[y as usize]) as i32) as i8;
            grow[y as usize] = (v+1)*5;
        }
    }
}
pub fn randomize_job_styles(){
    let job_list = JobData::get_list_mut().unwrap();
    let rng = crate::utils::get_rng();
    match GameVariableManager::get_number("G_BattleStyles") {
        1 => {  // Random
            if !crate::utils::can_rand() { return; }
            for x in 1..job_list.len() {
                let style = crate::enums::STYLE_NAMES[ rng.get_value(8) as usize ];
                job_list[x].style_name = style.into();
                job_list[x].on_completed();
            }
        },
        2 => {  // None
            for x in 1..job_list.len() {
                job_list[x].style_name = "スタイル無し".into();
                job_list[x].on_completed();
            }
        },
        0 => {  //Default
            println!("Reseting Job Styles");
            for x in 1..job_list.len() {
                let index = BATTLE_STYLES_DEFAULT.lock().unwrap()[x];
                if index == -1 { continue; }
                job_list[x].style_name = crate::enums::STYLE_NAMES[index as usize].into();
                job_list[x].on_completed();
            }
        },
        _ => {},
    }
}

pub fn random_grow(){
    if !crate::utils::can_rand() { return; }
    if GameVariableManager::get_number("G_Random_Seed") == 0 { return; }
    match GameVariableManager::get_number("G_Random_Grow_Mode") {
        1 => { randomize_person_grow(); },
        2 => { randomize_job_grow(); }, 
        3 => { 
            randomize_person_grow();
            randomize_job_grow(); },
        _ => {},
    }
}
#[skyline::from_offset(0x01a3c290)]
fn unit_learn_job_skill(this: &Unit, method_info: OptionalMethod) -> &'static engage::gamedata::skill::SkillData;

pub fn change_unit_autolevel(unit: &mut Unit, reverse: bool) {
    let person = if reverse { crate::person::switch_person_reverse(unit.person) } else { &unit.person };
    let new_person = if reverse { &unit.person } else { crate::person::switch_person(unit.person) }; 
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
    unit.set_person(person);
    unit.class_change(person.get_job().unwrap());
    if is_low {
       if current_level > 20 { //Old Unit is in a special class so new unit needs to be promoted
            if is_new_low && new_person.get_job().unwrap().has_high() {    // new unpromoted unit can promoted 
                let level = current_level - 20;
                let new_job = &new_person.get_job().unwrap().get_high_jobs()[0];
                unit.auto_grow_capability( level, current_level);
                unit.class_change(new_job);
                unit.set_level( level );
                unit.set_internal_level( 20 );
            }
            else if is_new_low && !new_person.get_job().unwrap().has_high() {   // special -> special
                unit.class_change(new_person.get_job().unwrap());
                unit.auto_grow_capability( current_level, current_level);
                unit.set_level( current_level );
                unit.set_internal_level( 0 );
            }
            else {  // special -> high
                unit.class_change(new_person.get_job().unwrap());
                unit.auto_grow_capability( current_level-20, current_level);
                unit.set_level( current_level - 20 );
                unit.set_internal_level( 20 );
            }
        }
        if is_new_low { // base or special class lvl < 20 -> base class
            unit.class_change(new_person.get_job().unwrap());
            unit.auto_grow_capability( current_level, current_level);
            unit.set_level( current_level );
            unit.set_internal_level( 0 );
            println!("{} -> {} Base -> Base Level {}",  person.get_name().unwrap().get_string().unwrap(), new_person.get_name().unwrap().get_string().unwrap(), current_level);
        }
        else {
            let new_job_list = new_person.get_job().unwrap().get_low_jobs();
            unit.auto_grow_capability(current_level, current_level);
            if new_job_list.len() == 3 {
                let index = crate::person::get_low_class_index(new_person);
                unit.class_change(&new_job_list[index as usize]);
            }
            else if new_job_list.len() == 0 { unit.class_change(JobData::get("JID_ソードファイター").unwrap()); }    // if promoted class doesn't have a low class, change to sword fighter
            else {  unit.class_change(&new_job_list[0]); }
            unit.set_level(current_level);
            unit.set_internal_level(0);
        }
    }
    else {  // Promoted
        if is_new_low { // new unit has a base class
            if new_person.get_job().unwrap().has_high() {   // base class -> 1st promoted class
                let new_high_jobs = new_person.get_job().unwrap().get_high_jobs();
                if new_high_jobs.len() == 0 { unit.class_change(JobData::get("JID_ソードマスター").unwrap());  } // if no high class, change to Swordmaster
                else { unit.class_change(&new_high_jobs[0]); }
                unit.auto_grow_capability(current_level, current_level + 20);
                unit.set_level(current_level);
                unit.set_internal_level(current_internal_level);
                println!("Promoted Unit -> Base Unit");
            }
            else { // Promoted -> Special
                if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  { 
                    unit.auto_grow_capability(current_level, current_level + 20);
                    unit.set_level(current_level);
                    unit.set_internal_level( current_internal_level );
                    println!("Promoted Unit -> Special Unit but promoted");
                } 
                else {
                    let total_level = current_internal_level + current_level;
                    unit.class_change(new_person.get_job().unwrap());
                    unit.auto_grow_capability(total_level, 20+current_level);
                    unit.set_level(total_level);
                    unit.set_internal_level(0);
                    unit.set_level( ( person.get_level() + person.get_internal_level() as u8 ).into() );
                    println!("Promoted Unit -> Special Unit");
                }
            }
        }
        else {  // Promoted -> Promoted
            unit.class_change(new_person.get_job().unwrap());
            unit.auto_grow_capability(current_level, current_level + 20);
            unit.set_level(current_level);
            unit.set_internal_level( current_internal_level );
        }
    }
    for x in 0..11 { original_gr[x as usize] = original_growth_rates[x as usize]; } // Change back to original growth rate
    unit.set_sp( person.get_sp() );
    unit.set_person(new_person);    // change person
    crate::person::fixed_unit_weapon_mask(unit);   // fixed weapon mask due to class changes
    unit.set_hp(unit.get_capability(0, true));
    unsafe {  unit_learn_job_skill(unit, None); }
}

pub fn change_interaction_data(choice: i32) {
    let interact_data = InteractData::get_list_mut().unwrap();
    println!("Change Interaction to {}", choice);
    match choice {
        1 => {  //Reverse
            for x in 0..10 {
                interact_data[x as usize].flag.value = INTERACT_DEFAULT.lock().unwrap()[( 10 + x as usize) ];
                interact_data[ (x + 10 as usize) ].flag.value = INTERACT_DEFAULT.lock().unwrap()[ x as usize ];
            }
        },
        2 => {  //Self-Interaction
            for x in 0..10 {
                interact_data[x as usize].flag.value =  ( 1 << x ) + ( 1 << (x + 10) );
                interact_data[ ( x + 10  as usize ) ].flag.value =  ( 1 << x ) + ( 1 << (x + 10) );
            } 
        },
        3 => {  // Random 
            if !crate::utils::can_rand() { return; }
            let rng = crate::utils::get_rng();
            let kinds = ["None", "Sword", "Lance", "Axe", "Bow", "Dagger", "Tome", "Rod", "Arts", "Special"];
            for x in 0..20 {
                if x % 10 == 0 { continue; }
                let mut chance = 100;
                let mut value: i32 = 0;
                let mut set: [bool; 20] = [true, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false];
                loop {  // for advantages
                    if chance < rng.get_value(100) { break; }
                    let interact = rng.get_value(10);
                    if set[interact as usize] { continue; }
                    else {
                        value |= ( 1 << interact );
                        chance = chance / 2;
                        set[interact as usize] = true;
                        println!("Weapon {} is Strong against {}", kinds[x as usize], kinds[interact as usize]);
                    }
                }
                chance = 120;   
                loop {  // for disadvantage
                    if chance < rng.get_value(100)  { break; }
                    let interact = rng.get_value(10) + 10;
                    if set[interact as usize] { continue; }
                    else {
                        value |= ( 1 << interact );
                        chance = chance / 2;
                        set[interact as usize] = true;
                        println!("Weapon {} is Weak against {}", kinds[x as usize], kinds[ (interact - 10 )  as usize]);
                    }
                }
                interact_data[x as usize].flag.value = value;
            }
        },
        4 => { for x in 0..20 { interact_data[x as usize].flag.value = 0; } },
        5 => { 
            for x in 1..10 { 
            interact_data[x as usize].flag.value = -1;
            interact_data[ (x + 10 as usize) ].flag.value = -1;
            }
        },
        _ => {
            for x in 0..20 { interact_data[x as usize].flag.value = INTERACT_DEFAULT.lock().unwrap()[x as usize] as i32;  }
        },
    }
}

pub struct RandomGrowMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGrowMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_grow, 0, 3, 1);
        if CONFIG.lock().unwrap().random_grow != result {
            CONFIG.lock().unwrap().random_grow = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_grow {
            1 => { "Personal growths rate will be randomized."},
            2 => { "Class growth rates modifiers will be randomized."},
            3 => { "Personal and class growth rates modifiers will be randomized." },
            _ => { "No changes made to growth rates." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text =  match CONFIG.lock().unwrap().random_grow {
            1 => { "Personal Growths" },
            2 => { "Class Growths" },
            3 => { "Personal / Class" },
            _ => { "Disable" },
        }.into();
    }
}
pub struct RandomBattleStyles;
impl ConfigBasicMenuItemSwitchMethods for RandomBattleStyles {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("BattleStyles", GameVariableManager::get_number("G_BattleStyles") );
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles }
                    else { GameVariableManager::get_number("BattleStyles") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles = result; }
            else { GameVariableManager::set_number("BattleStyles", result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().random_battle_styles } else { GameVariableManager::get_number("BattleStyles") };
        let string1 = match value {
            1 => { "Class types will be randomized." },
            2 => { "Classes will have no special types."},
            _ => { "Classes will have their default type."},
        }.to_string();
        if GameVariableManager::get_number("G_BattleStyles") != GameVariableManager::get_number("BattleStyles") {
            this.help_text = format!("{} (Press A to change)", string1).into();
        }
        else { this.help_text = string1.into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles }
                    else { GameVariableManager::get_number("BattleStyles") };
        this.command_text = match value {
            1 => { "Random" },
            2 => { "No Types"},
            _ => { "Default"},
        }.into();
    }
}

pub struct InteractionSettings;
impl ConfigBasicMenuItemSwitchMethods for InteractionSettings {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("InteractSetting", GameVariableManager::get_number("G_InteractSetting") );
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value =  if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().interaction_type } else { GameVariableManager::get_number("InteractSetting") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 5, 1);
        if value != result {
            if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().interaction_type = result;  }
            else {  GameVariableManager::set_number("InteractSetting", result);  }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    } 
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().interaction_type } else { GameVariableManager::get_number("InteractSetting") };
        let string1: String = match value {
            1 => { "Reversed weapon type interactions." },
            2 => { "Same weapon type only interactions."},
            3 => { "Randomized weapon type interactions."},
            4 => { "No weapon type interactions."},
            5 => { "All weapon type interact with each other."},
            _ => { "Default weapon interactions."},
        }.to_string();
        if GameVariableManager::get_number("InteractSetting") != GameVariableManager::get_number("G_InteractSetting") {  
            this.help_text = format!("{} (Press A to change)", string1).into();
        }
        else { this.help_text = string1.into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().interaction_type }
                    else { GameVariableManager::get_number("InteractSetting") };
        this.command_text = interaction_setting_text( value ).into();
    }
}

pub fn interaction_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 0 {return BasicMenuResult::new(); }
    if GameVariableManager::get_number("InteractSetting") == GameVariableManager::get_number("G_InteractSetting") { return BasicMenuResult::new();}
    let text = format!("Change Weapon Interactions:\n\tFrom '{}' to '{}'?",
        interaction_setting_text( GameVariableManager::get_number("G_InteractSetting")), 
        interaction_setting_text( GameVariableManager::get_number("InteractSetting")), 
    );
    YesNoDialog::bind::<InteractionConfirm>(this.menu, text, "Do it!", "Nah..");
    return BasicMenuResult::new();
}
pub fn battle_style_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 0 {return BasicMenuResult::new(); }
    if GameVariableManager::get_number("G_BattleStyles") == GameVariableManager::get_number("BattleStyles") { return BasicMenuResult::new();}

    let text = format!("Change Class Type Setting:\nFrom '{}' to '{}'?",
        style_setting_text( GameVariableManager::get_number("G_BattleStyles")), 
        style_setting_text( GameVariableManager::get_number("BattleStyles")), 
    );
    YesNoDialog::bind::<BattleStyleConfirm>(this.menu, text, "Do it!", "Nah..");
    return BasicMenuResult::new();
}
fn interaction_setting_text(choice: i32) -> String {
    match choice {
        1 => { "Reverse" },
        2 => { "Self-Interact"},
        3 => { "Random"},
        4 => { "None"},
        5 => { "All"},
        _ => { "Default"},
    }.to_string()
}
fn style_setting_text(choice: i32) -> String {
    match choice {
        1 => {"Randomized"},
        2 => {"No Types"},
        _ => { "Default"},
    }.to_string()
}
pub struct InteractionConfirm;
impl TwoChoiceDialogMethods for InteractionConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number("G_InteractSetting", GameVariableManager::get_number("InteractSetting"));
        change_interaction_data( GameVariableManager::get_number("G_InteractSetting") );
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub struct BattleStyleConfirm;
impl TwoChoiceDialogMethods for BattleStyleConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number("G_BattleStyles", GameVariableManager::get_number("BattleStyles"));
        randomize_job_styles();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}