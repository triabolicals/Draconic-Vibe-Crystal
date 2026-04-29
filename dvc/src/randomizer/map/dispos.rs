use engage::{
    gamedata::{dispos::DisposData, GamedataArray},
    gameuserdata::GameUserData, gamevariable::GameVariableManager,
};
use crate::{
    config::DVCVariables, enums::{EMBLEM_GIDS, EMBLEM_PARA},
    randomizer::data::EmblemPool, utils::{can_rand, clamp_value},
};
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
                    if GameVariableManager::exist(key.as_str()) { dispos.gid = Some(GameVariableManager::get_string(key.as_str())); }
                }
            }
        });
        if DVCVariables::EmblemRecruitment.get_value() != 0 {
            if let Some(pos) = EMBLEM_PARA.iter().position(|x| cid.ends_with(*x)) {
                let recruit_idx = DVCVariables::get_dvc_emblem_index(pos as i32, true);
                if recruit_idx >= 12 { return; }
                let mut levels = [(0, 0); 3];
                for x in 1..4 {
                    levels[x-1] = (PARALOGUE_LEVELS[4*pos + x], PARALOGUE_LEVELS[4*recruit_idx + x]);
                }
                let thief_level = clamp_value((levels[1].1 - if pos == 6 { 3 } else { 5 }) as i32, 20, 40) as u8;
                let mut custom = false;
                dispos.iter_mut()
                    .flat_map(|array| array.iter_mut())
                    .filter(|dispos| {
                        dispos.get_person().is_some_and(|p| p.asset_force > 0)
                    })
                    .for_each(|dispos| {
                        if let Some(person) = dispos.get_person() {
                            if person.engage_skill.is_some() {
                                if EmblemPool::get_dvc_emblem_data(EMBLEM_GIDS[pos]).filter(|v| EmblemPool::is_custom(v)).is_some() {
                                    for x in 0..6 {
                                        dispos.items[x].iid = None;
                                        dispos.items[x].drop = 0;
                                    }
                                    custom = true;
                                }
                                else if pos != recruit_idx && !custom { set_dispos_levels(dispos, levels[0].1); }
                            }
                            else if pos != recruit_idx && !custom {
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
}
