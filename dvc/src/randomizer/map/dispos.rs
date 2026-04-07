use engage::gamedata::dispos::DisposData;
use engage::gamedata::GamedataArray;
use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use crate::config::{DVCVariables};
use crate::utils::can_rand;

pub fn change_map_dispos() {
    if !can_rand() { return; }
    if let Some(dispos) =  DisposData::get_list_mut().as_mut() {
        // println!("Changing Map Dispos");
        let chapter = GameUserData::get_chapter();
        let lythos_crew = chapter.cid.str_contains("M001") || chapter.cid.str_contains("M002") || chapter.cid.str_contains("M003");
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
    }
}
