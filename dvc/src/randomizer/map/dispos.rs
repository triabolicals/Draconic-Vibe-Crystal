use engage::gamedata::dispos::DisposData;
use engage::gamedata::{Gamedata, GamedataArray, ItemData, PersonData};
use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use crate::config::{DVCVariables};
use crate::enums::EMBLEM_PARA;
use crate::randomizer::data::GameData;
use crate::utils::{can_rand, clamp_value};

// Recommended Level, Emblem Level, Lowest Generic Level, Special Enemy Level
pub const PARALOGUE_LEVELS: [u8; 48] = [
    34, 40, 15, 20,
    25, 30, 10, 20,
    30, 35, 12, 15,
    28, 35, 8, 13,
    30, 35, 11, 28,
    28, 35, 10, 15,
    19, 25, 1, 2,
    19, 28, 2, 4,
    23, 30, 4, 6,
    25, 32, 6, 10,
    27, 32, 6, 8,
    28, 35, 7, 9,
];
pub fn change_map_dispos() {
    if !can_rand() { return; }
    if let Some(dispos) =  DisposData::get_list_mut().as_mut() {
        let chapter = GameUserData::get_chapter();
        let cid = chapter.cid.to_string();
        let lythos_crew = cid.contains("M001") || cid.contains("M002") || cid.contains("M003");
        dispos.iter_mut().flat_map(|array| array.iter_mut()).for_each(|dispos| {
            if DVCVariables::UnitRecruitment.get_value() != 0 {
                if let Some(lythos_person) = dispos.get_person().filter(|p| p.parent.index > 0 && p.parent.index < 5 ) {
                    if lythos_person.parent.index == 1 || lythos_crew {
                        dispos.set_pid(Some(DVCVariables::get_dvc_person(lythos_person.parent.index - 1, false)));
                    }
                }
            }
            if DVCVariables::EmblemRecruitment.get_value() != 0 {
                if let Some(gid) = dispos.gid {
                    let key = format!("G_R_{}", gid);
                    if GameVariableManager::exist(key.as_str()) {
                        dispos.gid = Some(GameVariableManager::get_string(key.as_str()));
                    }
                }
            }
        });
        if DVCVariables::EmblemRecruitment.get_value() != 0 {
            if let Some(pos) = EMBLEM_PARA.iter().position(|x| cid.ends_with(*x)) {
                let recruit_idx = DVCVariables::get_dvc_emblem_index(pos as i32, true);
                if pos >= 12 { return; }
                let mut levels = [(0, 0); 3];
                for x in 1..4 {
                    levels[x-1] = (PARALOGUE_LEVELS[4*pos + x], PARALOGUE_LEVELS[4*recruit_idx + x]);
                }
                let thief_level = clamp_value((levels[1].1 - if pos == 6 { 3 } else { 5 }) as i32, 20, 40) as u8;

                dispos.iter_mut()
                    .flat_map(|array| array.iter_mut())
                    .filter(|dispos| {
                        dispos.get_person().is_some_and(|p| p.asset_force > 0)
                    })
                    .for_each(|dispos| {
                        if let Some(person) = dispos.get_person() {
                            if person.engage_skill.is_some() {
                                if pos != recruit_idx { set_dispos_levels(dispos, levels[0].1); } else if GameData::get().emblem_pool.emblem_persons.iter()
                                    .find(|x| x.hash == person.parent.hash).is_some_and(|v| v.is_custom())
                                {
                                    if let Some(items) = DVCVariables::get_god_from_index(pos as i32, true)
                                        .and_then(|g| PersonData::get(g.gid.to_string().replace("GID_", "PID_闘技場_")))
                                        .and_then(|p| p.items.as_ref())
                                    {
                                        let mut count = 0;
                                        items.iter().for_each(|iid| {
                                            dispos.items[count].iid = Some(iid.to_string().into());
                                            dispos.items[count].drop = 0;
                                            count += 1;
                                        });
                                        for x in count..6 {
                                            dispos.items[x].iid = None;
                                            dispos.items[x].drop = 0;
                                        }
                                    }
                                }
                            } else if pos != recruit_idx {
                                if let Some(job) = dispos.get_job().or(person.get_job()) {
                                    if job.parent.hash == -1001243599 &&
                                        dispos.ai_action_value.is_some_and(|ai| ai.str_contains("Treasure") || ai.str_contains("Terrain"))
                                    {
                                        set_dispos_levels(dispos, thief_level);
                                    }
                                    else if job.is_low() && job.max_level == 20 { set_dispos_levels(dispos, 20); } else if person.bmap_size == 1 {
                                        let person_level = person.level as u8;
                                        if person_level < levels[2].0 {
                                            let difference = levels[1].0 - person_level;
                                            let new_level = clamp_value((levels[1].1 + difference) as i32, 1, 20) as u8;
                                            if job.max_level == 40 { set_dispos_levels(dispos, new_level + 20); } else { set_dispos_levels(dispos, new_level); }
                                        }
                                        else {
                                            if job.max_level == 40 { set_dispos_levels(dispos, 20 + levels[2].1); }
                                            else { set_dispos_levels(dispos, levels[2].1); }
                                        }
                                    }
                                }
                            }
                        }
                    });
            }
        }
    }
}
fn set_dispos_levels(data: &mut DisposData, level: u8) {
    data.level_h = level;
    data.level_n = level;
    data.level_l = level;
    println!("Level of {}, set to {}", data.get_person().unwrap().get_name(), level)
}
