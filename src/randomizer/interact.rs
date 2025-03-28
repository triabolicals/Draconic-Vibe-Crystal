use super::*;
use std::sync::OnceLock;
pub static INTERACT_DEFAULT: OnceLock<Vec<i32>> = OnceLock::new();

#[unity::class("App", "InteractData")]
pub struct InteractData {
    pub parent: StructBaseFields,
    pub kind: &'static Il2CppString,
    pub flag: &'static mut WeaponMask,
}
impl Gamedata for InteractData {}

pub struct InteractionSettings;
impl ConfigBasicMenuItemSwitchMethods for InteractionSettings {
    fn init_content(_this: &mut ConfigBasicMenuItem){
         GameVariableManager::make_entry("InteractSetting", GameVariableManager::get_number(DVCVariables::INTERACT_KEY) );
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value =  if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().interaction_type } else { GameVariableManager::get_number("InteractSetting") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 6, 1);
        if value != result {
            if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().interaction_type = result;  }
            else {  GameVariableManager::set_number("InteractSetting", result);  }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    } 
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().interaction_type } else { GameVariableManager::get_number("InteractSetting") };
        let string1: String = match value {
            1 => { "Reversed weapon type interactions." },
            2 => { "Same weapon type only interactions."},
            3 => { "Randomized weapon type interactions."},
            4 => { "Fire Emblem Fates weapon type interactions."},
            5 => { "No weapon type interactions (3H Style)."},
            6 => { "All weapon types interact with each other."},
            _ => { "Default weapon type interactions."},
        }.to_string();
        if GameVariableManager::get_number("InteractSetting") != GameVariableManager::get_number(DVCVariables::INTERACT_KEY) {  
            this.help_text = format!("{} (Press A to change)", string1).into();
        }
        else { this.help_text = string1.into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().interaction_type }
            else { GameVariableManager::get_number("InteractSetting") };
        let changed = DVCVariables::changed_setting_text("InteractSetting", DVCVariables::INTERACT_KEY);
        this.command_text = format!("{}{}", changed, interaction_setting_text( value )).into();
    }
}
fn interaction_setting_text(choice: i32) -> String {
    match choice {
        1 => { "Reverse" },
        2 => { "Self-Interact"},
        3 => { "Random"},
        4 => { "Fates"},
        5 => { "None"},
        6 => { "All"},
        _ => { "Default"},
    }.to_string()
}

pub fn interaction_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if DVCVariables::is_main_menu() {return BasicMenuResult::new(); }
    if GameVariableManager::get_number("InteractSetting") == GameVariableManager::get_number(DVCVariables::INTERACT_KEY) { return BasicMenuResult::new();}
    if GameVariableManager::get_number("InteractSetting") == 3 && !DVCVariables::random_enabled() { return BasicMenuResult::new(); }
    let text = format!("Change Weapon Interactions:\n\tFrom '{}' to '{}'?",
        interaction_setting_text( GameVariableManager::get_number(DVCVariables::INTERACT_KEY)), 
        interaction_setting_text( GameVariableManager::get_number("InteractSetting")), 
    );
    YesNoDialog::bind::<InteractionConfirm>(this.menu, text, "Do it!", "Nah..");
    return BasicMenuResult::new();
}
pub struct InteractionConfirm;
impl TwoChoiceDialogMethods for InteractionConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number(DVCVariables::INTERACT_KEY, GameVariableManager::get_number("InteractSetting"));
        change_interaction_data( GameVariableManager::get_number("InteractSetting"), false);
        unsafe { 
            let menu = std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap());
            let index = menu.select_index;
            InteractionSettings::set_help_text(menu.menu_item_list[index as usize], None);
            InteractionSettings::set_command_text(menu.menu_item_list[index as usize], None);
            menu.menu_item_list[index as usize].update_text();
        }
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
} 

pub extern "C" fn vibe_interaction() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<InteractionSettings>("Weapon Triangle Setting");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::build_attribute_normal as _);
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = interaction_setting_acall as _ );
    switch
}


pub fn change_interaction_data(choice: i32, loaded: bool) {
    if loaded && choice == 0 { return; }
    let interact_data = InteractData::get_list_mut().unwrap();
    println!("Change Interaction to {}", choice);
    let interact = INTERACT_DEFAULT.get().unwrap();
    match choice {
        1 => {  //Reverse
            for x in 0..10 {
                interact_data[x as usize].flag.value = interact[ 10 + x as usize ];
                interact_data[ x as usize + 10 ].flag.value = interact[ x as usize ];
            }
        },
        2 => {  //Self-Interaction
            for x in 0..10 {
                interact_data[x as usize].flag.value =  ( 1 << x ) + ( 1 << (x + 10) );
                interact_data[ x as usize  + 10  ].flag.value =  ( 1 << x ) + ( 1 << (x + 10) );
            } 
        },
        3 => {  // Random 
            if !DVCVariables::random_enabled() { return; }
            let rng = crate::utils::get_rng();
            for x in 0..20 {
                if x % 10 == 0 { continue; }
                let mut chance = 100;
                let mut value: i32 = 0;
                let mut set: [bool; 20] = [true, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false];
                loop {  // for advantages
                    if chance < rng.get_value(100) { break; }
                    let interact = rng.get_value(10);
                    if set[interact as usize] { continue; }
                    value |= 1 << interact;
                    chance = chance / 2;
                    set[interact as usize] = true;
                }
                chance = 100;   
                loop {  // for disadvantage
                    if chance < rng.get_value(100)  { break; }
                    let interact = rng.get_value(10) + 10;
                    if set[interact as usize] { continue; }
                    value |=  1 << interact ;
                    chance = chance / 2;
                    set[interact as usize] = true;
                }
                interact_data[x as usize].flag.value = value;
            }
        },
        4 => {  // Fates Weapon Triangle
            let values = [0, 36888, 24642, 67620, 329764, 286786, 299032, 0, 112, 0, 0, 24612, 67608, 36930, 37186, 67864, 24868, 0, 114688, 0];
            for x in 0..20 { interact_data[x as usize].flag.value = values[x as usize];  }
        },
        5 => { for x in 0..20 { interact_data[x as usize].flag.value = 0; } },
        6 => { 
            for x in 1..10 { 
                interact_data[x as usize].flag.value = -1;
                interact_data[ x as usize + 10].flag.value = -1;
            }
        },
        _ => {
            for x in 0..20 { interact_data[x as usize].flag.value = interact[x as usize] as i32;  }
        },
    }
}

pub fn get_style_interact_default_values() {
    super::styles::BATTLE_STYLES_DEFAULT.get_or_init(||{
        let mut list: Vec<i32> = Vec::new();
        list.push(-1);
        let job_list = JobData::get_list().unwrap();
        for x in 1..job_list.len() {
            let style_name = job_list[x].style_name.to_string();
            if let Some(pos) = crate::enums::STYLE_NAMES.iter().position(|&x| x == style_name) {
                list.push(pos as i32);
            }
            else {
                list.push(-1);
            }
        }

        list
    });
    INTERACT_DEFAULT.get_or_init(||{
        let mut list: Vec<i32> = Vec::new();
        InteractData::get_list().unwrap().iter().for_each(|data| list.push(data.flag.value));
        list
    });
}

