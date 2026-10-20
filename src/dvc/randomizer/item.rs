use super::*;
use engage::{
    app::{
        transporter::*, unititem::*, unititemlist::*, wellsequence::*,
        CommonRewardSequence, IBitField32,
    },
    prelude::List_1
};
use crate::dvc::unit::for_each_unit;

pub fn process_non_unit_inventory_item(unit_item: UnitItem) {
    let kind = unit_item.m_item().get_kind().value;
    let user_data = DVCVariables::user_data();
    let price = unit_item.m_item().get_price();
    match kind {
        14 => { user_data.set_refine_iron(user_data.get_refine_iron() + 1); }
        15 => { user_data.set_refine_steel(user_data.get_refine_steel() + 1); }
        16 => { user_data.set_refine_silver(user_data.get_refine_silver() + 1); }
        17 => { user_data.set_m_piece_of_bond(user_data.get_piece_of_bond() + price); }
        18 => { user_data.set_m_gold(user_data.m_gold() + price); }
        _ => { unit_item.m_item().add_inventory(1); }
    }
}
pub fn adjust_non_unit_items_inventory() {
    for x in 0..999 {
        let item = Transporter::get(x);
        let i = item.m_unit_item().m_item();
        if i.index() > 3 && (i.get_kind().value > 13 || i.is_inventory()) {
            process_non_unit_inventory_item(item.m_unit_item());
            Transporter::delete(x);
        }
    }
    for_each_unit(25u32, |unit| {
        let mut changed = false;
        let list = unit.get_item_list();
        list.m_unit_items().iter().filter(|ui| ui.m_item().is_inventory() || ui.m_item().get_kind().value > 13)
            .for_each(|ui| {
                process_non_unit_inventory_item(ui);
                ui.dispose();
                changed = true;
            });
        if changed && !unit.get_god_unit().is_null() { list.put_engage_item(unit.get_god_unit(), unit.is_engaging_2()); }
    });
}
pub extern "C" fn well_get_item_rng(proc: WellSequence, _method_info: OptionalMethod) {
    let item_flag = DVCFlags::WellItems.get_value();
    let use_flag = WellSequence::get_use_flag();
    let item_pool = GameData::get_item_pool();
    if item_pool.has_well && !item_flag  {
        proc.get_item();
        return;
    }
    if use_flag == WellSequence_UseFlags::item_return() {
        let pool = GameData::get_item_pool();
        let level = WellSequence::get_exchange_level();
        let seed = WellSequence::get_seed();
        let rng = Random_2::new_2(seed as u32);
        let map_completed = DVCVariables::chapter_number_complete(true);
        let list = List_1::<ItemData>::new();
        if !item_pool.has_well || level == 0 || map_completed  < 6 {
            list.add(ItemData::get("IID_スキルの書・守".into()));
            list.add(ItemData::get("IID_スキルの書・破".into()));
        }
        else if item_flag {
            let sum = 1500*(level + 2) + rng.get_value_2(10)*500*level;
            let count = 2*(level + 1 + rng.get_value_2(5) );
            let mut total_price = 0;
            let price_cap = 1000 + 750 * level;
            let mut n_items = 0;
            while total_price < sum && n_items < count  {
                let iid = pool.random_item(RandomItemReplacementType::Reward, false);
                if let Some(item) = utils::to_opt(ItemData::get(iid)).filter(|x| x.get_price() < price_cap && x.get_price() > 100 && x.get_flag().m_value() & 131 == 0){
                    total_price += item.get_price();
                    list.add(item);
                    n_items += 1;
                }
            }
            let is_continuous = DVCVariable::Continuous.get_value() != 0 || DVCFlags::RandomSP.get_value();
            if is_continuous || level > 0 { list.add(ItemData::get("IID_スキルの書・守".into())); }
            if is_continuous || ( level >= 2 && rng.get_value_2(2) != 0 ) { list.add(ItemData::get("IID_スキルの書・破".into())); }
            if level > 3 && ( rng.get_value_2(10) < level || is_continuous) { list.add(ItemData::get("IID_スキルの書・離".into())); }
        }
        else {
            list.add(ItemData::get("IID_スキルの書・守".into()));
            list.add(ItemData::get("IID_スキルの書・破".into()));
        }
        CommonRewardSequence::create_bind_for_well(proc, list, "MID_Hub_Well_ItemExchange_Get");
        WellSequence::set_seed(0);
        WellSequence::set_use_flag(WellSequence_UseFlags::not_use());
    }
}