use super::*;

pub struct BustGauge;
impl ConfigBasicMenuItemGaugeMethods  for BustGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = CONFIG.lock().unwrap().misc_option_1 / 5.0;
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = CONFIG.lock().unwrap().misc_option_1 / 5.0;
        let result = ConfigBasicMenuItem::change_key_value_f(gauge, 0.0, 1.0, 0.05);
        let value = (result * 100.0).trunc() / 100.0; 
        if gauge != value {
            CONFIG.lock().unwrap().misc_option_1 = 5.0*value;
            this.gauge_ratio = result;
            Self::set_help_text(this, None);
            this.update_text();
            CONFIG.lock().unwrap().save();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if this.gauge_ratio <= 0.09 { this.help_text = "Current Volume Value: Default".into() }
        else { this.help_text = format!("Current Volume Value: {:2}", this.gauge_ratio*5.0).into();  }
        ASSET_DATA.lock().unwrap().apply_bust_changes();
    }
}

pub extern "C" fn vibe_bust() -> &'static mut ConfigBasicMenuItem {
    ConfigBasicMenuItem::new_gauge::<BustGauge>("Unit Bust Volume Slider")
}

pub fn get_bust_values() {
    let static_fields = &Il2CppClass::from_name("App", "AssetTable").unwrap().get_static_fields::<AssetTableStaticFields>().search_lists[2];
    ASSET_DATA.lock().unwrap().bust_values.clear();
    for x in 0..static_fields.len() {
        let volume_bust = static_fields[x as usize].scale_stuff[11];
        if volume_bust > 0.10 {
            ASSET_DATA.lock().unwrap().bust_values.push( (x as i32, volume_bust) ); 
        }
    }
}