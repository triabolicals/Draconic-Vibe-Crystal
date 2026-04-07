use engage::gamedata::{Gamedata, GodData, PersonData};
use engage::unit::UnitPool;
use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use engage::spriteatlasmanager::SpriteAtlasManager;
use unity::engine::Sprite;
use unity::prelude::*;
use crate::config::{DVCVariables};
use crate::enums::PIDS; // {MPIDS, PIDS, RINGS};
use crate::ironman::vtable_edit;
use crate::randomizer::names::get_emblem_person;

mod ring_select;
pub mod telop;



pub fn install_sprite_menu_methods() {
    vtable_edit(
        Il2CppClass::from_name("App", "RingSelectMenuItemContent").unwrap(), 
        "Build",
        ring_select::ring_select_menu_item_content_build as _
    );
    vtable_edit(
        Il2CppClass::from_name("App", "ArenaBondGodSelectMenuItemContent").unwrap(),
        "Build",
        ring_select::ring_select_menu_item_content_build as _
    );
    vtable_edit(
        Il2CppClass::from_name("App", "GodUnitSelectMenuItem").unwrap(),
        "Build",
        ring_select::god_select_menu_content_build as _
    );
}
/*
#[skyline::hook(offset=0x021e1250)]
pub fn get_bond_face(this: &Unit, _method_info: OptionalMethod) -> &Il2CppString {
    if let Some(name) = this.person.name.as_ref().map(|v| v.to_string()) {
        let result = call_original!(this, None);
        if let Some(old) = MPIDS.iter().position(|&x| x == name) {
            if old == 0 { return format!("Telop/LevelUp/FaceThumb/{}", get_gender_lueur_ascii(false)).into(); }
            let new_name = &MPIDS[old][5..];
            format!("Telop/LevelUp/FaceThumb/{}", new_name).into()
        }
        else if let Some(pos) = RINGS.iter().find(|&x| this.person.name.is_some_and(|v| v.to_string().contains(x))) {
            format!("Telop/LevelUp/FaceThumb/{}", pos).into()
        }
        else if ResourceManager::file_exist(result) { result } else {
            let rng = create_rng(this.person.parent.hash, 1);
            let len = SEARCH_LIST.get().unwrap().bond_face.len();
            SEARCH_LIST.get().unwrap().bond_face.get(rng.get_value(len as i32) as usize).unwrap().into()
        }
    }
    else { call_original!(this, None) }
}
*/
pub fn get_gender_lueur_ascii(god: bool, _female: bool) -> String {
    let is_female =
        if GameVariableManager::exist(DVCVariables::LUEUR_GENDER) {  GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2  }
        else if let Some(lueur_unit) = UnitPool::get_from_pid(PIDS[0].into(), false) {
            if lueur_unit.edit.is_enabled() { lueur_unit.edit.gender == 2  } else { false }
        }
        else { false };

    match (god, is_female) {
        (true, true) => { "LueurW_God"}
        (false, true) =>  { "LueurW"}
        (true, false) => { "Lueur_God"}
        (false, false) => {"Lueur"}
    }.to_string()
}
/*
#[skyline::hook(offset=0x021e16f0)]
pub fn get_god_face(this: &GodData, method_info: OptionalMethod) -> &Il2CppString {
    let mid = this.mid;
    let mut is_rng = false;
    if let Some(person) = get_emblem_person(mid).and_then(|x| x.get_ascii_name()) {
        let path = format!("Telop/LevelUp/FaceThumb/{}", person).into();
        if ResourceManager::file_exist(path) { return path; }
        is_rng = true;
    }
    let result = call_original!(this, method_info);
    if mid.str_contains("Lueur") && this.gid.str_contains("リュール") {
        return format!("Telop/LevelUp/FaceThumb/God{}", get_gender_lueur_ascii(false)).into();
    }
    if !is_rng && ResourceManager::file_exist(result) { result }
    else {
        let rng = create_rng(this.parent.hash, 1);
        let len = SEARCH_LIST.get().unwrap().bond_face.len();
        SEARCH_LIST.get().unwrap().bond_face.get(rng.get_value(len as i32) as usize).unwrap().into()
    }
}
*/

#[unity::hook("App", "SpriteAtlasManager", "TryGet")]
pub fn try_get_sprite(this: &SpriteAtlasManager, name: &Il2CppString, method_info: OptionalMethod) -> Option<&'static Sprite> {
    if name.is_null() || GameUserData::get_sequence() == 0 { return call_original!(this, name, method_info); }
    let path = this.handle.path.to_string();
    if path.contains("UI/Common/UnitList/FaceThumb/FaceThumb") {
        let mut ascii_name = name.to_string();
        let name_original = name.to_string();
        if name_original.contains("Lueur") {
            let is_god = name_original.contains("_God");
            ascii_name = get_gender_lueur_ascii(is_god, DVCVariables::is_lueur_female());
        }
        else if name_original.contains("PhantomW") { ascii_name = "Phantom".to_string(); }
        // Switch the thumbs of Guest Nel (El) and Nil (Il)
        if GameUserData::is_evil_map() && DVCVariables::UnitRecruitment.get_value()  != 0 {
            let person_index = if ascii_name.contains("El") { 36 }
            else if ascii_name.contains("Il") { 37 }
            else { 0 };

            if person_index != 0 {
                if let Some(person) = PersonData::get(DVCVariables::get_dvc_person(person_index, false)){
                    if person.pid.to_string() != PIDS[person_index as usize] {
                        ascii_name = person.get_ascii_name().unwrap().to_string();
                    }
                }
            }
        }
        return call_original!(this, ascii_name.into(), method_info).or_else(|| call_original!(this, "Phantom".into(), method_info));
    }
    if path.contains("Unit/UnitIndexes") && GameUserData::get_sequence() != 0 {
        let parts = name.to_string().split("_").map(|str| str.to_string()).collect::<Vec<String>>();
        if parts.len() >= 2 && parts[0].len() > 3 {
            if parts[0].starts_with("70") || parts[0].starts_with("71") {   // Generic
                if let Some(unit_icon) = get_unit_icon_from_unique(parts[1].as_str()) {
                    return call_original!(this, format!("{}_{}_NoWeapon", unit_icon, parts[1]).into(), None);
                }
            }
            if call_original!(this, name, None).is_none() {
                if let Some(unit_icon) = is_player_with_default_weapon(parts[0].as_str()) {
                    return call_original!(this, unit_icon.into(), None);
                }
            }
            if parts[0].ends_with("E") {
                if call_original!(this, name, None).is_none() {
                    let no_e = parts[0].trim_end_matches("E");
                    let s = call_original!(this, format!("{}_{}", no_e, no_e).into(), None);
                    if s.is_some() { return s; }
                }
            }
        }
    }
    call_original!(this, name, None)
}

#[skyline::hook(offset=0x01f827f0)]
pub fn unit_icon_set_god_icon(this: u64, god_data: Option<&GodData>, is_female: bool, is_dark: bool, method_info: OptionalMethod) {
    if let Some(person) = god_data.and_then(|d| get_emblem_person(d.mid))
        .filter(|x| x.unit_icon_id.is_some() && x.get_job().is_some_and(|x| !x.unit_icon_weapon_id.is_null()))
    {
        let unit_icon = person.unit_icon_id.unwrap();
        let job = person.get_job().unwrap();
        let icon_key =
            if is_female && job.unit_icon_id_f.is_some() {
                format!("{}_{}_{}", unit_icon, job.unit_icon_id_f.unwrap(), job.unit_icon_weapon_id)
            }
            else if !is_female && job.unit_icon_id_m.is_some() {
                format!("{}_{}_{}", unit_icon, job.unit_icon_id_m.unwrap(), job.unit_icon_weapon_id)
            }
            else { format!("{}_{}", unit_icon, unit_icon) };
        unsafe { unit_icon_try_set(this, Some(icon_key.into()), person.unit_icon_id, None); }
    }
    else { call_original!(this, god_data, is_female, is_dark, method_info); }
}

#[skyline::from_offset(0x01f82440)]
fn unit_icon_try_set(this: u64, index: Option<&Il2CppString>, palette_name: Option<&Il2CppString>, method_info: OptionalMethod);

fn is_player_with_default_weapon(person_icon: &str) -> Option<String> {
    match person_icon {
        "102Louis" => Some("102Louis_630LanceArmor_Lance".to_string()),
        "153Chloe" => Some("153Chloe_646LancePegasus_Lance".to_string()),
        "203Umber" => Some("203Umber_637LanceKnight_Lance".to_string()),
        "250Jade" => Some("250Jade_631AxArmor_Ax".to_string()),
        _ => None
    }
}
fn get_unit_icon_from_unique(job_icon: &str) -> Option<String>  {
    match job_icon {
        "600DragonLord"|"602DragonKing" => Some("001Lueur".to_string()),
        "601DragonLord"|"603DragonKing" => Some("051Lueur".to_string()),
        "718ShadowLord" => Some("002Lueur".to_string()),
        "719ShadowLord" => Some("052Lueur".to_string()),
        "681ShadowPrincess" => Some("551Veyre".to_string()),
        "694ShadowKing" => Some("504Sombre".to_string()),
        "678AvenirLC"|"679Avenir" => Some("100Alfred".to_string()),
        "675FleurageLC"|"676Fleurage" => Some("150Celine".to_string()),
        "683SuccesseurLC"|"684Successeur" => Some("200Diamand".to_string()),
        "685TirailleurLC"|"686TirailleurLC" => Some("201Staluke".to_string()),
        "687LindwurmLC"|"656Lindwurm" => Some("350Ivy".to_string()),
        "688SleipnirLC"|"651Sleipnir" => Some("351Hortensia".to_string()),
        "692PitchforkLC"|"693Pitchfork" => Some("450Misutira".to_string()),
        "690CupidoLC"|"691Cupido" => Some("400Fogato".to_string()),
        "716Melusine" => Some("553Selestia".to_string()),
        "673Dancer" => Some("403Seadas".to_string()),
        "748ShadowPrincessR" => Some("099El".to_string()),
        "749ShadowLordR" => Some("049Il".to_string()),
        _ => None,
    }
}