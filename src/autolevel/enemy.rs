use super::*;

pub struct EnemyEmblemGauge;
impl ConfigBasicMenuItemGaugeMethods  for EnemyEmblemGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_emblem_rate as f32 / 100.0 }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_EMBLEM_KEY) as f32 / 100.0  };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_emblem_rate }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_EMBLEM_KEY) };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 100, 10);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_emblem_rate = result; }
            else { GameVariableManager::set_number(DVCVariables::ENEMY_EMBLEM_KEY,  result); }
            this.gauge_ratio = 0.01 * result as f32;
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new(); 
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_emblem_rate }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_EMBLEM_KEY) };

        if value == 0 { this.help_text = "Enemy units will not have an chance to equipped emblems.".into(); }
        else {  this.help_text = format!("{}% chance of enemy units equipped with an emblem.", value).into();  }

    }
}


pub extern "C" fn vibe_enemy_emblem() -> &'static mut ConfigBasicMenuItem { 
    let enemy_emblem = ConfigBasicMenuItem::new_gauge::<EnemyEmblemGauge>("Enemy Emblem Rate"); 
    enemy_emblem.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::not_in_map_sortie_build_attr as _);
    enemy_emblem
}

pub fn try_equip_emblem(unit: &Unit, emblem: usize) -> bool {
    if !GameVariableManager::exist("EnemyEmblemSet") { GameVariableManager::make_entry_norewind("EnemyEmblemSet", 0); }
    let mut emblem_set_flag = GameVariableManager::get_number("EnemyEmblemSet");
    if unit.person.gender == 0 { return false; }
    if emblem < 31 { if emblem_set_flag & (1 << emblem) != 0 { return false; }  }

    if let Some(god) = GodData::try_index_get(ENEMY_EMBLEM_LIST.get().unwrap()[emblem]) {
        if let Some(god_unit) = engage::godpool::GodPool::create(god) {
            let valid = unit.try_connect_god(god_unit).is_some();
            god_unit.set_escape(true);
            if valid && emblem < 31 {
                emblem_set_flag |= 1 << emblem;
                GameVariableManager::set_number("EnemyEmblemSet", emblem_set_flag);
            }
            return valid;
        }
    }
    return false;
}
