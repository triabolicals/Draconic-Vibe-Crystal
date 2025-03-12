use accessory::AccessoryData;
use unity::prelude::*;
use engage::{
    gamedata::{item::*, shop::{ShopData, *}, *}, gamevariable::*, hub::access::HubRandomSet, menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult}, random::*
};
use super::{RANDOM_ITEM_POOL, *};

pub struct ShopRandomizer {
    pub pool: Vec<RandomItem>,
}
pub struct RandomItem {
    pub index: i32,
    pub is_inf: bool,
    pub used: bool,
}
impl RandomItem {
    pub fn new(item_index: i32, inf: bool, in_used: bool) -> Self {
        Self { index: item_index, is_inf: inf, used: in_used,  }
    }
}
impl ShopRandomizer {
    pub fn new() -> Self {  Self { pool: Vec::new(), } }
    pub fn reset(&mut self) {
        self.pool.iter_mut()
            .for_each(|entry|{
                if !entry.is_inf { entry.used = false; }
                if let Some(item) = ItemData::try_index_get(entry.index) {
                    if item.usetype == 7 || item.usetype == 13 || item.usetype == 21  { entry.used = true; }
                }
            }
        );
    }
    pub fn flag_item(&mut self, iid: &Il2CppString, is_inf: bool) {
        let index = ItemData::get_index(iid);
        if index <= 2 { return; }
        if let Some(found) = self.pool.iter_mut().find(|x| x.index == index ) {
            found.is_inf = is_inf;
            found.used = true;
        }
        else { self.pool.push(RandomItem::new(index, is_inf, true)); }
    }
    pub fn add_list(&mut self, item: &ItemData){
        if !self.pool.iter().any(|x| x.index == item.parent.index) {
            self.pool.push(RandomItem::new(item.parent.index, false, false));
        }
    }
    pub fn get_random(&mut self, rng: &Random) -> Option<&'static ItemData> {
        let mut pool: Vec<_> = self.pool.iter_mut().filter(|item| ( !item.used && !item.is_inf ) ).collect();
        let size = pool.len();
        if size == 0 { return None; }
        if let Some(item) = pool.iter_mut().nth( rng.get_value(size as i32) as usize) {
            item.used = true;
            return ItemData::try_index_get( item.index);
        }
        return None;
    }
}

pub fn reset_shopdata(){
    WeaponShopData::unload();
    WeaponShopData::load();
    ItemShopData::unload();
    ItemShopData::load();
    FleaMarketData::unload();
    FleaMarketData::load();
    HubRandomSet::unload();
    HubRandomSet::load();
}

pub fn randomize_shop_data() {
    if !crate::utils::can_rand() || GameVariableManager::get_number(DVCVariables::SHOP_KEY) == 0  { return; }
    if super::super::RANDOMIZER_STATUS.read().unwrap().shop_randomized { return; }

    let rng = crate::utils::get_rng();
    println!("Randomizing Shop Data");
    let _ = super::super::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.shop_randomized = true);
    let mut ishop_rzr = ShopRandomizer::new();
    let mut wshop_rzr = ShopRandomizer::new();
    let mut fm_rzr = ShopRandomizer::new();
    ItemData::get_list().unwrap().iter()
        .for_each(|item|{
            if item.price != 100 {
                if item.is_weapon() { wshop_rzr.add_list(item); }
                else if item.kind == 13 {
                    fm_rzr.add_list(item);
                    ishop_rzr.add_list(item);
                }
                else if item.usetype >= 32 && item.usetype <= 39 { fm_rzr.add_list(item); }
                else if item.usetype == 0 {
                    if item.kind >= 14 && item.kind <= 16 { 
                        fm_rzr.add_list(item);
                        ishop_rzr.add_list(item);
                    } 
                    else if item.kind >= 17 { fm_rzr.add_list(item); }
                    else { ishop_rzr.add_list(item); }
                }
                else { ishop_rzr.add_list(item); }
            }
        }
    );
// ItemShop Random Item Additions
    // Getting inf stock items first to remove them from the list
    ItemShopData::get_list_mut().unwrap().iter().for_each(|shop|shop.iter().for_each(|item|ishop_rzr.flag_item(item.iid, item.stock == -1)));
    ItemShopData::get_list_mut().unwrap().iter_mut()
        .for_each(|shop|{
            ishop_rzr.reset();
            shop.iter().as_ref().into_iter().for_each(|item|ishop_rzr.flag_item(item.iid, item.stock == -1));
            let num_new_items = rng.get_value(5) + 5;
            for _ in 0..num_new_items {
                if let Some(item) = ishop_rzr.get_random(rng) {
                    let new_data = ItemShopData::instantiate().unwrap();
                    new_data.ctor();
                    new_data.iid = item.iid;
                    new_data.stock = match item.usetype {
                        5|6|7|15|18|21|27 => { 1 }
                        8|9|11|13|23|24 => { 3 + rng.get_value(3) }
                        _ => { 3 + rng.get_value(5) }
                    };
                    shop.add(new_data);
                }
            }
        }
    );
    WeaponShopData::get_list_mut().unwrap().iter().for_each(|shop|shop.iter().for_each(|item|wshop_rzr.flag_item(item.iid, item.stock == -1)));
    WeaponShopData::get_list_mut().unwrap().iter_mut()
        .for_each(|shop|{
            wshop_rzr.reset();
            shop.iter().as_ref().into_iter().for_each(|item|wshop_rzr.flag_item(item.iid, item.stock == -1));
            let num_new_items = rng.get_value(5) + 5;
            for _ in 0..num_new_items {
                if let Some(item) = wshop_rzr.get_random(rng) {
                    let new_data = WeaponShopData::instantiate().unwrap();
                    new_data.ctor();
                    new_data.iid = item.iid;
                    new_data.stock = 1 + rng.get_value(3);
                    shop.add(new_data);
                }
            }
        }
    );  
//FleaMarketData
    FleaMarketData::get_list_mut().unwrap().iter().for_each(|shop|shop.iter().for_each(|item|fm_rzr.flag_item(item.iid, item.stock == -1)));
    FleaMarketData::get_list_mut().unwrap().iter_mut()
        .for_each(|shop|{
            fm_rzr.reset();
            shop.iter().as_ref().into_iter().for_each(|item|fm_rzr.flag_item(item.iid, item.stock == -1));
            let num_new_items = rng.get_value(10) + 5;
            for _ in 0..num_new_items {
                if let Some(item) = wshop_rzr.get_random(rng) {
                    let new_data = FleaMarketData::instantiate().unwrap();
                    new_data.ctor();
                    new_data.iid = item.iid;
                    new_data.stock = 3 + rng.get_value(5);
                    shop.add(new_data);
                }
            }
        }
    );
    FleaMarketData::register();
    WeaponShopData::register();
    ItemShopData::register();
    randomize_item_evolve();
}

pub struct RandomShopMod;
impl ConfigBasicMenuItemSwitchMethods for RandomShopMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_shop_items);
        if CONFIG.lock().unwrap().random_shop_items != result {
            CONFIG.lock().unwrap().random_shop_items = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().random_shop_items { "Random items will be added to shops" }
            else {"No random items will be added to shops." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().random_shop_items { "Add Random" }
            else { "Default" }.into();
    }
}

#[unity::class("App", "ItemEvolveData")]
pub struct ItemEvolveData {
    pub parent: StructDataArrayFields,
    pub iid: &'static Il2CppString,
    pub iron: u16,
    pub steel: u16,
    pub silver: u16,
    pub price: u16,
    pub refine_level: u8,
}
impl GamedataArray for ItemEvolveData  {}
impl ShopData for ItemEvolveData {}


pub fn randomize_item_evolve() {
    if !crate::utils::can_rand() || !GameVariableManager::get_bool(DVCVariables::SHOP_KEY)  { return; }
    let rng = crate::utils::get_rng();
    let item_pool_size = RANDOM_ITEM_POOL.lock().unwrap().len();
    ItemEvolveData::get_list_mut().unwrap().iter_mut()
        .for_each(|list|{
            let new_evolve = ItemEvolveData::instantiate().unwrap();
            new_evolve.ctor();
            new_evolve.iron = 100;
            new_evolve.steel = 10;
            new_evolve.silver = 1;
            new_evolve.price = 5000;
            loop {
                let index = rng.get_value(item_pool_size as i32);
                let item_index =  RANDOM_ITEM_POOL.lock().unwrap()[index as usize];
                if let Some(item) = ItemData::try_get_hash(item_index) {
                    if item.price < 2000 {  continue;  }
                    match item.usetype {
                        1|5|6|8|9|11 => { },
                        7 => {  new_evolve.price = 10000; },
                        23|24 => {  new_evolve.price = 1000; },
                        21 => { new_evolve.price = 5000;  },
                        _ => { continue; },
                    }
                    new_evolve.iid = item.iid;
                    break;
                }
            }
            list.add(new_evolve);
        }
    );
    println!("Randomizing Refine Items");
    unsafe { regist_evolve_flags(None); }
}
#[skyline::from_offset(0x0203dfd0)]
fn regist_evolve_flags(method_info: OptionalMethod);


pub fn add_personal_outfits() {
    let accessory = AccessoryData::get_list().unwrap();
    if GameVariableManager::get_bool(accessory[1].flag_name) { return; }
    else {
        for x in 1..42 { GameVariableManager::set_bool(accessory[x].flag_name, true);}
    }
}