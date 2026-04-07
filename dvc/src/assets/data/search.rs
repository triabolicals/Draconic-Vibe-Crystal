use super::*;

/*
pub fn asset_table_search(mode: i32, con: &Vec<&str>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let search_conditions_indexes: Vec<i32> = con.iter().map(|con| AssetTableStaticFields::get_condition_index(con) ).collect();
    let conditions_size = search_conditions_indexes.len();
    asset_table_sf.search_lists[mode as usize].iter()
        .filter(|entry| entry.mode == mode && entry.condition_indexes.list.iter().count() == conditions_size)
        .find(|entry| search_conditions_indexes.iter().all(|&search| entry.condition_indexes.list.iter().any(|con_index| con_index.iter().any(|&index| index == search)) )
    )
}
pub fn search_by_key<'a>(mode: i32, key: impl Into<&'static Il2CppString>, start: Option<i32>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let key_index  = AssetTableStaticFields::get_condition_index(key);
    if key_index < 1 { return None; }
    let start_index = start.unwrap_or(0);
    asset_table_sf.search_lists[mode as usize].iter().find(|entry| entry.mode == mode && entry.parent.index > start_index && has_condition(entry, key_index))
}


pub fn search_by_2_keys<'a>(mode: i32, key1: impl Into<&'static Il2CppString>,  key2: impl Into<&'static Il2CppString>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let key1_index  = AssetTableStaticFields::get_condition_index(key1);
    let key2_index  = AssetTableStaticFields::get_condition_index(key2);
    if key1_index < 1 || key2_index < 1 { return None; }
    asset_table_sf.search_lists[mode as usize].iter().find(|entry| 
        entry.mode == mode && has_condition(entry, key1_index) && has_condition(entry, key2_index)
    )
}
pub fn search_by_3_keys<'a>(mode: i32, key1: impl Into<&'static Il2CppString>,  key2: impl Into<&'static Il2CppString>, key3: impl Into<&'static Il2CppString>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let key1_index = AssetTableStaticFields::get_condition_index(key1);
    let key2_index = AssetTableStaticFields::get_condition_index(key2);
    let key3_index = AssetTableStaticFields::get_condition_index(key3);
    if key1_index < 1 || key2_index < 1 { return None; }
    asset_table_sf.search_lists[mode as usize].iter().find(|entry| 
        entry.mode == mode && 
        has_condition(entry, key1_index) && 
        has_condition(entry, key2_index) &&
        has_condition(entry, key3_index) 
    )
}

 */


pub fn search_by_iid(iid: &'static Il2CppString, mode: i32) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let iid_index  = AssetTableStaticFields::get_condition_index(iid);
    if iid_index < 1 { return None }
    asset_table_sf.search_lists[mode as usize].iter()
        .find(|entry| has_condition(entry, iid_index) && ( entry.left_hand.is_some() || entry.right_hand.is_some()) )
}

pub fn has_condition(entry: &AssetTable, condition_index: i32) -> bool { 
    entry.condition_indexes.list.iter().any(|s| s.iter().any(|&index| index ==  condition_index)) 
}

pub fn has_condition_with(entry: &AssetTable, condition_index: i32, is_accessory: bool) -> bool {
    let sf = AssetTableStaticFields::get();
    let bit = &sf.condition_flags.bits;
    if is_accessory {
        let count = entry.condition_indexes.list.iter().filter(|x| x.iter().any(|s| bit.get(*s) || *s == condition_index) ).count();
        count >= entry.condition_indexes.list.len()
    }
    else { entry.condition_indexes.list.iter().all(|x| x.iter().any(|s| bit.get(*s) || *s == condition_index) ) }
}
