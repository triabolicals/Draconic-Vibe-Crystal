use super::*;
use engage::sequence::{
    commonrewardsequence::CommonRewardSequence,
    wellsequence::{WellSequence, WellSequenceUseFlags},
};
pub extern "C" fn well_get_item_rng(proc: &mut WellSequence, _method_info: OptionalMethod) {
    let item_flag = DVCFlags::RandomEventItems.get_value();
    let use_flag = WellSequence::get_use_flag();
    let item_pool = GameData::get_item_pool();
    if item_pool.has_well && !item_flag  {
        WellSequence::get_item(proc);
        return;
    }
    if use_flag == WellSequenceUseFlags::ItemReturn {
        let pool = GameData::get_item_pool();
        let level = WellSequence::get_exchange_level();
        let seed = WellSequence::get_seed();
        let rng = Random::new(seed as u32);
        let map_completed = get_continious_total_map_complete_count();
        let list = List::<ItemData>::with_capacity(3).unwrap();
        if !item_pool.has_well || level == 0 || map_completed  < 6 {
            list.add(ItemData::get_mut("IID_スキルの書・守").unwrap());
            list.add(ItemData::get_mut("IID_スキルの書・破").unwrap());
        }
        else if item_flag && DVCVariables::random_enabled() {
            let sum = 1500*(level + 2) + rng.get_value(10)*500*level;
            let count = 2*(level + 1 + rng.get_value(5) );
            let mut total_price = 0;
            let price_cap = 1000 + 750 * level;
            let mut n_items = 0;
            while total_price < sum && n_items < count  {
                let iid = pool.random_item(4, false);
                if let Some(item) = ItemData::get_mut(iid).filter(|x| x.price < price_cap && x.price > 100 && x.flag.value & 131 == 0){
                    total_price += item.price;
                    list.add(item);
                    n_items += 1;
                }
            }
            let is_continuous = DVCVariables::Continuous.get_value() != 0 || DVCFlags::RandomSP.get_value();
            if is_continuous || level > 0 { list.add(ItemData::get_mut("IID_スキルの書・守").unwrap()); }
            if is_continuous || ( level >= 2 && rng.get_value(2) != 0 ) { list.add(ItemData::get_mut("IID_スキルの書・破").unwrap()); }
            if level > 3 && ( rng.get_value(10) < level || is_continuous) { list.add(ItemData::get_mut("IID_スキルの書・離").unwrap()); }
        }
        else {
            list.add(ItemData::get_mut("IID_スキルの書・守").unwrap());
            list.add(ItemData::get_mut("IID_スキルの書・破").unwrap());
        }
        CommonRewardSequence::create_bind_for_well(proc, list, "MID_Hub_Well_ItemExchange_Get".into());
        WellSequence::set_seed(0);
        WellSequence::set_use_flag(WellSequenceUseFlags::NotUse);
    }
}
