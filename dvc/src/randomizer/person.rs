use skyline::patching::Patch;
pub use engage::{
    unit::{Unit, UnitPool},
    resourcemanager::ResourceManager,
    mess::*,
    gamevariable::*, gameuserdata::*, hub::access::*, random::*,
    gamedata::{*, item::*, skill::SkillData, dispos::*},
    spriteatlasmanager::SpriteAtlasManager,
};
use engage::force::ForceType;
use engage::gamedata::hub::{HubDisposData, HubFacilityData};
use engage::gamedata::skill::SkillDataCategorys;
use engage::unit::{Gender, UnitUtil};
use unity::il2cpp::object::Array;
use crate::{enums::*, utils::*, autolevel::*};
use crate::config::DVCFlags;
use crate::randomizer::data::GameData;
use crate::randomizer::menu::CUSTOM_RECRUITMENT_ORDER;
use super::{get_data_read, DVCVariables, Randomizer, RANDOMIZER_STATUS};

pub mod ai;
pub mod unit; 
pub mod hub;

pub fn is_playable_person(person: &PersonData) -> bool { get_data_read().playables.iter().any(|p| p.hash == person.parent.hash) }
pub fn check_playable_classes() {
    let list = &get_data_read().playables;
    list.iter().for_each(|index|{
        if let Some(person) = PersonData::try_get_hash_mut(index.hash) {
            if person.get_job().is_none() {
                if person.get_sp() >= 1000 || person.get_internal_level() > 0 { person.jid = Some("JID_ソードマスター".into()); }
                else {person.jid = Some("JID_ソードファイター".into()); }
                person.on_completed();
            }
        }
    });
}
fn set_hub_facilities() {
    let aid = ["AID_蚤の市", "AID_筋肉体操", "AID_ドラゴンライド", "AID_釣り", "AID_占い小屋"];
    let locator = ["LocatorSell01", "LocatorTraining01", "LocatorDragon01", "LocatorFish01", "LocatorFortune01"];
    let index = [ 23, 4, 17, 14, 27];
    if let Some(hub_dispos) = HubDisposData::get_list_mut() {
        for x in 0..aid.len() {
            let data = HubFacilityData::get_mut(aid[x]);
            let pid = PIDS[index[x] as usize];
            let a_index = pid_to_index(&pid.to_string(), true) as usize;
            if data.is_some() && a_index < 41 {
                let facility = data.unwrap();
                facility.condition_cid = format!("CID_{}", RECRUIT_CID[a_index] ).into() ;
                for y in 0..hub_dispos[1].len() {
                    if let Some(hub_locator) = hub_dispos[1][y].locator.as_ref() {
                        if hub_locator.to_string() == locator[x] {
                            hub_dispos[1][y].set_chapter(RECRUIT_CID[a_index].into() );
                            break;
                        }
                    }
                }
            }
        }
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
                let list = &get_data_read().playables;
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
                pids.iter().for_each(|pid| {
                    GameVariableManager::make_entry_str(format!("G_R_{}", pid.as_str()).as_str(), pid.as_str());
                    GameVariableManager::make_entry_str(format!("G_R2_{}", pid.as_str()).as_str(), pid.as_str());
                });

                // println!("Playable Unit Size: {}", playable_list.len());
                // Alear and somniel royals must be switched with non-dlc units
                //  x_i in to_replace, x_j in playable_list, royals are x_i
                //  x_j -> x_i, remove royal (x_i) from to_replace and remove x_j from playable_list
                let royals = [0, 23, 4, 17, 14, 27];
                for x_royal in royals {
                    if let Some((pos, royal)) =  playable_list.iter().enumerate().find(|(i, x)| x.0 == x_royal) {
                        if let Some(royal_replacement) = to_replace_list.get_remove_filter(rng, |(i, gender)| *i < 36 && *i != 30 && ((with_gender && *gender == royal.1) || !with_gender)){
                            DVCVariables::set_person_recruitment(royal_replacement.0 as i32, royal.0 as i32);
                            println!("#{}: {} -> {}", royal_replacement.0, Mess::get_name(PIDS[royal_replacement.0]), Mess::get_name(PIDS[x_royal]));
                            playable_list.remove(pos);
                        }
                    }
                }
                to_replace_list.iter().for_each(|(index_x, gender_x)|{
                    if let Some((index_y, _)) = playable_list.get_remove_filter(rng, |(i, gender_y)| (with_gender && gender_x == gender_y) || !with_gender)
                        .or_else(|| playable_list.get_remove(rng))
                    {
                        GameVariableManager::set_string(format!("G_R_{}", pids[*index_x].as_str()), pids[index_y].as_str());
                        GameVariableManager::set_string(format!("G_R2_{}", pids[index_y].as_str()), pids[*index_x].as_str());
                        if *index_x < pids.len() && index_y < pids.len() {
                            println!("#{}: {} -> {}", index_x, Mess::get_name(pids[*index_x].as_str()), Mess::get_name(pids[index_y].as_str()));
                        }
                    }
                });
            },
            2 => {   //Reverse
                for x in 0..41 { DVCVariables::set_person_recruitment(x, RR_ORDER[x as usize] as i32); }
            },
            3 => { // Custom
                let order =crate::DeploymentConfig::get().get_custom_recruitment(false);
                order.iter().for_each(|(p1, p2)|{
                    println!("{} -> {}", *p1, *p2);
                    println!("{} to {}", Mess::get_name(PIDS[*p1 as usize]), Mess::get_name(PIDS[*p2 as usize]));
                    DVCVariables::set_person_recruitment(*p1, *p2);
                });
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
                    if GameVariableManager::exist(&key) { GameVariableManager::set_string(&key, randomized.pid); }
                    else { GameVariableManager::make_entry_str(&key, randomized.pid); }
                    
                    if GameVariableManager::exist(&reversed) { GameVariableManager::set_string(&reversed, original.pid); }
                    else { GameVariableManager::make_entry_str(&reversed, original.pid); }
                    println!("{} -> {}", Mess::get_name(original.pid), Mess::get_name(randomized.pid));
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
    if !can_rand() || RANDOMIZER_STATUS.read().unwrap().alear_person_set  { return; }
    if DVCVariables::get_dvc_person(0, false).to_string() == PIDS[0] {
        let _ = RANDOMIZER_STATUS.try_write().map(|mut lock| lock.alear_person_set = true);
        if let Some(lueur) = UnitPool::get_from_pid("PID_リュール".into(), false) {
            if DVCVariables::ClassMode.get_value()== 1 && is_start && can_rand() {
                crate::randomizer::job::unit_change_to_random_class(lueur, false);
                unit::adjust_unit_items(lueur);
                // println!("Lueur Class Changed to {}", Mess::get_name(lueur.job.jid));
            }
        }
        return;
    }
    // println!("Lueur is {}", Mess::get_name(GameVariableManager::get_string("G_R_PID_リュール")));
    // remove hero status on alear and place it on the replacement and add alear skills on the replacement
    let person_lueur = PersonData::get(PIDS[0]).unwrap();
    let mut lueur_sids = person_lueur.get_common_sids().unwrap();
    if let Some(hero_sid) = lueur_sids.iter_mut().find(|x| x.to_string().contains("SID_主人公")) {
        *hero_sid =  "SID_無し".into();
    }
    person_lueur.on_completed();
    if let Some(new_hero) = switch_person(person_lueur) {
        if let Some(hero) = UnitPool::get_from_person_force_mask(&new_hero, -1) {
            hero.private_skill.add_sid("SID_主人公", SkillDataCategorys::Private, 0);
            hero.private_skill.add_sid("SID_王族", SkillDataCategorys::Private, 0);
            hero.private_skill.add_sid("SID_リベラシオン装備可能", SkillDataCategorys::Private, 0);
            hero.private_skill.add_sid("SID_ヴィレグランツ装備可能", SkillDataCategorys::Private, 0);
        }
        let sids = new_hero.get_common_sids().unwrap();

        let new_sids = Array::<&Il2CppString>::new_specific(sids.get_class(), sids.len() + 4).unwrap();
        for x in 0..sids.len() { new_sids[x] = sids[x]; }
        new_sids[sids.len()] = "SID_主人公".into();
        new_sids[sids.len() + 1] = "SID_リベラシオン装備可能".into();
        new_sids[sids.len() + 2] = "SID_ヴィレグランツ装備可能".into();
        new_sids[sids.len() + 3] = "SID_王族".into();
        new_hero.set_common_sids(new_sids);
        new_hero.on_completed();
        if let Some(god) = GodData::get_mut("GID_リュール") {
            god.link = Some(new_hero.pid);
            new_hero.set_link_god(Some(god));
        }
        if is_start {   // Move alear to force 5
            if let Some(lueur_unit) = UnitPool::get_from_pid(PIDS[0].into(), false) {
                unit::change_unit_autolevel(lueur_unit, true);
                if DVCVariables::ClassMode.get_value()== 1 {
                    super::job::unit_change_to_random_class(lueur_unit, true);
                    unit::fixed_unit_weapon_mask(lueur_unit);
                    unit::adjust_unit_items(lueur_unit);
                }
                lueur_unit.transfer(ForceType::Lost, false);
                get_lueur_name_gender(); // grab gender and name
                GameVariableManager::make_entry(DVCVariables::LUEUR_GENDER, lueur_unit.edit.gender);
            }
            if let Some(unit) = UnitUtil::join_unit_person(new_hero) {
                unit.edit.set_name(new_hero.get_name());
                unit.edit.set_gender(new_hero.get_gender());
                unit.private_skill.add_sid("SID_主人公", SkillDataCategorys::Private, 0);
                unit.private_skill.add_sid("SID_王族", SkillDataCategorys::Private, 0);
                unit.private_skill.add_sid("SID_リベラシオン装備可能", SkillDataCategorys::Private, 0);
                unit.private_skill.add_sid("SID_ヴィレグランツ装備可能", SkillDataCategorys::Private, 0);
                unit.transfer(ForceType::Absent, false);
            }
        }

        Patch::in_text(0x02d524e0).nop().unwrap();
        Patch::in_text(0x02d524e4).nop().unwrap();

        // LueurW_God or Lueur_God in GetPath
        if GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 {
            Patch::in_text(0x02d524e8).bytes(&[0x48, 0x00, 0x80, 0x52]).unwrap();
            person_lueur.set_gender(Gender::Female);
        } else {
            Patch::in_text(0x02d524e8).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap();
            person_lueur.set_gender(Gender::Male);
        }

        Patch::in_text(0x0233f104).bytes(&[0x01, 0x10, 0x80, 0x52]).unwrap(); // GodUnit$$GetName ignore hero flag on Emblem Alear
        let lueur_god_offsets = [0x02d51dec, 0x021e12ac, 0x02915844, 0x02915844, 0x02915694, 0x01c666ac, 0x02081edc, 0x01c69d60, 0x01c66588];
        for x in lueur_god_offsets { mov_x0_0(x); }

        // For Hub-Related Activities
        let offsets = [0x02ae8d28, 0x02ae9000, 0x02a5d0f4, 0x01cfd4c4, 0x01d03184, 0x01e5fe00, 0x01e5ff4c, 0x027049c8];
        let new_hero_gender = if new_hero.get_gender() == 2 || (new_hero.get_gender() == 1 && new_hero.flag.value & 32 != 0) { 2 } else { 1 };
        for x in offsets {
            if new_hero_gender == 1 {  mov_x0_0(x); }
            else { mov_1(x); }
        }
        if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() {
            lock.alear_person_set = true;
            lock.set_enable();
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

pub fn get_low_class_index(this: &PersonData) -> usize {
    let apt = this.aptitude.value;
    for x in 0..3 { if apt & (1 << (x+1) ) != 0 { return x; } }
    let apt2 = this.sub_aptitude.value;
    for x in 0..3 { if apt2 & (1 << (x+1) ) != 0 { return x; } }
    0
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
    if GameUserData::get_chapter().cid.to_string() == "CID_M011" && lueur_on_map() {
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