use std::cmp::max;
use engage::{
    app::{
        itemdata::*, transporter::*, random_2::*,
        itemshopbuymenuitem::*, weaponshopbuymenuitem::*,
        weaponshopbuymenu::*, itemshopbuymenu::*,
        weaponshopbuyroot::*, itemshopbuyroot::*,
        shopbuyyesnodialog::*, gameuserdata::*, 
        titlebar::*, basicmenuitem::*, shopcontent::*, shopcore::*,
        ItemRefineData, BasicMenu_Result, GameMessage, Unit, GameSound, GameColor,
        IStructBase, IStructData_1Methods, IBitField32,
        ISingletonScriptableObject_1Methods, IGameColor,
    },
    prelude::List_1, GameVariableManager, List_1Ext, Mess,
    system::{collections::generic::IList_1Methods, IDelegate, Object}
};
use engage::app::GameParam;
use outfit_core::Randomizer;
use unity::{Array, Cast, FromIlInstance, IlNull, OptionalMethod};
use crate::{DVCFlags, DVCVariables, gamedata::RandomizedGameData};
mod evolve;
pub use evolve::edit_refine_evolve_items;

const ITEM_COUNT: usize = 5;

#[unity::hook("App", "ItemRefineData", "TryGetFromItem")]
pub fn item_refine_data_try_get(item: ItemData, method_info: OptionalMethod) -> List_1<ItemRefineData>{
    if DVCVariables::get_sequence() != 0 || item.is_null() || !DVCFlags::RefineItem.get_value()  { call_original!(item, method_info) }
    else {
        if let Some(item) = RandomizedGameData::get_read().refine.get(item.index() as usize).map(|v| ItemData::try_get_from_hash(*v)){
            return call_original!(item, method_info)
        }
        call_original!(item, method_info)
    }
}
pub fn update_added_shop_items(map_complete: bool) {
    let rng = Random_2::get_system();
    let (max_price, rank) =
        match DVCVariables::chapter_number_complete(false) {
            0..5 => { (1500, 2) }
            5..10 => { (2000, 3) }
            10..15 => { (3000, 3) }
            15..20 => { (3500, 4) }
            20..23 => { (6000, 5) }
            _ => { (10000, 7) }
        };
    let mut possible_items: Vec<_> =
        ItemData::get_list().iter()
            .filter(|p|{
                p.get_use_type().value < 31 &&
                p.get_price() > 100 && p.get_price() <= max_price && p.get_flag().m_value() & 130 == 0 && !p.is_weapon() && p.get_kind().value != 17 && p.get_kind().value != 18 && can_buy(*p)
            })
            .map(|h| h.hash())
            .collect();

    let mut possible_weapons: Vec<_> =
        ItemData::get_list().iter()
            .filter(|p| p.get_price() > 100 && p.get_price() <= max_price && p.is_weapon() && p.get_flag().m_value() & 130 == 0 && p.m_weapon_level().value < rank )
            .map(|h| h.hash()).collect();

    for x in 0..2*ITEM_COUNT {
        let key = if x < ITEM_COUNT { format!("G_DVC_I{}", x) } else { format!("G_DVC_W{}", x-ITEM_COUNT) };
        let hash = GameVariableManager::get_number(key.as_str());
        let list = if x < ITEM_COUNT { &mut possible_items } else { &mut possible_weapons };
        if (!map_complete && ItemData::try_get_from_hash(hash).is_null()) || map_complete {
            let mut hash = -1;
            while hash == -1 && !list.is_empty(){
                if let Some(item) = list.get_remove(rng).map(|h| ItemData::try_get_from_hash(h)){
                    let key = format!("G_在庫_{}", item.get_prefixless_iid());
                    if GameVariableManager::is_exist(key.as_str()) {
                        if GameVariableManager::get_number(key.as_str()) as u32 > 0 { continue; }
                    }
                    hash = item.hash();
                }
            }
            GameVariableManager::set_number(key, hash);
        }
    }
}

#[unity::inject(namespace="DVC", name="DVCShopItemMenuItem", parent=ItemShopBuyMenuItem)]
pub struct DVCShopItemMenuItem {}

#[unity::injected_methods]
impl DVCShopItemMenuItem {
    #[override_virtual(name="OnBuildMenuItemContent")]
    pub fn on_build_menu_item_content(self) { self.on_build(); }

    #[override_virtual(name="OnBuild")]
    pub fn on_build(self) {
        self.set_initial_color();
        let attribute = self.get_attribute().value;
        self.set_initial_color();
        if attribute & 2 == 0 {
            let green = get_green_color();
            self.set_cursor_color(green);
            self.set_m_active_text_color(green);
            self.set_m_active_text_color2(green);
            self.set_m_inactive_text_color(green);
            self.set_m_inactive_text_color2(green);
            IItemShopBuyMenuItemMethods::set_text_color(self, green, green, green, green);
        }
    }
}
#[unity::inject(namespace="DVC", name="DVCShopWeaponMenuItem", parent=WeaponShopBuyMenuItem)]
pub struct DVCShopWeaponMenuItem {}
#[unity::injected_methods]
impl DVCShopWeaponMenuItem {
    #[override_virtual(name="OnBuildMenuItemContent")]
    pub fn on_build_menu_item_content(self) { self.on_build(); }
    #[override_virtual(name="OnBuild")]
    pub fn on_build(self) {
        let attribute = self.get_attribute().value;
        self.set_initial_color();
        if attribute & 2 == 0 {
            let green = get_green_color();
            self.set_cursor_color(green);
            self.set_m_active_text_color(green);
            self.set_m_active_text_color2(green);
            self.set_m_inactive_text_color(green);
            self.set_m_inactive_text_color2(green);
            IWeaponShopBuyMenuItemMethods::set_text_color(self, green, green, green, green);
        }
    }
}
#[unity::hook("App", "ItemShopBuyMenu", "CreateMenuItem")]
pub fn item_buy_item_create_menu_item(
    content_array: Array<ShopContent>,
    unit: Unit,
    select: ItemShopBuyMenu_SelectEventHandler,
    decided: ItemShopBuyMenu_DecideEventHandler,
    change_prev: ItemShopBuyMenu_ChangeUnitToPrevEventHandler,
    change_next: ItemShopBuyMenu_ChangeUnitToNextEventHandler,
    flea_market: bool,
    method_info: OptionalMethod) -> List_1<ItemShopBuyMenuItem>
{
    let list = call_original!(content_array, unit, select, decided, change_prev, change_next, flea_market, method_info);
    if !flea_market && DVCFlags::AddedShopItems.get_value() {
        let target = decided.m_target();
        let decide_handler = ItemShopBuyMenu_DecideEventHandler::new(Object::null(), random_item_shop_buy_decide_method_info().into());
        decide_handler.set_m_target(target);
        for x in 0..ITEM_COUNT {
            if let Some(item) = crate::utils::to_opt(ItemData::try_get_from_hash(GameVariableManager::get_number(format!("G_DVC_I{}", x)))) {
                let c = DVCShopItemMenuItem::instantiate().unwrap();
                let shop_content = ShopContent::new();
                shop_content.set_m_stock_num(1);
                shop_content.set_m_new_arrival(false);
                shop_content.set_m_iid(item.get_iid());
                IItemShopBuyMenuItemMethods::ctor(c, shop_content, unit, select, decide_handler, change_prev, change_next, flea_market);
                list.add(c.into());
            }
        }
    }
    list
}
#[unity::hook("App", "WeaponShopBuyMenu", "CreateMenuItem")]
pub fn weapon_buy_item_create_menu_item(
    content_array: Array<ShopContent>,
    kind: i32,
    unit: Unit,
    select: WeaponShopBuyMenu_SelectEventHandler,
    decided: WeaponShopBuyMenu_DecideEventHandler,
    change_prev:  WeaponShopBuyMenu_ChangeUnitToPrevEventHandler,
    change_next:  WeaponShopBuyMenu_ChangeUnitToNextEventHandler,
    method_info: OptionalMethod
) -> List_1<WeaponShopBuyMenuItem>{
    let list = call_original!(content_array, kind, unit, select, decided, change_prev, change_next, method_info);
    if DVCFlags::AddedShopItems.get_value() {
        let target = decided.m_target();
        let new_decided = WeaponShopBuyMenu_DecideEventHandler::new(Object::null(), random_weapon_shop_buy_decide_method_info().into());
        new_decided.set_m_target(target);
        for x in 0..ITEM_COUNT {
            if let Some(item) = crate::utils::to_opt(ItemData::try_get_from_hash(GameVariableManager::get_number(format!("G_DVC_W{}", x)))){
                if item.get_kind().value == kind || kind == 0 {
                    let c = DVCShopWeaponMenuItem::instantiate().unwrap();
                    let shop_content = ShopContent::new();
                    shop_content.set_m_stock_num(1);
                    shop_content.set_m_new_arrival(false);
                    shop_content.set_m_iid(item.get_iid());
                    IWeaponShopBuyMenuItemMethods::ctor(c, shop_content, unit, select, new_decided, change_prev, change_next);
                    list.add(c.into());
                }
            }
        }
    }
    list
}
#[unity::callback]
pub fn random_item_shop_buy_decide(root: ItemShopBuyRoot, item: ItemData, _: OptionalMethod) -> bool {
    if item.is_inventory() && ShopCore::is_inventory_max(item) {
        GameMessage::create_key_wait(root.m_item_shop_buy_menu(), Mess::get("MID_MENU_SHOP_BUY_NOTICE_FULL"));
        false
    }
    else {
        let yes_handler = ShopBuyYesNoDialog_YesEventHandler::new(root.into(), item_shop_dialog_on_yes_method_info().into());
        let yes_no_dialog = ShopBuyYesNoDialog::create_bind_2(root.m_item_shop_buy_menu(), item, 1, true, yes_handler);
        root.set_m_item_data(item);
        root.set_m_yes_no_dialog(yes_no_dialog);
        true
    }
}
#[unity::callback]
pub fn random_weapon_shop_buy_decide(root: WeaponShopBuyRoot, item: ItemData, _: OptionalMethod) -> bool {
    if item.is_inventory() && ShopCore::is_inventory_max(item) {
        GameMessage::create_key_wait(root.m_weapon_shop_buy_menu(), Mess::get("MID_MENU_SHOP_BUY_NOTICE_FULL"));
        false
    }
    else {
        let yes_handler = ShopBuyYesNoDialog_YesEventHandler::new(root.into(), weapon_shop_dialog_on_yes_method_info().into());
        let yes_no_dialog = ShopBuyYesNoDialog::create_bind_2(root.m_weapon_shop_buy_menu(), item, 1, true, yes_handler);
        root.set_m_item_data(item);
        root.set_m_yes_no_dialog(yes_no_dialog);
        true
    }
}

#[unity::callback]
pub fn item_shop_dialog_on_yes(this: ItemShopBuyRoot, _: OptionalMethod) -> BasicMenu_Result {
    if let Some(item) = crate::utils::to_opt(this.m_item_data()){
        let hash = item.hash();
        for x in 0..ITEM_COUNT {
            let item_key = format!("G_DVC_I{}", x);
            if GameVariableManager::get_number(item_key.as_str()) == hash {
                GameVariableManager::set_number(item_key.as_str(), -1);
                break;
            }
        }
        buy_random_item(item);
        this.m_item_shop_buy_menu().rebuild_menu_2(true, true);
    }
    BasicMenu_Result::close_decide()
}
#[unity::callback]
pub fn weapon_shop_dialog_on_yes(this: WeaponShopBuyRoot, _: OptionalMethod) -> BasicMenu_Result {
    if let Some(item) = crate::utils::to_opt(this.m_item_data()){
        let hash = item.hash();
        for x in 0..ITEM_COUNT {
            let item_key = format!("G_DVC_W{}", x);
            if GameVariableManager::get_number(item_key.as_str()) == hash {
                GameVariableManager::set_number(item_key.as_str(), -1);
                break;
            }
        }
        buy_random_item(item);
        this.m_weapon_shop_buy_menu().rebuild_menu_2(true, true);
    }
    BasicMenu_Result::close_decide()
}

pub fn can_buy(item_data: ItemData) -> bool {
    let count = item_data.get_inventory();
    if item_data.get_flag().m_value() & 144 != 0 { return false; }
    let user_data = DVCVariables::user_data();
    match item_data.get_kind().value {
        18 => { user_data.get_gold() + item_data.get_price() < 9999999 }
        17 => { user_data.get_piece_of_bond() + item_data.get_price() < 9999999 }
        14..18 => { count < 9999 }
        _ => {
            match item_data.get_use_type().value {
                40|41 => DVCVariables::has_dlc(),
                36..40 => false,
                _ => { if !item_data.is_inventory() { Transporter::can_add() } else { count < 999 } }
            }
        }
    }
}
pub fn buy_random_item(item_data: ItemData) {
    let count = item_data.get_inventory();
    let user_data = DVCVariables::user_data();
    let mut price = item_data.get_price();
    if ShopCore::is_price_down() {
        let factor = 1.0 - GameParam::get_float("シルバーカード割引率");
        price = ( price as f32 * factor) as i32;
    }
    let gold = max(user_data.get_gold() - price, 0);
    match item_data.get_kind().value {
        18 => { user_data.set_gold(user_data.get_gold() + item_data.get_price()); }
        17 => { user_data.set_piece_of_bond(user_data.get_piece_of_bond() + item_data.get_price()); },
        16 => if count < 9998 { user_data.set_refine_silver(user_data.get_refine_silver() + 1); },
        15 => if count < 9994 { user_data.set_refine_steel(user_data.get_refine_steel() + 5); },
        14 => if count < 9989 { user_data.set_refine_iron(user_data.get_refine_iron() + 10); },
        _ => {
            if item_data.is_inventory() { item_data.add_inventory(1); }
            else if Transporter::can_add() { Transporter::add_2(item_data); }
        }
    }
    user_data.set_gold( gold );
    GameSound::post_event("Shop_BuySell", engage::combat::Character::null());
    TitleBar::get_instance().update_footer_values();
}
fn get_green_color() -> engage::unity_engine::Color { GameColor::get_instance().緑文字コマンド() }