use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    random::*,
    gamedata::{*, item::*},
};
pub static mut SHOP_SET: bool = false;
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
    pub fn get_random(&mut self, rng: &Random) -> &'static ItemData {
        let mut index = rng.get_value(self.pool.len() as i32 );
        while self.pool[index as usize].used || self.pool[index as usize].is_inf {
            index = rng.get_value(self.pool.len() as i32 );
        }
        self.pool[index as usize].used = true;
        return ItemData::try_index_get( self.pool[index as usize].index).unwrap();
    }
}

trait ShopData: Il2CppClassData + Sized {
    fn register() {
        let mut method = Self::class().get_methods().iter().find(|method| method.get_name() == Some(String::from("Regist")));
        if method.is_none() {
            method = Self::class()._1.parent.get_methods().iter().find(|method| method.get_name() == Some(String::from("Regist")));
        }
        let regist = unsafe {
            std::mem::transmute::<_, extern "C" fn(&MethodInfo) -> () >(
                method.unwrap().method_ptr,
            )
        };
        regist(method.unwrap());
    }
    fn ctor(&self) {
        let method = Self::class().get_methods().iter().find(|method| method.get_name() == Some(String::from(".ctor")));
        if method.is_none() { return; }
        let ctor = unsafe {
            std::mem::transmute::<_, extern "C" fn(&Self, &MethodInfo) -> () >(
                method.unwrap().method_ptr,
            )
        };
        ctor(self, method.unwrap());
    }
}

#[unity::class("App", "ItemShopData")]
pub struct ItemShopData {
    pub parent: StructDataArrayFields,
    pub iid: &'static Il2CppString,
    pub stock: i32, 
    pub attribute: i32,
}
impl GamedataArray for ItemShopData {}
impl ShopData for ItemShopData {}

#[unity::class("App", "WeaponShopData")]
pub struct WeaponShopData {
    pub parent: StructDataArrayFields,
    pub iid: &'static Il2CppString,
    pub stock: i32, 
    pub attribute: i32,
}
impl GamedataArray for WeaponShopData {}
impl ShopData for WeaponShopData {}

#[unity::class("App", "FleaMarketData")]
pub struct FleaMarketData {
    pub parent: StructDataArrayFields,
    pub iid: &'static Il2CppString,
    pub stock: i32, 
    pub attribute: i32,
}
#[unity::class("App", "AccessoryShopData")]
pub struct AccessoryShopData {
    pub parent: StructDataArrayFields,
    pub aid: &'static Il2CppString,
}
impl GamedataArray for AccessoryShopData {}
impl ShopData for AccessoryShopData {}
impl GamedataArray for FleaMarketData {}
impl ShopData for FleaMarketData {}

pub fn reset_shopdata(){
    WeaponShopData::unload();
    WeaponShopData::load();
    ItemShopData::unload();
    ItemShopData::load();
    FleaMarketData::unload();
    FleaMarketData::load();
    unsafe { SHOP_SET = false; }
    HubRandomSet::unload();
    HubRandomSet::load();
}

pub fn randomize_shop_data() {
    if !crate::utils::can_rand() || GameVariableManager::get_number("G_Random_Shop_Items") == 0  { return; }
    unsafe {
        if SHOP_SET { return;}
        else { SHOP_SET = true;  }
    }
    let rng = crate::utils::get_rng();
    println!("Randomizing Shop Data");
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
                let new_data = ItemShopData::instantiate().unwrap();
                new_data.ctor();
                let item = ishop_rzr.get_random(rng);
                new_data.iid = item.iid;
                new_data.stock = match item.usetype {
                    5|6|7|15|18|21|27 => { 1 }
                    8|9|11|13|23|24 => { 2 + rng.get_value(3) }
                    _ => { 3 + rng.get_value(5) }
                };
                shop.add(new_data);
            }
        }
    );
    WeaponShopData::get_list_mut().unwrap().iter().for_each(|shop|shop.iter().for_each(|item|wshop_rzr.flag_item(item.iid, item.stock == -1)));
    WeaponShopData::get_list_mut().unwrap().iter_mut()
        .for_each(|shop|{
            wshop_rzr.reset();
            shop.iter().as_ref().into_iter().for_each(|item|wshop_rzr.flag_item(item.iid, item.stock == -1));
            let num_new_items = rng.get_value(3) + 5;
            for _ in 0..num_new_items {
                let new_data = WeaponShopData::instantiate().unwrap();
                new_data.ctor();
                new_data.iid = wshop_rzr.get_random(rng).iid;
                new_data.stock = 1 + rng.get_value(3);
                shop.add(new_data);
            }
        }
    );  
//FleaMarketData
    FleaMarketData::get_list_mut().unwrap().iter().for_each(|shop|shop.iter().for_each(|item|fm_rzr.flag_item(item.iid, item.stock == -1)));
    FleaMarketData::get_list_mut().unwrap().iter_mut()
        .for_each(|shop|{
            fm_rzr.reset();
            shop.iter().as_ref().into_iter().for_each(|item|fm_rzr.flag_item(item.iid, item.stock == -1));
            let num_new_items = rng.get_value(5) + 5;
            for _ in 0..num_new_items {
                let new_data = FleaMarketData::instantiate().unwrap();
                new_data.ctor();
                new_data.iid = fm_rzr.get_random(rng).iid;
                new_data.stock = 5*(rng.get_value(10) + 1);
                shop.add(new_data);
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
        this.command_text = if CONFIG.lock().unwrap().random_shop_items { "Random Shop Items" }
                            else { "Standard Shop Items" }.into();
    }
}

#[unity::class("App", "HubRandomSet")]
pub struct HubRandomSet {
    pub parent: StructDataArrayFields,
    pub iid: &'static Il2CppString,
    pub rate: i32,
    pub count: i32,
}

impl GamedataArray for HubRandomSet {}
impl ShopData for HubRandomSet {}

pub fn randomize_hub_random_items(){
    if !crate::utils::can_rand() || GameVariableManager::get_number("G_Random_Item") & 2 == 0  { return; }
    println!("Randomizing Hub Items");
    HubRandomSet::get_list_mut().unwrap().iter_mut()
        .for_each(|set|{
            set.iter_mut()
                .for_each(|item|{
                    let iid = item.iid.to_string();
                    if item.rate != 0 && !iid.contains("の晶石") && iid.contains("IID_") {
                        item.iid = super::random_item(2, false);
                    }
                }
            );
        }
    );
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
#[skyline::from_offset(0x0203dfd0)]
fn regist_evolve_flags(method_info: OptionalMethod);

pub fn randomize_item_evolve() {
    if !crate::utils::can_rand() || !GameVariableManager::get_bool("G_Random_Shop_Items")  { return; }
    let rng = crate::utils::get_rng();
    let item_pool_size = RANDOM_ITEM_POOL.lock().unwrap().len();
    ItemEvolveData::get_list_mut().unwrap().iter_mut()
        .for_each(|list|{
            let new_evolve = ItemEvolveData::instantiate().unwrap();
            new_evolve.ctor();
            new_evolve.iron = 200;
            new_evolve.steel = 10;
            new_evolve.silver = 5;
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

pub struct RandomHubItemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomHubItemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().exploration_items, 0, 3, 1);
        if CONFIG.lock().unwrap().exploration_items != result {
            CONFIG.lock().unwrap().exploration_items = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().exploration_items {
            1 => {"Excludes gift items from exploration." },
            2 => { "Excludes food items from exploration."},
            3 => { "Excludes gift and food items from exploration."},
            _ => { "Exploration items includes both gift and food items."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().exploration_items {
            1 => {  "No Gifts" },
            2 => {  "No Food" },
            3 => {  "No Gift/Food"},
            _ => {  "Default"},
        }.into();
    }
}
// For In-game
pub struct RandomHubItemMod2;
impl ConfigBasicMenuItemSwitchMethods for RandomHubItemMod2{
    fn init_content(_this: &mut ConfigBasicMenuItem){ GameVariableManager::make_entry("G_HubItem", CONFIG.lock().unwrap().exploration_items); }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value =  GameVariableManager::get_number("G_HubItem");
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            GameVariableManager::set_number("G_HubItem", result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match GameVariableManager::get_number("G_HubItem") {
            1 => { "Excludes gift items from exploration." },
            2 => { "Excludes food items from exploration."},
            3 => { "Excludes gift and food items from exploration."},
            _ => { "Exploration items includes both gift and food items."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match  GameVariableManager::get_number("G_HubItem") {
            1 => {  "No Gifts" },
            2 => {  "No Food" },
            3 => {  "No Gift/Food"},
            _ => {  "Default"},
        }.into();
    }
}

pub extern "C" fn vibe_hub_items() -> &'static mut ConfigBasicMenuItem {  
    let hub_items = ConfigBasicMenuItem::new_switch::<RandomHubItemMod2>("Exploration Items");
    hub_items.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_hub_items as _);
    hub_items
}

pub fn add_personal_outfits() {
    if engage::gamedata::assettable::AssetTable::get_count() >= 4000 { return; }
    let hublist = AccessoryShopData::get_list_mut().unwrap();
    if hublist[0].len() > 40 {  AccessoryShopData::register(); }
    else {
        for x in 1..41 {
            let aid = format!("AID_{}私服", &crate::enums::PIDS[x][4..]);
            let acc = AccessoryShopData::instantiate().unwrap();
            acc.ctor();
            acc.aid = aid.into();
            hublist[0].add(acc);
        }
        AccessoryShopData::register();
    }
    println!("Outfits Registered");
}