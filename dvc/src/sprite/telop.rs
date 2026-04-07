use engage::gamedata::assettable::AssetTableResult;
use engage::gamedata::GodData;
use engage::god::GodUnit;
use engage::proc::ProcInstFields;
use engage::resourcemanager::ResourceManager;
use engage::unit::Unit;
use unity::engine::Sprite;
use unity::prelude::*;
use crate::DVCVariables;
use crate::enums::{MPIDS, RINGS};
use crate::randomizer::names::get_emblem_person;
use crate::sprite::get_gender_lueur_ascii;

#[unity::class("App", "TelopManager")]
pub struct TelopManager {}
impl TelopManager {
    #[unity::class_method(16)] pub fn get_bond_level_face_path(unit: &Unit) -> &'static Il2CppString; // Offset: 0x21E1250 Flags: 0
    #[unity::class_method(17)] pub fn get_bond_level_face_path2(god: &GodUnit) -> &'static Il2CppString; // Offset: 0x21E1450 Flags: 0
    #[unity::class_method(18)] pub fn get_bond_level_face_path3(data: &GodData) -> &'static Il2CppString; // Offset: 0x21E16F0 Flags: 0
}

#[unity::class("", "ProcBondLevelUp")]
pub struct ProcBondLevelUp {
    pub proc: ProcInstFields,
    pub unit: Option<&'static Unit>,
    pub god: Option<&'static GodUnit>,
    pub level: i32,
    pub next_level: i32,
    pub god_data: Option<&'static GodData>,
}

#[unity::class("", "ProcBondEngagePair")]
pub struct ProcBondEngagePair {
    pub proc: ProcInstFields,
    root: u64,
    result: Option<&'static AssetTableResult>,
    pub sequence: i32,
    pub wait_frame: i32,
    pub main_god: Option<&'static GodData>,
    pub sub_god: Option<&'static GodData>,
}


pub fn proc_bond_level_up_load_face(this: &ProcBondLevelUp, _optional_method: OptionalMethod) {
    if let Some(unit) = this.unit.as_ref() { ResourceManager::load_global_async::<Sprite>(get_bond_face(unit), None); }
    if let Some(god) = this.god_data.as_ref() { ResourceManager::load_global_async::<Sprite>(get_god_face(god), None); }
}

pub fn proc_bond_level_up_release_face(this: &ProcBondLevelUp, _optional_method: OptionalMethod) {
    if let Some(unit) = this.unit.as_ref() { ResourceManager::release_global(get_bond_face(unit)); }
    if let Some(god) = this.god_data.as_ref() { ResourceManager::release_global(get_god_face(god)); }
}

pub fn proc_bond_engage_pair_load_face(this: &ProcBondEngagePair, _optional_method: OptionalMethod) {
    if let Some(god) = this.main_god.as_ref() { ResourceManager::load_global_async::<Sprite>(get_god_face(god), None); }
    if let Some(god) = this.sub_god.as_ref() { ResourceManager::load_global_async::<Sprite>(get_god_face(god), None); }
}

pub fn proc_bond_engage_pair_release_face(this: &ProcBondEngagePair, _optional_method: OptionalMethod) {
    if let Some(god) = this.main_god.as_ref() { ResourceManager::release_global(get_god_face(god)); }
    if let Some(god) = this.sub_god.as_ref() { ResourceManager::release_global(get_god_face(god)); }
}

pub fn get_bond_face(this: &Unit) -> &'static Il2CppString {
    if let Some(name) = this.person.name.as_ref().map(|v| v.to_string()) {
        let result = TelopManager::get_bond_level_face_path(this);
        if let Some(old) = MPIDS.iter().position(|&x| x == name) {
            if old == 0 { return format!("Telop/LevelUp/FaceThumb/{}", get_gender_lueur_ascii(false, this.edit.gender == 2)).into(); }
            let new_name = &MPIDS[old][5..];
            format!("Telop/LevelUp/FaceThumb/{}", new_name).into()
        }
        else if let Some(pos) = RINGS.iter().find(|&x| this.person.name.is_some_and(|v| v.to_string().contains(x))) {
            format!("Telop/LevelUp/FaceThumb/{}", pos).into()
        }
        else if ResourceManager::file_exist(result) { result }
        else { TelopManager::get_bond_level_face_path(this) }
    }
    else { TelopManager::get_bond_level_face_path(this) }
}
pub fn get_god_face(this: &GodData) -> &Il2CppString {
    let mid = this.mid;
    if let Some(person) = get_emblem_person(mid).and_then(|x| x.get_ascii_name()) {
        let path = format!("Telop/LevelUp/FaceThumb/{}", person).into();
        if ResourceManager::file_exist(path) { return path; }
    }
    let result = TelopManager::get_bond_level_face_path3(this);
    if mid.str_contains("Lueur") && this.gid.str_contains("リュール") {
        if DVCVariables::get_dvc_recruitment_index(0) == 0 {
            format!("Telop/LevelUp/FaceThumb/God{}", get_gender_lueur_ascii(false, DVCVariables::is_lueur_female())).into()
        }
        else if let Some(unit) = DVCVariables::get_dvc_unit(0, false){
            TelopManager::get_bond_level_face_path(unit)
        }
        else { result }
    }
    else { result }
}