use super::*;
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