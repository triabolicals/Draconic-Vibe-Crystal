use engage::{
    force::*, gamedata::terrain::TerrainData,
    gamemessage::GameMessage, gamesound::GameSound,
    map::{
        inspectors::{MapInspectorKind, MapInspectors},
        mind::MapMind, overlap::MapOverlap, terrain::MapTerrain
    },
    script::*, sequence::unitgrowsequence::UnitGrowSequence,
    unit::{Unit, UnitPool},
};
use std::sync::Mutex;
use crate::randomizer::status::RandomizerStatus;
use super::*;
pub mod effects;
pub(crate) mod dispos;
pub mod shuffle;

#[repr(i32)]
#[derive(PartialEq, Clone)]
pub enum EffectType {
    NoEffect = 0,
    Skill = 1,
    HP = 2,
    Level = 3,
    SP = 4,
    Stat = 5,
    Emblem = 6,
    Other = 7,
    Exp = 8,
    Class = 9,
    GoldItem = 10,
    SpawnUnit = 11,
}
impl EffectType {
    pub fn can_repeat(&self) -> bool {
        match self {
            EffectType::SpawnUnit|EffectType::Stat|EffectType::Class|EffectType::GoldItem => { true }
            _ => { false }
        }
    }
    pub fn to_u8(&self) -> u8 {
        match self {
            EffectType::NoEffect => 0,
            EffectType::Skill => 1,
            EffectType::HP => 2,
            EffectType::Level => 3,
            EffectType::SP => 4,
            EffectType::Stat => 5,
            EffectType::Emblem => 6,
            EffectType::Other => 7,
            EffectType::Exp => 8,
            EffectType::Class => 9,
            EffectType::GoldItem => 10,
            EffectType::SpawnUnit => 11,
            
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
pub extern "C" fn register_script_commands(script: &EventScript) { effects::install_tilebolical_effects(script);  }
fn register_action(script: &EventScript, name: &str, action: extern "C" fn(&Il2CppArray<&DynValue>, OptionalMethod), ty: EffectType) {
    EventScript::register_action(script, name, action);
    let lock = &mut SCRIPT_COMMANDS.lock().unwrap();
    if lock.iter().find(|&x| x.0 == name).is_none() { lock.push((name.to_string(), ty)); }
}

pub fn tilabolical() {
    if !DVCFlags::Tile.get_value() { return; }
    if let Some(mut ran) = RANDOMIZED_DATA.get().and_then(|v| v.write().ok()) { ran.person_appearance.randomize(true); }
    let status = RandomizerStatus::get();
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 7 {
        if !GameVariableManager::exist("TileSkills") { GameVariableManager::make_entry("TileSkills", 0); }
        if let Some(terrain) = MapTerrain::get_instance() {
            status.map_tile = true;
            let start_x = terrain.x;
            let end_x = terrain.width;
            let start_z = terrain.z;
            let end_z = terrain.height;
            let pillars = TerrainData::get("TID_ブロック").unwrap();
            let rng = DVCVariables::init_tile_rng(false);
            let array = Array::<&DynValue>::new_from_element_class(DynValue::class(), 3).unwrap();
            for x in 0..3 { array[x] = DynValue::new_number(0.0); }
            if let Ok(script_commands) = SCRIPT_COMMANDS.try_lock() {
                let n_commands = script_commands.len();
                let mut selection: Vec<_> = (0..n_commands).collect();
                let skill_len = SKILL_SIDS.len();
                let mut skill_count = 0;
                let mut count = 0;
                for z in start_z..end_z {
                    for x in start_x..end_x {
                        if rng.get_value(100) < 5 && can_add_inspector(x, z) && MapOverlap::can_create(None, x, z, pillars) && count < 32 {
                            let index = (z as usize * 32) + x as usize;
                            array[1].assign_number(x as f64);
                            array[2].assign_number(z as f64);
                            let slen = 3*selection.len() + skill_len;
                            let i = rng.get_value(slen as i32) as usize;
                            if i < skill_len && skill_count < skill_len {
                                if let Some(func) = EventScript::get_func("RandomSkill"){
                                    status.tilabolical[index] = EffectType::Skill as u8;
                                    array[0] = func;
                                    ScriptMap::event_entry_visit(array);
                                    skill_count += 1;
                                }
                            }
                            else {
                                if let Some(s_index) = selection.get_remove(rng) {
                                    let name = &script_commands[s_index];
                                    if let Some(func) = EventScript::get_func(name.0.as_str()) {
                                        let ty = name.1.to_u8();
                                        status.tilabolical[index] = ty;
                                        array[0] = func;
                                        ScriptMap::event_entry_visit(array);
                                    }
                                }
                            }
                            count += 1;
                        }
                    }
                }
            }
        }
    }
}

pub fn remove_map_effects() {
    Il2CppClass::from_name("App", "UnitPool").unwrap()
        .get_static_fields_mut::<job::UnitPoolStaticFieldsMut>().s_unit
        .iter_mut()
        .filter(|unit| unit.force.is_some_and(|f| ( 1 << f.force_type) & 25 != 0 ))
        .for_each(|unit|{
            unit.hp_stock_count = 0;
            unit.hp_stock_count_max = 0;
            unit.extra_hp_stock_count = 0;
            unit.extra_hp_stock_count_max = 0;
            EXTRA_SIDS.iter().for_each(|sid| { unit.private_skill.remove_sid(sid.into()); });
            SKILL_SIDS.iter().for_each(|sid| { unit.private_skill.remove_sid(sid.into()); });
        }
    );
    if GameVariableManager::get_bool("G_Revive"){
        if let Some(dead) = Force::get(ForceType::Dead) { dead.iter().for_each(|u|{ u.transfer(ForceType::Absent, true); }); }
        GameVariableManager::set_bool("G_Revive", false);
    }
}

pub fn visit_command_name(_item: engage::menu::BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    if let Some(unit) = MapMind::get_unit() {
        let ty = RandomizerStatus::get_tile(unit.x, unit.z);
        let mess =
        match ty {
            1 => { "MID_SYS_Skill" }
            2 => { "MID_SYS_HP" }
            3 => { "MID_SYS_LV" }
            4 => { "MID_SYS_SP" }
            5 => { "MID_SYS_Capability" }
            6 => { "MID_H_INFO_Param_Correction_God" }
            7 => { "MPID_Hide" }
            8 => { "MID_SYS_Exp" }
            9 => { "MID_SYS_Class" }
            10 => { "MID_SYS_GOLD_COUNT"}
            11 => { "MID_SYS_GOLD_SKILL"}
            12 => { "MID_MENU_UNIT_LIST"}
            _ => { return Mess::get("MID_MENU_VISIT"); }
        };
        format!("{}: {}", Mess::get("MID_H_INFO_Param_Correction_Effect"), Mess::get(mess)).into()
    }
    else { Mess::get("MID_MENU_VISIT") }
}
pub fn visit_command_help(_item: engage::menu::BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    let map_mind = MapMind::get_unit().unwrap();
    let ty = RandomizerStatus::get_tile(map_mind.x, map_mind.z);
    if ty == 0 { Mess::get("MID_MENU_HELP_VISIT") } else { "Tilebolical Effect".into() }
}
fn can_add_inspector(x: i32, z: i32) -> bool {
    !MapInspectors::is_enable_at_position(MapInspectorKind::Visit, x, z) && !MapInspectors::is_enable_at_position(MapInspectorKind::Escape, x, z)
}