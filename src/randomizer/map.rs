use super::*;
use engage::{
    force::*, gamedata::terrain::TerrainData, 
    gamemessage::GameMessage, gamesound::GameSound, godpool::GodPool, 
    map::{overlap::MapOverlap, terrain::MapTerrain}, mapmind::MapMind, 
    script::*, sequence::unitgrowsequence::UnitGrowSequence, 
    unitpool::UnitPool, 
};
use std::sync::Mutex;
static SCRIPT_COMMANDS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub mod effects;

pub struct MapTileRandomizer;
impl ConfigBasicMenuItemSwitchMethods for  MapTileRandomizer {
    fn init_content(_this: &mut ConfigBasicMenuItem){ GameVariableManager::make_entry(DVCVariables::TERRAIN, 0); }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().tile } else { GameVariableManager::get_bool(DVCVariables::TILE) };
        let result =  ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().tile = result; }
            else { GameVariableManager::set_bool(DVCVariables::TILE, result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().tile } else { GameVariableManager::get_bool(DVCVariables::TILE) };
        this.help_text = 
            if value { "Additional map event tiles will produce a random effect." }
            else { "No additional map event tiles are added."}.into();

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().tile } else { GameVariableManager::get_bool(DVCVariables::TILE) };
        this.command_text = if value { "Enabled" } else { "Disabled" }.into();
    }
}

pub extern "C" fn vibe_tile() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<MapTileRandomizer>("Tilebolical");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::not_in_map_build_attr as _);
    switch
}

fn register_action(script: &EventScript, name: &str, action: extern "C" fn(&Il2CppArray<DynValue>, OptionalMethod)){
    EventScript::register_action(script, name, action);
    let lock = &mut SCRIPT_COMMANDS.lock().unwrap();
    if lock.iter().find(|&x| x == name).is_none() { lock.push(name.to_string()); }
}

pub extern "C" fn register_script_commands(script: &EventScript) {
    println!("Installing DVC Lua Commands");
    effects::install_tilebolical_effects(script);
    GameVariableManager::make_entry_norewind(DVCVariables::TILE, 0);
    super::RANDOMIZER_STATUS.try_write().map(|mut lock| { lock.map_tile = false;   }  ).unwrap();
    if CONFIG.lock().unwrap().debug {
        Force::get(ForceType::Player).unwrap().iter().chain( Force::get(ForceType::Absent ).unwrap().iter())
            .for_each(|unit|{
                unit.set_base_capability(0, 50);
                unit.set_base_capability(10, 50);
                unit.set_base_capability(9, 50);
                unit.set_base_capability(7, 25);
                unit.set_base_capability(8, 25);
                unit.set_base_capability(6, 25);
                unit.set_base_capability(2, 50);
                unit.set_base_capability(3, 50);
                unit.set_base_capability(4, 50);
                unit.set_hp(unit.get_capability(0, true));
            }
        );
    }
    /*
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
        EventScript::register_action(script, "UnitJoin", crate::script::unit_join);
    }
    */
}   


pub fn tilabolical() {
    if !GameVariableManager::get_bool(DVCVariables::TILE) { return; }
    if super::RANDOMIZER_STATUS.read().unwrap().map_tile { return; }
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 7 {
        if let Some(terrain) = MapTerrain::get_instance() {
            println!("Initializing Tilebolical");
            super::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.map_tile = true).unwrap();
            let start_x = terrain.x;
            let end_x = terrain.width;
            let start_z = terrain.z;
            let end_z = terrain.height;
            let pillars = TerrainData::get("TID_ブロック").unwrap();
            let rng = Random::get_system();
            let array: &mut Array<_> = Array::from_slice(vec![ DynValue::new_number(0.0), DynValue::new_number(0.0), DynValue::new_number(0.0) ]).unwrap();
            let ncommands = SCRIPT_COMMANDS.lock().unwrap().len();
            let mut selection: Vec<_> = (0..ncommands).collect();        
            let rate = if CONFIG.lock().unwrap().debug { 5 } else { 1 };    
            for z in start_z..end_z {
                for x in start_x..end_x {
                    if MapOverlap::can_create(None, x, z, pillars) && rng.get_value(20) < rate {
                        array[1].number = x as f64;
                        array[2].number = z as f64;
                        let slen = selection.len();
                        if slen > 0 {
                            let i = rng.get_value(slen as i32) as usize;
                            let index = selection[ i ];
                            let name = &SCRIPT_COMMANDS.lock().unwrap()[ index ];
                            println!("Effect Added: {}, at {}, {}", name, x, z);
                            if let Some(func) = EventScript::get_func(name.as_str()) {
                                array[0] = func;
                                unsafe { tbox_entry(array, None); }
                            }
                            selection.remove(i );
                        }
                    }
                }
            }
        }
   }
}

pub fn remove_map_effects() {
    let double_exp = "SID_経験値２倍".into();
    let corrupted = "SID_異形兵".into();
    Il2CppClass::from_name("App", "UnitPool").unwrap().get_static_fields_mut::<crate::randomizer::job::UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.is_some_and(|f| f.force_type == 0 || f.force_type == 3))
        .for_each(|unit|{
            unit.hp_stock_count = 0;
            unit.hp_stock_count_max = 0;
            unit.extra_hp_stock_count = 0;
            unit.extra_hp_stock_count_max = 0;
            unit.private_skill.remove_sid(double_exp);
            unit.private_skill.remove_sid(corrupted);
        }
    );
}

#[skyline::from_offset(0x01ed2e30)]
fn tbox_entry(args: &Array<&mut DynValue>, method_info: OptionalMethod);

#[skyline::from_offset(0x01dbb6c0)]
fn unit_map_effect(name: &Il2CppString, unit: &Unit, method_info: OptionalMethod); 

#[skyline::from_offset(0x01c76a90)]
fn vision_create(this: &Unit, method_info: OptionalMethod);
