use super::*;
use engage::{
    app::{
        disposdata::*, mapsortieposition::*,
        Gender, ISingletonClass_1Methods, IStructDataArray_1Methods, IStructData_1Methods,
    },
    ForceExt, List_1Ext
};
use outfit_core::Randomizer;

mod full;
mod shuffle;
mod sortie;

pub use full::*;
pub use shuffle::*;
pub use sortie::*;
use crate::utils::to_opt;

pub fn get_unit_rating(this: Unit) -> i32 {
    let mut result: i32 = 0;
    for x in 1..9 { result += this.get_capability(x, false);  }
    result
}
pub fn deployment_modes(){
    let user_data = GameUserData::get_instance();
    let hero_unit = UnitPool::get_hero(false);
    if hero_unit.is_null() { return; }
    let absent_force = Force::get(Force_Type::absent());
    if !DVCVariables::is_main_chapter_complete(3) { return; }

    if DVCVariable::UnitDeployment.get_value() == 3 {
        let v = user_data.m_status().m_value();
        if v & 64 != 0 { user_data.m_status().set_m_value(v & !64); }  //Disables Continuous Flag
    }
    let player_force = Force::get(Force_Type::player());
    let max_player = player_force.get_count();
    let absent_count = absent_force.get_count();
    let rng = Random_2::get_game();
    let unit_deployment_mode = DVCVariable::UnitDeployment.get_value();
    // Free deployment
    let must_sortie = Unit_Status {value: 20};     // Status for Must Deploy (4) and Cannot change Position (16)
    if unit_deployment_mode == 3 && DVCVariables::get_chapter_index() != 22  {
        if hero_unit.check_status(must_sortie) { hero_unit.clear_status(must_sortie); }
        return;
    }
    else if absent_count == 0 || user_data.is_evil_map() || unit_deployment_mode == 0 { return; }
    if unit_deployment_mode == 1 || unit_deployment_mode == 2 || unit_deployment_mode > 5 { Force::get(Force_Type::dead()).transfer(Force_Type::absent(), true); }
    let max = max_player as usize;
    let positions = get_deployment_positions(Some(max));
    if positions.is_empty() { return; }
    let mut position_iter = positions.into_iter();
    player_force.transfer(Force_Type::absent(), true);
    hero_unit.transfer(Force_Type::player(), false);
    hero_unit.try_create_actor();
    if let Some(hero_position) = position_iter.next(){ hero_unit.set_position_2(hero_position.0, hero_position.1, true); }
    if !user_data.is_encount_map() && unit_deployment_mode != 3 { hero_unit.set_status(must_sortie); }
    match unit_deployment_mode {
        1 => {
            let mut units: Vec<_> = Force::iter(absent_force).collect();
            while let Some((unit, pos)) = units.get_remove(rng).zip(position_iter.next()){
                transfer_and_set_position(unit, pos, true);
            }
        }
        2|5 => {
            let mut units: Vec<_> = Force::iter(absent_force).map(|u| (u, get_unit_rating(u))).collect();
            if unit_deployment_mode == 2 { units.sort_by(|a, b| a.1.cmp(&b.1)) } else { units.sort_by(|a, b| b.1.cmp(&a.1)); }
            units.iter().zip(position_iter).for_each(|((unit, _), pos)| { transfer_and_set_position(*unit, pos, true); });
        }
        6 | 7 => {
            let gender = if unit_deployment_mode == 7 { Gender::female() } else { Gender::male() };
            Force::iter(absent_force).filter(|u| u.get_gender() == gender)
                .zip(position_iter)
                .for_each(|(unit, pos)|{ transfer_and_set_position(unit, pos, true); });
        }
        _ => {}
    }
}
pub fn get_deployment_positions(max_count: Option<usize>) -> Vec<(i32, i32, f32)> {
    let max = max_count.unwrap_or(64);
    if let Some(data) = to_opt(MapSortiePosition::get_instance()) {
        data.m_data().iter().enumerate().filter(|(i, _)| *i < max)
            .map(|(_, d)|{( d.get_x(), d.get_z(), d.get_angle()) })
            .collect()
    }
    else if let Some(player) = to_opt(DisposData::get("Player1".into())).or_else(|| to_opt(DisposData::get("Player".into()))) {
        player.base().base().iter().enumerate().filter(|(i, _)| *i < max)
            .map(|(_, dispos)| (dispos.get_dispos_x() as i32, dispos.get_dispos_y() as i32, (dispos.get_direction().value * 45) as f32))
            .collect()
    }
    else { vec![] }
}
fn transfer_and_set_position(unit: Unit, pos: (i32, i32, f32), is_last: bool){
    unit.transfer(Force_Type::player(), is_last);
    unit.try_create_actor();
    unit.set_position_2(pos.0, pos.1, true);
    unit.set_rotation(pos.2);
}