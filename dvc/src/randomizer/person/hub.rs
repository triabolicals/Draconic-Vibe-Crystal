use engage::gamedata::hub::{HubDisposData, HubRandomSet};
pub use super::*;
use crate::config::DVCVariables;

pub fn change_somniel_hub_dispos() {
    let hub_list = HubDisposData::try_get_mut("Hub_Solanel").unwrap();
    if !GameUserData::is_cid_completed("CID_M004") && DVCVariables::EmblemRecruitment.get_value() != 0 {   //Sigurd Replacement
        if let Some(sigurd) = hub_list.iter_mut().find(|dispos| dispos.get_aid().filter(|aid| aid.to_string().contains("シグルド")).is_some()) {
            sigurd.set_aid( GameVariableManager::get_string("G_R_GID_シグルド"));
        }
    }
    if DVCVariables::UnitRecruitment.get_value()  > 0 {
        let pid = DVCVariables::get_dvc_person(0, false);
        HubRandomSet::get_list_mut().unwrap().iter_mut()
            .flat_map(|x| x.iter_mut()).filter(|x| x.iid == pid)
            .for_each(|x| x.iid = PIDS[0].into() );
    }
    if DVCVariables::Continuous.get_value() > 2 {
        if let Some(last_chapter_locator) = hub_list.iter_mut().find(|dispos| dispos.locator.is_some_and(|v|v.str_contains("LastChapter"))){
            last_chapter_locator.chapter = Some("M022".into());
        }
    }
}

pub fn change_kizuna_dispos() {
    let unit_order_changed = DVCVariables::UnitRecruitment.get_value() != 0;
    let emblem_order_changed = DVCVariables::EmblemRecruitment.get_value() != 0;
    let generics = DVCFlags::RandomBossesNPCs.get_value();
    if !unit_order_changed && !emblem_order_changed && !generics{ return; }
    let chapter =  GameUserData::get_chapter();
    let cid = chapter.field;
    if let Some(hub_list) = HubDisposData::try_get_mut(cid) {
        if unit_order_changed {
            hub_list.iter()
                .for_each(|dispos| {
                    if let Some(aid) = dispos.get_aid() {
                        let key = format!("G_R_{}", aid);
                        if unit_order_changed && GameVariableManager::exist(key.as_str()) {
                            let pid = GameVariableManager::get_string(key);
                            dispos.set_aid( pid);
                        }
                    }
                }
            );
        }
        if emblem_order_changed {
            if chapter.prefixless_cid.str_contains("G00") {
                if let Some(emblem) = hub_list.iter_mut()
                    .find(|dispos| dispos.get_aid().is_some_and(|aid| aid.str_contains("GID_")))
                {
                    let gid = emblem.get_aid().unwrap();
                    emblem.set_aid( GameVariableManager::get_string(format!("G_R_{}", gid)));
                }
            }
        }
    }
}
