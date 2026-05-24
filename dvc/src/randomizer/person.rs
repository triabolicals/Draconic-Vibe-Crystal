use skyline::patching::Patch;
pub use engage::{
    unit::{Unit, UnitPool, Gender, UnitUtil},
    force::ForceType, spriteatlasmanager::SpriteAtlasManager,
    resourcemanager::ResourceManager, mess::*, 
    gamevariable::*, gameuserdata::*, hub::access::*, random::*, 
    gamedata::{
        hub::{HubDisposData, HubFacilityData}, *, 
        item::*, skill::{SkillDataCategorys, SkillData}, dispos::*
    },
};
use crate::{
    config::DVCFlags, enums::*, utils::*, autolevel::*, DVCVariables,
    randomizer::job::reclass::ReclassType
};
use super::{data::GameData, menu::CUSTOM_RECRUITMENT_ORDER, Randomizer};

pub mod ai;
pub mod unit; 
pub mod hub;
const PROTAG_SKILLS: [&str; 4] = ["SID_主人公", "SID_王族", "SID_リベラシオン装備可能", "SID_ヴィレグランツ装備可能"];

pub fn is_playable_person(person: &PersonData) -> bool { GameData::get().playables.iter().any(|p| p.hash == person.parent.hash) }
fn set_hub_facilities() {
    let aid = [
        ("AID_蚤の市", "LocatorSell01", 23), ("AID_筋肉体操", "LocatorTraining01", 4),
        ("AID_ドラゴンライド", "LocatorDragon01", 17), ("AID_釣り", "LocatorFish01", 14), ("AID_占い小屋", "LocatorFortune01", 27)
    ];
    if let Some(somniel) = HubDisposData::get_list_mut().and_then(|v| v.get_mut(1)) {
        aid.iter().for_each(|(aid, locator, recruitment_idx)| {
            let new_recruit_idx = DVCVariables::get_dvc_recruitment_index(*recruitment_idx);
            if new_recruit_idx != -1 {
                if let Some(dispos) = somniel.iter_mut().find(|v| v.locator.is_some_and(|l| l.to_string() == *locator)) {
                    dispos.set_chapter(RECRUIT_CID[new_recruit_idx as usize].into());
                }
                if let Some(facility) = HubFacilityData::get_mut(aid) {
                    facility.condition_cid = format!("CID_{}", RECRUIT_CID[new_recruit_idx as usize]).into() ;
                }
            }
        });
    }
}
pub fn randomize_person() {
    if !can_rand() { DVCVariables::create_recruitment_variables(false); }
    else if DVCVariables::is_recruitment_set(false) {
        if DVCVariables::UnitRecruitment.get_value() != 0 {
            set_hub_facilities();
            hub::change_somniel_hub_dispos();
        }
    }
    else {
        let rng = get_rng();
        DVCVariables::create_recruitment_variables(false);
        match DVCVariables::UnitRecruitment.get_value(){
            1 => {
                let with_gender = DVCFlags::RRGenderUnitMatch.get_value();
                let no_dlc = !dlc_check() | DVCFlags::ExcludeDLCUnitRR.get_value();
                let with_custom_units = DVCFlags::CustomUnits.get_value();
                let list = &GameData::get().playables;
                let mut playable_list: Vec<_> =
                    list.iter()
                        .enumerate()
                        .map(|(i, person)|{
                            let is_female = UnitPool::get_from_person_force_mask(person.get_person_data(), -1)
                                .filter(|unit| unit.edit.gender > 0).map(|lueur| lueur.edit.gender == 2)
                                .unwrap_or(person.get_person_data().gender == 2);
                            (i, is_female)
                        }).filter(|&(i, u)| i < 36 || (i >= 36 && i < 41 && !no_dlc) || (i >= 41 && with_custom_units))
                        .collect();
                let mut to_replace_list: Vec<_> = playable_list.clone();
                let pids: Vec<String> = list.iter().map(|x| PersonData::try_get_hash(x.hash).unwrap().pid.to_string()).collect();
                let royals = [0, 23, 4, 17, 14, 27];
                for x_royal in royals {
                    if let Some((pos, royal)) =  playable_list.iter().enumerate().find(|(i, x)| x.0 == x_royal) {
                        if let Some(royal_replacement) = to_replace_list.get_remove_filter(rng, |(i, gender)| *i < 36 && *i != 30 && ((with_gender && *gender == royal.1) || !with_gender)){
                            println!("{} -> {}", royal_replacement.0, royal.0);
                            DVCVariables::set_person_recruitment(royal_replacement.0 as i32, royal.0 as i32);
                            playable_list.remove(pos);
                        }
                    }
                }
                to_replace_list.iter().for_each(|(index_x, gender_x)|{
                    if let Some((index_y, _)) = playable_list.get_remove_filter(rng, |(i, gender_y)| (with_gender && gender_x == gender_y) || !with_gender)
                        .or_else(|| playable_list.get_remove(rng))
                    {
                        DVCVariables::set_variable_key_string(format!("G_R_{}", pids[*index_x].as_str()), pids[index_y].as_str());
                        DVCVariables::set_variable_key_string(format!("G_R2_{}", pids[index_y].as_str()), pids[*index_x].as_str());
                    }
                });
            },
            2 => {   //Reverse
                for x in 0..41 { DVCVariables::set_person_recruitment(x, RR_ORDER[x as usize] as i32); }
            },
            3 => { // Custom
                let order =crate::DVCConfig::get().get_custom_recruitment(false);
                order.iter().for_each(|(p1, p2)|{ DVCVariables::set_person_recruitment(*p1, *p2); });
            },
            4 => {
                let list = unsafe { &CUSTOM_RECRUITMENT_ORDER };
                let mut pool: Vec<_> = (0..list[41]).collect();
                let mut order = vec![];
                list.iter().enumerate().filter(|v| *v.1 < list[41])
                    .for_each(|(i, v)| {
                        if let Some(pos) = pool.iter().position(|vv| *vv == *v) {
                            pool.swap_remove(pos);
                            order.push((i as i32, *v as i32));
                        }
                    });
                list.iter().enumerate().filter(|v| *v.1 >= list[41])
                    .for_each(|(i, v)| {
                        if let Some(rand_index) = pool.get_remove(rng) {
                            order.push((i as i32, rand_index as i32));
                        }
                    });
                (41..list[41]).into_iter().for_each(|v|{
                    if let Some(rand_index) = pool.get_remove(rng) { order.push((v as i32, rand_index as i32)); }
                });
                let playables = &GameData::get().playables;
                order.iter().flat_map(|indexes| 
                    playables.get(indexes.0 as usize).and_then(|p| PersonData::try_get_hash(p.hash))
                        .zip(playables.get(indexes.1 as usize).and_then(|p| PersonData::try_get_hash(p.hash)))
                ).for_each(|(original, randomized)|{
                    let key = format!("G_R_{}", original.pid);
                    let reversed = format!("G_R2_{}", randomized.pid);
                    DVCVariables::set_variable_key_string(key.as_str(), randomized.pid);
                    DVCVariables::set_variable_key_string(reversed.as_str(), original.pid);
                });
            }
            _ => { DVCVariables::create_recruitment_variables(false); },
        }
        set_hub_facilities();
        hub::change_somniel_hub_dispos();
    }
}
pub fn find_pid_replacement(pid: &String, reverse: bool) -> Option<String>{
    if PIDS.iter().position(|&x| x == *pid).is_some() || EMBLEM_GIDS.iter().position(|&x| x == *pid).is_some() {
        if reverse { Some(GameVariableManager::get_string(&format!("G_R2_{}", pid)).to_string()) }
        else { Some(GameVariableManager::get_string(&format!("G_R_{}", pid)).to_string()) }
    }
    else { None }
}

pub fn change_lueur_for_recruitment(is_start: bool) {
    if !can_rand()  { return; }
    if DVCVariables::get_dvc_person(0, false).to_string() == PIDS[0] {
        if is_start && can_rand() {
            if let Some(lueur) = UnitPool::get_from_pid(PIDS[0].into(), false) {
                if can_rand() && DVCVariables::ClassMode.get_value() > 0 {
                    crate::randomizer::job::reclass::unit_reclass(lueur,  ReclassType::get_from_settings(true));
                    unit::adjust_unit_items(lueur);
                }
            }
        }
        return;
    }
    let person_lueur = PersonData::get(PIDS[0]).unwrap();
    person_lueur.set_link_god(None);
    PROTAG_SKILLS.iter().for_each(|x| { person_lueur.get_mask_skill().remove_sid(x.into()); });
    if let Some(new_hero) = DVCVariables::get_dvc_person_data(0, false){
        PROTAG_SKILLS.iter().for_each(|x|{ new_hero.get_mask_skill().add_sid(x, SkillDataCategorys::Private, 0); });
        if let Some(god) = GodData::get_mut("GID_リュール") {
            god.link = Some(new_hero.pid);
            new_hero.set_link_god(Some(god));
        }
        if is_start {   // Move alear to force 5
            if let Some(lueur_unit) = UnitPool::get_from_pid(PIDS[0].into(), false) {
                unit::change_unit_autolevel(lueur_unit, true);
                crate::randomizer::job::reclass::unit_reclass(lueur_unit, ReclassType::get_from_settings(true));
                unit::fixed_unit_weapon_mask(lueur_unit);
                unit::adjust_unit_items(lueur_unit);
                lueur_unit.transfer(ForceType::Lost, false);
                get_lueur_name_gender(); // grab gender and name
                DVCVariables::LueurGender.init_var(lueur_unit.edit.gender, true);
            }
            if let Some(unit) = UnitUtil::join_unit_person(new_hero) {
                unit.edit.set_name(new_hero.get_name());
                unit.edit.set_gender(new_hero.get_gender());
                PROTAG_SKILLS.iter().for_each(|x|{ unit.private_skill.add_sid(x, SkillDataCategorys::Private, 0); });
                unit.transfer(ForceType::Absent, false);
            }
        }

        Patch::in_text(0x02d524e0).nop().unwrap();
        Patch::in_text(0x02d524e4).nop().unwrap();

        // LueurW_God or Lueur_God in GetPath
        if DVCVariables::LueurGender.get_value() == 2 {
            Patch::in_text(0x02d524e8).bytes(&[0x48, 0x00, 0x80, 0x52]).unwrap();
            person_lueur.set_gender(Gender::Female);
        } else {
            Patch::in_text(0x02d524e8).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap();
            person_lueur.set_gender(Gender::Male);
        }
        let lueur_god_offsets = [0x02d51dec, 0x021e12ac]; //, 0x02915844, 0x02915694, 0x01c666ac, 0x02081edc, 0x01c69d60, 0x01c66588];
        for x in lueur_god_offsets { mov_x0_0(x); }
        // For Hub-Related Activities
        let offsets = [0x02ae8d28, 0x02ae9000, 0x02a5d0f4, 0x01cfd4c4, 0x01d03184, 0x01e5fe00, 0x01e5ff4c, 0x027049c8];
        let new_hero_gender = if new_hero.get_gender() == 2 || (new_hero.get_gender() == 1 && new_hero.flag.value & 32 != 0) { 2 } else { 1 };
        for x in offsets {
            if new_hero_gender == 1 {  mov_x0_0(x); }
            else { mov_1(x); }
        }
    }
}

#[skyline::hook(offset=0x1c54fa0)]
pub fn unit_pool_get_from_person(pid: Option<&Il2CppString>, relay: bool, optional_method: OptionalMethod) -> Option<&'static mut Unit> {
    if pid.is_none() {
        let pid = DVCVariables::get_dvc_person(0, false);
        let unit = call_original!(Some(pid), relay, optional_method);
        unit
    }
    else { call_original!(pid, relay, optional_method) }
}

pub fn pid_to_index(pid: &String, reverse: bool) -> i32 {
    if let Some(replacement) = find_pid_replacement(pid, reverse) {
        if let Some(found_pid) = PIDS.iter().position(|&x| x == replacement) { return found_pid as i32; }
        if let Some(found_gid) = EMBLEM_GIDS.iter().position(|&x| x == replacement).filter(|x| *x < 19) { return found_gid as i32;  }
    }
    -1  // to cause crashes
}

pub fn switch_person(person: &PersonData) -> Option<&'static PersonData> {
    if DVCVariables::UnitRecruitment.get_value()  == 0 { None }
    else { GameVariableManager::try_get_string(format!("G_R_{}", person.pid)).and_then(|pid| PersonData::get(pid)) }
}
pub fn switch_person_reverse(person: &PersonData) -> Option<&'static PersonData> {
    GameVariableManager::try_get_string(format!("G_R2_{}", person.pid)).and_then(|pid| PersonData::get(pid))
}

// Handle the case of Chapter 11 ends with not escape
pub fn m011_ivy_recruitment_check(){
    if !DVCVariables::random_enabled() || DVCVariables::UnitRecruitment.get_value()  == 0 { return; }
    if DVCVariables::get_chapter_index() == 11  && lueur_on_map() {
        GameVariableManager::make_entry("MapRecruit", 1);
        GameVariableManager::set_bool("MapRecruit", true);
    }
}
pub fn lueur_recruitment_check() {
    if let Some(lueur) = UnitPool::get_from_person_force_mask(PersonData::get(PIDS[0]).unwrap(), 6){
        if lueur.force.is_some_and(|f| ( GameUserData::get_chapter().cid.str_contains("M018") && f.force_type == 1 ) || f.force_type == 2) {
            if GameUserData::get_sequence() == 3 { lueur.transfer(ForceType::Player, true); }
            else if GameUserData::get_sequence() == 5 { lueur.transfer(ForceType::Absent, true); }
        }
    }
}