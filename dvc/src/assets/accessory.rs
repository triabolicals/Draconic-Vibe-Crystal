use super::*;

pub fn new_asset_table_accessory(model: &str, loc: &str) -> &'static mut AssetTableAccessory {
    let new_accessory = AssetTableAccessory::class().instantiate_as::<AssetTableAccessory>().unwrap();
    new_accessory.model = Some(model.into() );
    new_accessory.locator = Some(loc.into());
    new_accessory
}

pub fn add_accessory_to_list(list: &mut AssetTableAccessoryList, model: &str, location: &str) {
    let new_accessory = new_asset_table_accessory(model, location);
    list.try_add(new_accessory);
}
pub fn clear_accessory_from_list(list: &mut AssetTableAccessoryList, model: &str) {
    for x in 0..list.list.len() {
        if let Some(accessory_model) = list.list[x].model {
            if accessory_model.str_contains(model) {
                list.list[x].model = Some("null".into());
            }
        }
    }
}

pub fn clear_accessory_at_locator(list: &mut AssetTableAccessoryList, locator: &str) {
    if let Some(acc) = list.list.iter_mut()
        .find(|acc| acc.locator.is_some_and(|loc| loc.to_string() == locator)) {
            acc.model = Some("null".into());
        }
}
pub fn change_accessory(list: &mut AssetTableAccessoryList, model: &str, locator: &str){
    if model != "null" {
        // check if accessory exists 
        if list.list.iter_mut().any(|f| f.model.filter(|m| m.str_contains(model)).is_some() ) { return; }
    }
    // check if locator exists then replace the model
    if let Some(acc) = list.list.iter_mut().find(|f| f.locator.is_some_and(|m|m.str_contains(locator))) {
        acc.model = Some(model.into());
    }
    else { add_accessory_to_list(list, model, locator); }
}