use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    random::*,
    gamedata::{*, item::*},
};
pub static mut SHOP_SET: bool = false;
use super::CONFIG;

pub struct ShopRandomizer {
    pub pool: Vec<RandomItem>,
    pub size: i32,
}
pub struct RandomItem {
    pub index: i32,
    pub is_inf: bool,
    pub used: bool,
}
impl RandomItem {
    pub fn new(item_index: i32, inf: bool, in_used: bool) -> Self {
        Self {
            index: item_index,
            is_inf: inf,
            used: in_used,
        }
    }
}
impl ShopRandomizer {
    pub fn new() -> Self {  Self { pool: Vec::new(), size: 0, } }
    pub fn reset(&mut self) {
        for x in &mut self.pool {
            if !x.is_inf { x.used = false; }
        }
    }
    pub fn flag_item(&mut self, iid: &Il2CppString, is_inf: bool) {
        let index = ItemData::get_index(iid);
        let pos = self.pool.iter_mut().find(|x| x.index == index );
        if pos.is_some() {
            let item = pos.unwrap();
            item.is_inf = is_inf;
            item.used = true;
        }
        else {
            self.pool.push(RandomItem::new(index, is_inf, true));
            self.size += 1;
        }
    
    }
    pub fn add_list(&mut self, item: &ItemData){
        if self.pool.iter().find(|&x| x.index == item.parent.index).is_none() {
            self.pool.push(RandomItem::new(item.parent.index, false, false));
            self.size += 1;
        }
    }
    pub fn get_random(&mut self, rng: &Random) -> &'static ItemData {
        let mut index = rng.get_value(self.size);
        while self.pool[index as usize].used || self.pool[index as usize].is_inf {
            index = rng.get_value(self.size);
        }
        self.pool[index as usize].used = true;
        return ItemData::try_index_get( self.pool[index as usize].index).unwrap();
    }
}

trait ShopData: Il2CppClassData + Sized {
    fn register() {
        println!("{} Register",  Self::class().get_name());
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
        println!("{} ctor",  Self::class().get_name());
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
}

pub fn randomize_shop_data() {
    let seed;
    if GameVariableManager::exist("G_Random_Shop_Items") {
        seed = GameVariableManager::get_number("G_Random_Shop_Items") as u32;;
    }
    else if CONFIG.lock().unwrap().random_shop_items {
        seed = crate::utils::get_random_number_for_seed();
        GameVariableManager::make_entry("G_Random_Shop_Items", seed as i32);
    }
    else {  return;  }
    unsafe {
        if SHOP_SET { return;}
        else { SHOP_SET = true;  }
    }
    let rng = Random::instantiate().unwrap();
    rng.ctor(seed);
    println!("Randomizing Shop Data");
    let mut ishop_rzr = ShopRandomizer::new();
    let mut wshop_rzr = ShopRandomizer::new();
    let mut fm_rzr = ShopRandomizer::new();

    let item_list = ItemData::get_list().unwrap();
    for x in 0..item_list.len() {
        let item = &item_list[x];
        if item.kind == 13 {
            fm_rzr.add_list(item);
            ishop_rzr.add_list(item);
            continue; 
        }
        if item.price == 100 { continue; }
        if item.is_weapon() {
            wshop_rzr.add_list(item);
            continue;
        }
        if item.usetype >= 32 && item.usetype <= 39 { 
            fm_rzr.add_list(item);
            continue; 
        } 
        if item.usetype == 0 {
            if item.kind >= 14 || item.kind <= 16 { fm_rzr.add_list(item); }   
            continue;
        }
        ishop_rzr.add_list(item);
    }
    let shop_list = ItemShopData::get_list_mut().unwrap();
    let length = shop_list.len();

// ItemShop Random Item Additions
    // Getting inf stock items first to remove them from the list
    for x in 0..length  {
        let slist = &mut shop_list[x];
        for y in 0..slist.len() {
            let is_inf = slist[y].stock == -1; 
            ishop_rzr.flag_item( slist[y].iid, is_inf); 
        }
    }
    // Adding Random items and removing items already on the list 
    for x in 0..length  {
        let slist = &mut shop_list[x];
        ishop_rzr.reset();
        for y in 0..slist.len() {
            let is_inf = slist[y].stock == -1; 
            ishop_rzr.flag_item( slist[y].iid, is_inf);   
        }
        let num_new_items = rng.get_value(5) + 5;
        for _ in 0..num_new_items {
            let new_data = ItemShopData::instantiate().unwrap();
            new_data.ctor();
            let item = ishop_rzr.get_random(rng);
            new_data.iid = item.iid;
            match item.usetype {
                5|6|7|15|18|21|27 => { new_data.stock = 1; }
                8|9|11|23|24 => { new_data.stock = 1 + rng.get_value(3); }
                _ => { new_data.stock = 1 + rng.get_value(5); }
            }
            slist.add(new_data);
        }
    }
// WeaponShop
    let shop_list = WeaponShopData::get_list_mut().unwrap();
    let length = shop_list.len();
    for x in 0..length  {
        let slist = &mut shop_list[x];
        for y in 0..slist.len() {
            let is_inf = slist[y].stock == -1; 
            wshop_rzr.flag_item( slist[y].iid, is_inf); 
        }
    }
    // Adding Random items and removing items already on the list 
    for x in 0..length  {
        let slist = &mut shop_list[x];
        wshop_rzr.reset();
        for y in 0..slist.len() {
            let is_inf = slist[y].stock == -1; 
            wshop_rzr.flag_item( slist[y].iid, is_inf);   
        }
        let num_new_items = rng.get_value(3) + 5;
        for _ in 0..num_new_items {
            let new_data = WeaponShopData::instantiate().unwrap();
            new_data.ctor();
            let item = wshop_rzr.get_random(rng);
            new_data.iid = item.iid;
            new_data.stock = 1;
            slist.add(new_data);
        }
    }
//FleaMarketData
    let shop_list = FleaMarketData::get_list_mut().unwrap();
    let length = shop_list.len();
    for x in 0..length  {
        let slist = &mut shop_list[x];
        for y in 0..slist.len() {
            let is_inf = slist[y].stock == -1; 
            fm_rzr.flag_item( slist[y].iid, is_inf); 
        }
    }
    // Adding Random items and removing items already on the list 
    for x in 0..length  {
        let slist = &mut shop_list[x];
        fm_rzr.reset();
        for y in 0..slist.len() {
            let is_inf = slist[y].stock == -1; 
            fm_rzr.flag_item( slist[y].iid, is_inf);   
        }
        let num_new_items = rng.get_value(5) + 5;
        for _ in 0..num_new_items {
            let new_data = FleaMarketData::instantiate().unwrap();
            new_data.ctor();
            let item = fm_rzr.get_random(rng);
            new_data.iid = item.iid;
            new_data.stock = 1 + rng.get_value(20);
            slist.add(new_data);
        }
    }
    FleaMarketData::register();
    WeaponShopData::register();
    ItemShopData::register();
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
        let selection = CONFIG.lock().unwrap().random_shop_items;
        if selection  { this.help_text = "Random items will be added to shops".into(); }
        else { this.help_text = "No random items will be added to shops.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_shop_items;
        if selection { this.command_text = "Random Shop Items".into(); }
        else { this.command_text = "Default Shop Item".into(); }
    }
}