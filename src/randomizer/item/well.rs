use super::*;
use engage::proc::ProcInst;
use engage::sequence::commonrewardsequence::CommonRewardSequence;
use engage::sequence::wellsequence::WellSequence;

pub extern "C" fn well_get_item_rng(proc: &mut ProcInst, method_info: OptionalMethod) {
    let item_flag = GameVariableManager::get_number(DVCVariables::ITEM_KEY);
    let use_flag = WellSequence::get_use_flag();
    if *HAS_REWARD.get().unwrap() && item_flag & 1 == 0 {
        unsafe { well_get_item(proc, method_info); }
        return;
    }
    if use_flag == 2 {

        let level = WellSequence::get_exchange_level();
        let seed = WellSequence::get_seed();
        println!("[Random Well Get Item] Lvl: {}", level);
        let rng = Random::new(seed as u32);
        let map_completed = get_continious_total_map_complete_count();
        let list_class = get_list_item_class();
        let list = il2cpp::instantiate_class::<List<ItemData>>(list_class).unwrap();
        list.items = Il2CppArray::new(25).unwrap();
        if !HAS_REWARD.get().unwrap() || level == 0 || map_completed  < 6 {
            list.add(ItemData::get_mut("IID_スキルの書・守").unwrap());
            list.add(ItemData::get_mut("IID_スキルの書・破").unwrap());
        }
        else if item_flag & 1 != 0 && DVCVariables::random_enabled() {
            let sum = 1500*(level + 2) + rng.get_value(10)*500*level;
            let count = 2*(level + 1 + rng.get_value(5) );
            let mut total_price = 0;
            let price_cap = 1000 + 750 * level;
            let mut n_items = 0;
            while total_price < sum && n_items < count  {
                let iid = random_item(1, false);
                if let Some(item) = ItemData::get_mut(iid).
                    filter(|x| x.price < price_cap && x.price > 100 && x.flag.value & 131 == 0) {
                    total_price += item.price;
                    list.add(item);
                    n_items += 1;
                }
            }
            let is_continuous = GameVariableManager::get_number(DVCVariables::CONTINUOUS) != 0
                || GameVariableManager::get_number(DVCVariables::SP_KEY) != 0;
            if is_continuous || level > 0 {
                list.add(ItemData::get_mut("IID_スキルの書・守").unwrap());
            }
            if is_continuous || ( level >= 2 && rng.get_value(2) != 0 ) {
                list.add(ItemData::get_mut("IID_スキルの書・破").unwrap());
            }
            if level > 3 && ( rng.get_value(10) < level || is_continuous) {
                list.add(ItemData::get_mut("IID_スキルの書・離").unwrap());
            }
        }
        else {
            list.add(ItemData::get_mut("IID_スキルの書・守").unwrap());
            list.add(ItemData::get_mut("IID_スキルの書・破").unwrap());
        }
        CommonRewardSequence::create_bind_for_well(proc, list, "MID_Hub_Well_ItemExchange_Get".into());
        WellSequence::set_seed(0);
        WellSequence::set_use_flag(0);
        unsafe { well_set_exchance_level(0, None); }
    }
}

#[unity::from_offset("App", "WellSequence", "set_ExchangeLevel")]
fn well_set_exchance_level(value: i32, method_info: OptionalMethod);

#[skyline::from_offset(0x0293cb60)]
fn well_get_item(this: &ProcInst, method_info: OptionalMethod);
