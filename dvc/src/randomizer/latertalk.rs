use unity::{prelude::*, system::List};
use engage::{
    unit::Unit, gamevariable::GameVariableManager, random::Random, proc::ProcInstFields,
    gamedata::{Gamedata, GamedataArray, PersonData, StructBaseFields},
};
use crate::{DVCVariables, assets::data::SEARCH_LIST, randomizer::{Randomizer, data::GameData}};

pub struct LaterTalkSetterUnitData {
    pub unit: &'static Unit,
    pub is_alive: bool,
    pub marriage: Option<&'static Unit>,
    pub map_name: &'static Il2CppString,
}
#[unity::class("App", "LaterTalkSetter")]
pub struct LaterTalkSetter {
    stuff: [u8; 0x50],
    pub unit_data: &'static mut List<LaterTalkSetterUnitData>,
}

#[unity::class("App", "LaterTalkSequence")]
pub struct LaterTalkSequence {
    pub proc: ProcInstFields,
}

#[unity::class("App", "LaterTalkData")]
pub struct LaterTalkData {
    pub parent: StructBaseFields,
    pub array_name: &'static Il2CppString,
    pub person: &'static Il2CppString,
    pub field: &'static Il2CppString,
    pub back_degree: i32,
    pub light_degree: i32,
}
impl GamedataArray for LaterTalkData {}

pub fn edit_later_talk_data() {
    if let Some(data) = LaterTalkData::get_list_mut() {
        let map_list = &SEARCH_LIST.get().unwrap().map_events;
        let rng = Random::get_system();
        if DVCVariables::UnitRecruitment.get_value() != 0 {
            data.iter_mut().flat_map(|v| v.iter_mut()).for_each(|d|{
                let key = format!("G_R_{}", d.person);
                if GameVariableManager::exist(&key) {
                    d.person = GameVariableManager::get_string(&key);
                    if let Some(field) = map_list.get_random_element(rng) { d.field = field.into(); }
                }
            });
        }
        let playables = &GameData::get().playables;
        if playables.len() > 41 {
            playables.iter().filter(|k| k.playable_slot > 40 )
                .flat_map(|k|PersonData::try_get_hash(k.hash))
                .for_each(|p|
            {
                let key = format!("G_R_{}", p.pid);
                let pid = if GameVariableManager::exist(&key) { GameVariableManager::get_string(&key) }
                else { p.pid };
                if let Ok(entry) = LaterTalkData::instantiate() {
                    entry.person = pid;
                    entry.light_degree = 100;
                    entry.back_degree = 0;
                    if let Some(field) = map_list.get_random_element(rng) { entry.field = field.into(); }
                    if let Some(last) = data.iter_mut().last() {
                        last.add(entry);
                    }
                }
            });
        }
    }
}