use engage::app::assettable::*;
// use unity::{Cast, Il2CppString};
// use crate::dvc::asset::new_accessory;

pub fn edit_result_for_gmap(result: AssetTable_Result) {
    let scale = if result.get_ride_model().is_null() { 4.8_f32 } else { 4.15_f32 };
    result.set_map_scale_all(scale);
    /*
    result.set_ride_model(Il2CppString::null());
    let body_model = result.get_body_model().to_rust_string();
    if body_model.contains("F_c") { result.set_body_anim("UAS_Map0AF"); }
    else { result.set_body_anim("UAS_Map0AF"); }
    result.commit_8(new_accessory("l_shld_loc", "null"));
    result.replace(AssetTable_Modes::onmap());
     */
}