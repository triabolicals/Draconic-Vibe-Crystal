use engage::app::{AssetTable_Modes, AssetTable_Result, IAssetTable_ResultMethods};
use unity::{Il2CppString};
use crate::dvc::asset::new_accessory;

pub fn edit_result_for_gmap(result: AssetTable_Result) {
    result.set_map_scale_all(4.8);
    result.set_ride_model(Il2CppString::null());
    let body_model = result.get_body_model().to_rust_string();
    if body_model.contains("F_c") { result.set_body_anim("UAS_Map0AF"); }
    else { result.set_body_anim("UAS_Map0AF"); }
    result.commit_8(new_accessory("l_shld_loc", "null"));
    result.replace(AssetTable_Modes::onmap());
}