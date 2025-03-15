pub use super::*;
use crate::config::DVCVariables;

pub fn change_somniel_hub_dispos() {
    let hub_list = HubDisposData::try_get_mut("Hub_Solanel").unwrap();
    if !GameUserData::is_cid_completed("CID_M004") && GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {   //Sigurd Replacement
        if let Some(sigurd) = hub_list.iter_mut().find(|dispos| dispos.get_aid().filter(|aid| aid.to_string().contains("シグルド")).is_some()) {
            sigurd.set_aid( GameVariableManager::get_string(format!("G_R_GID_シグルド")));
        }
    }
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) > 0 {
        let pid = DVCVariables::get_dvc_person(0, true);
        HubRandomSet::get_list_mut().unwrap().iter_mut()
            .flat_map(|x| x.iter_mut()).filter(|x| x.iid == pid)
            .for_each(|x| x.iid = PIDS[0].into() );
    }
    if GameVariableManager::get_number(DVCVariables::CONTINIOUS) >= 4 {
        if let Some(last_chapter_locator) = hub_list.iter_mut().find(|dispos| dispos.locator.contains("LastChapter")) {
            last_chapter_locator.chapter = Some("M022".into());
        }

    }
}

pub fn change_kizuna_dispos() {
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 && GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { return; }
    if crate::randomizer::RANDOMIZER_STATUS.read().unwrap().kizuna_replacements {return; }
    let chapter =  GameUserData::get_chapter();
    let cid = chapter.field;
    if let Some(hub_list) = HubDisposData::try_get_mut(cid) {
        if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) > 0 {
            hub_list.iter().filter(|dispos| dispos.get_locator().to_string().contains("LocatorStory"))
                .for_each(|dispos| {
                    if let Some(aid) = dispos.get_aid() {
                        let key = format!("G_R_{}", aid);
                        if GameVariableManager::exist(key.as_str()) {
                            let pid = GameVariableManager::get_string(key);
                            dispos.set_aid( pid); 
                        }
                    }
                }
            );
        }
     //Replace Emblem in Divine Paralogue
        if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
            if chapter.get_prefixless_cid().to_string().contains("G00") {
                if let Some(emblem) = hub_list.iter_mut().find(|dispos| dispos.get_aid().is_some_and(|aid| aid.to_string().contains("GID_"))) {
                    let gid = emblem.get_aid().unwrap();
                    emblem.set_aid( GameVariableManager::get_string(format!("G_R_{}", gid)));
                }
            }
        }
    }
    crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.kizuna_replacements = true).unwrap();
}
