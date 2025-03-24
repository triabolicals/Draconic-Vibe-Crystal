use super::*;


pub fn asset_table_search(mode: i32, con: &Vec<&str>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let search_conditions_indexes: Vec<i32> = con.iter().map(|con| AssetTableStaticFields::get_condition_index(con) ).collect();
    let conditions_size = search_conditions_indexes.len();
    asset_table_sf.search_lists[mode as usize].iter()
        .filter(|entry| entry.condition_indexes.list.iter().count() == conditions_size)
        .find(|entry| search_conditions_indexes.iter().all(|&search| entry.condition_indexes.list.iter().any(|con_index| con_index.iter().any(|&index| index == search)) )
    )
}
pub fn search_for_body_anim(mode: i32, act_prefix: &str, unique_id: &str) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    asset_table_sf.search_lists[mode as usize].iter().find(|entry|  entry.body_anim.is_some_and(|b| b.to_string().contains(act_prefix) && b.to_string().contains(unique_id) ) )
}
pub fn search_by_jid(mode: i32, jid: &Il2CppString) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let jid_index  = AssetTableStaticFields::get_condition_index(jid);
    if jid_index < 1 { return None; }
    asset_table_sf.search_lists[mode as usize].iter().find(|entry| 
        entry.condition_indexes.list.iter().any(|s| s.iter().any(|&index| index == jid_index)) &&
        entry.dress_model.is_some() || entry.body_model.is_some() )
}

pub fn search_by_key<'a>(mode: i32, key: impl Into<&'a Il2CppString>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let key_index  = AssetTableStaticFields::get_condition_index(key);
    if key_index < 1 { return None; }
    asset_table_sf.search_lists[mode as usize].iter().find(|entry| has_condition(entry, key_index))
}

pub fn search_by_key_with_dress<'a>(mode: i32, key: impl Into<&'a Il2CppString>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let key_index  = AssetTableStaticFields::get_condition_index(key);
    if key_index < 1 { return None; }
    asset_table_sf.search_lists[mode as usize].iter().find(|entry| has_condition(entry, key_index) && 
       entry.dress_model.or(entry.body_model).is_some_and(|dress|{ let d = dress.to_string(); d.contains("M_c") || d.contains("F_c") }) &&
       entry.head_model.is_some_and(|head| !head.to_string().contains("null"))
    )
}

pub fn search_by_2_keys<'a>(mode: i32, key1: impl Into<&'a Il2CppString>,  key2: impl Into<&'a Il2CppString>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let key1_index  = AssetTableStaticFields::get_condition_index(key1);
    let key2_index  = AssetTableStaticFields::get_condition_index(key2);
    if key1_index < 1 || key2_index < 1 { return None; }
    asset_table_sf.search_lists[mode as usize].iter().find(|entry| 
        has_condition(entry, key1_index) && 
        has_condition(entry, key2_index)
    )
}

pub fn search_by_3_keys<'a>(mode: i32, key1: impl Into<&'a Il2CppString>,  key2: impl Into<&'a Il2CppString>, key3: impl Into<&'a Il2CppString>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let key1_index  = AssetTableStaticFields::get_condition_index(key1);
    let key2_index  = AssetTableStaticFields::get_condition_index(key2);
    let key3_index  = AssetTableStaticFields::get_condition_index(key3);
    if key1_index < 1 || key2_index < 1 { return None; }
    asset_table_sf.search_lists[mode as usize].iter().find(|entry| 
        has_condition(entry, key1_index) && 
        has_condition(entry, key2_index) &&
        has_condition(entry, key3_index) 
    )
}


pub fn search_by_iid(iid: &Il2CppString) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let iid_index  = AssetTableStaticFields::get_condition_index(iid);
    if iid_index < 1 { return None }
    asset_table_sf.search_lists[2].iter().find(|entry| has_condition(entry, iid_index) && ( entry.left_hand.is_some() || entry.right_hand.is_some()) )
}

pub fn has_condition(entry: &AssetTable, condition_index: i32) -> bool { 
    entry.condition_indexes.list.iter().any(|s| s.iter().any(|&index| index ==  condition_index)) 
}