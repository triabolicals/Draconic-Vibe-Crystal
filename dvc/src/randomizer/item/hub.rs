use engage::sequence::hub::HubSequence;
use super::*;

pub fn hub_item_randomized() {
    if !DVCVariables::random_enabled() || DVCVariables::ExplorationItem.get_value() == 0 { return; }
    if let Some(locator_group) = HubSequence::get_instance().and_then(|s| s.get_locator_group()){
        let rng = Random::get_system();
        locator_group.access_list.iter_mut()
            .flat_map(|a| a.access_data.as_mut())
            .for_each(|access|{
                if access.aid.is_some_and(|aid| aid.to_string().contains("IID")) {
                    if ItemData::get(access.aid.unwrap()).is_some_and(|item| item.kind < 14 ) {
                        access.aid = Some(GameData::get_random_item(2, false));
                        access.item_count = rng.get_value(2) + 1;
                    }
                }
        });
    }
}

