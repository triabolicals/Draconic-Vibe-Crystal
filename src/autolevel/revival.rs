use super::*;


pub struct EnemyRevivalStones;
impl ConfigBasicMenuItemGaugeMethods  for EnemyRevivalStones {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().revival_stone_rate as f32 / 100.0 }
            else { GameVariableManager::get_number(DVCVariables::REVIVAL_STONE_GAUGE_KEY) as f32 / 100.0  };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().revival_stone_rate  }
            else { GameVariableManager::get_number(DVCVariables::REVIVAL_STONE_GAUGE_KEY) };

        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 100, 10);

        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().revival_stone_rate = result; }
            else { GameVariableManager::set_number(DVCVariables::REVIVAL_STONE_GAUGE_KEY, result); }
            this.gauge_ratio = 0.01 * result as f32;
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new(); 
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){ 
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().revival_stone_rate }
            else { GameVariableManager::get_number(DVCVariables::REVIVAL_STONE_GAUGE_KEY)};
        if value == 0 { this.help_text = "Enemy units will not gain revival stones.".into(); }
        else {
            this.help_text = format!("Chance of enemy units gaining a revival stone: {}%.", value).into();
        }
    }
}

pub extern "C" fn vibe_enemy_stones() -> &'static mut ConfigBasicMenuItem { 
    let enemy_stones = ConfigBasicMenuItem::new_gauge::<EnemyRevivalStones>("Enemy Revival Stone Rate"); 
    enemy_stones.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::not_in_map_sortie_build_attr as _);
    enemy_stones
}