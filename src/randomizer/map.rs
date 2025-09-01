use super::*;
use engage::{
    force::*, gamedata::terrain::TerrainData, 
    gamemessage::GameMessage, gamesound::GameSound, godpool::GodPool, 
    map::{overlap::MapOverlap, terrain::MapTerrain}, mapmind::MapMind, 
    script::*, sequence::unitgrowsequence::UnitGrowSequence, 
    unitpool::UnitPool, 
};
use std::sync::Mutex;

#[repr(i32)]
#[derive(PartialEq, Clone)]
pub enum EffectType {
    NoEffect,
    Skill,
    HP,
    Level,
    SP,
    Stat,
    Emblem,
    Other,
    Exp,
    Class,
    GoldItem,
    SpawnUnit,
}
impl EffectType {
    pub fn can_repeat(&self) -> bool {
        match self {
            EffectType::SpawnUnit|EffectType::Stat|EffectType::Class|EffectType::GoldItem => { true }
            _ => { false }
        }
    }
}
static SCRIPT_COMMANDS: Mutex<Vec<(String, EffectType)>> = Mutex::new(Vec::new());
pub const EXTRA_SIDS: &[&str] = &[
    "SID_手加減", "SID_不死身", "SID_異形兵", "SID_慈悲", "SID_チェインアタック許可",
];
pub const SKILL_SIDS: &[&str] = &[
    "SID_慈悲",   // Enemy Only   2
    "SID_デュアルアシスト＋",  //Dual Assist
    "SID_努力の才", "SID_経験値２倍", "SID_残像", "SID_竜脈", "SID_杖使い＋＋", "SID_先生", "SID_血統", "SID_血讐＋", "SID_竜脈・異",
    "SID_迅走", "SID_再移動＋", "SID_天刻の拍動＋", "SID_踏ん張り", "SID_ブレイク時追撃", "SID_勇将", "SID_日月の腕輪",
    "SID_切り返し", "SID_血讐", "SID_契約", "SID_囮指名", "SID_七色の叫び", "SID_絆盾_気功",
];
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
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().tile } else { GameVariableManager::get_bool(DVCVariables::TILE) };
        this.command_text = if value { "Enabled" } else { "Disabled" }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().tile }
        else { GameVariableManager::get_bool(DVCVariables::TILE) };
        this.help_text =
            if value { "Additional map event tiles will produce a random effect." }
            else { "No additional map event tiles are added."}.into();
    }
}

pub extern "C" fn vibe_tile() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<MapTileRandomizer>("Tilebolical");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute")
        .map(|method| method.method_ptr = crate::menus::buildattr::not_in_map_build_attr as _);
    switch
}

fn register_action(script: &EventScript, name: &str, action: extern "C" fn(&Il2CppArray<DynValue>, OptionalMethod), ty: EffectType) {
    EventScript::register_action(script, name, action);
    let lock = &mut SCRIPT_COMMANDS.lock().unwrap();
    if lock.iter().find(|&x| x.0 == name).is_none() {
        lock.push((name.to_string(), ty));
    }
}

pub extern "C" fn register_script_commands(script: &EventScript) {
    println!("Installing DVC Lua Commands");
    effects::install_tilebolical_effects(script);
    GameVariableManager::make_entry_norewind(DVCVariables::TILE, 0);
}   


pub fn tilabolical() {
    if RANDOMIZER_STATUS.read().unwrap().map_tile { return; }
    if !GameVariableManager::get_bool(DVCVariables::TILE) { return; }
    if CONFIG.lock().unwrap().debug {
        Force::get(ForceType::Player).unwrap().iter().for_each(|u|{
            u.private_skill.add_sid("SID_迅走", 10, 0);
        });
    }
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 7 {
        GameVariableManager::find_starts_with("Tile").iter().for_each(|u|{
            GameVariableManager::remove(u.to_string());
        });
        GameVariableManager::make_entry("TileSkills", 0);
        if let Some(terrain) = MapTerrain::get_instance() {
            println!("Initializing Tilebolical");
            RANDOMIZER_STATUS.try_write().map(|mut lock| lock.map_tile = true).unwrap();
            let start_x = terrain.x;
            let end_x = terrain.width;
            let start_z = terrain.z;
            let end_z = terrain.height;
            let pillars = TerrainData::get("TID_ブロック").unwrap();
            let rng = Random::get_system();
            let array: &mut Array<_> = Array::from_slice(
                vec![
                    DynValue::new_number(0.0),
                    DynValue::new_number(0.0),
                    DynValue::new_number(0.0)
                ]
            ).unwrap();
            let ncommands = SCRIPT_COMMANDS.lock().unwrap().len();
            let mut selection: Vec<_> = (0..ncommands).collect();
            let skill_len = SKILL_SIDS.len();
            let mut skill_count = 0;
            let rate = if CONFIG.lock().unwrap().debug { 45 } else { 10 };
            for z in start_z..end_z {
                for x in start_x..end_x {
                    if can_add_inspector(x, z) && MapOverlap::can_create(None, x, z, pillars) && rng.get_value(100) < rate {
                        array[1].number = x as f64;
                        array[2].number = z as f64;
                        let slen = 3*selection.len() + skill_len;
                        let i = rng.get_value(slen as i32) as usize;
                        if i < skill_len && skill_count < skill_len {
                            if let Some(func) = EventScript::get_func("RandomSkill") {
                                GameVariableManager::make_entry_norewind(
                                    format!("Tile_{}_{}", x, z).as_str(),
                                    EffectType::Skill as i32
                                );
                                array[0] = func;
                                unsafe { tbox_entry(array, None); }
                                skill_count += 1;
                            }
                        }
                        else {
                            let i = rng.get_value(selection.len() as i32) as usize;
                            let index = selection[ i ];
                            let name = &SCRIPT_COMMANDS.lock().unwrap()[ index ];
                            if let Some(func) = EventScript::get_func(name.0.as_str()) {
                                println!("Effect Added: {}, at {}, {}", name.0.as_str(), x, z);
                                GameVariableManager::make_entry_norewind(
                                    format!("Tile_{}_{}", x, z).as_str(),
                                    name.1.clone() as i32
                                );
                                array[0] = func;
                                unsafe { tbox_entry(array, None); }
                            }
                            if !name.1.can_repeat() { selection.remove( i ); }
                        }
                    }
                }
            }
        }
   }
}

pub fn remove_map_effects() {
    if GameVariableManager::get_bool("Revive") {
        if let Some(dead) = Force::get(ForceType::Dead) {
            dead.transfer(3, true);
        }
        GameVariableManager::remove("Revive");
    }
    Il2CppClass::from_name("App", "UnitPool").unwrap()
        .get_static_fields_mut::<crate::randomizer::job::UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.is_some_and(|f| f.force_type == 4 || f.force_type == 0 || f.force_type == 3))
        .for_each(|unit|{
            unit.hp_stock_count = 0;
            unit.hp_stock_count_max = 0;
            unit.extra_hp_stock_count = 0;
            unit.extra_hp_stock_count_max = 0;
            EXTRA_SIDS.iter().for_each(|sid| {
                unit.private_skill.remove_sid(sid.into());
            });
            SKILL_SIDS.iter().for_each(|sid| {
                unit.private_skill.remove_sid(sid.into());
            });
        }
    );
}

pub fn visit_command_name(_item: BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString
{
    let map_mind = MapMind::get_unit();
    let ty = GameVariableManager::get_number(format!("Tile_{}_{}", map_mind.x, map_mind.z).as_str());
    if ty == 0 { return Mess::get("MID_MENU_VISIT"); }
    let ty =
    match ty {
        1 => { Mess::get("MID_SYS_Skill") }
        2 => { Mess::get("MID_SYS_HP") }
        3 => { Mess::get("MID_SYS_LV") }
        4 => { Mess::get("MID_SYS_SP") }
        5 => { Mess::get("MID_SYS_Capability") }
        6 => { Mess::get("MID_H_INFO_Param_Correction_God") }
        7 => { Mess::get("MPID_Hide") }
        8 => { Mess::get("MID_SYS_Exp") }
        9 => { Mess::get("MID_SYS_Class") }
        10 => { Mess::get("MID_SYS_GOLD_COUNT")}
        11 => { Mess::get("MID_SYS_GOLD_SKILL")}
        12 => { Mess::get("MID_CONFIG_ROD_DANCE_MYUNIT") }
        _ => { Mess::get("MID_MENU_VISIT") }
    };
    format!("{}: {}", Mess::get("MID_H_INFO_Param_Correction_Effect"), ty).into()
}

pub fn visit_command_help(_item: BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString
{
    let map_mind = MapMind::get_unit();
    let ty = GameVariableManager::get_number(format!("Tile_{}_{}", map_mind.x, map_mind.z).as_str());
    if ty == 0 { Mess::get("MID_MENU_HELP_VISIT") } else { "Tilebolical Effect".into() }
}

fn can_add_inspector(x: i32, z: i32) -> bool {
    unsafe {
        !map_inspector_is_enable(8, x, z, None) && !map_inspector_is_enable(9, x, z, None)
    }
}
#[skyline::from_offset(0x01de7480)]
fn map_inspector_is_enable(kind: i32, x: i32, z: i32, method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x01ed2e30)]
fn tbox_entry(args: &Array<&mut DynValue>, method_info: OptionalMethod);

#[skyline::from_offset(0x01dbb6c0)]
fn unit_map_effect(name: &Il2CppString, unit: &Unit, method_info: OptionalMethod); 

#[skyline::from_offset(0x01c76a90)]
fn vision_create(this: &Unit, method_info: OptionalMethod);


