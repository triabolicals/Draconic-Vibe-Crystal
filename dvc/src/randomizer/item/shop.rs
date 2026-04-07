use accessory::AccessoryData;
use unity::{prelude::*, engine::Color, il2cpp::object::Array, system::List};
use engage::{
    gameuserdata::GameUserData,
    gamedata::{item::*, shop::{ShopData, *}, *},
    gamevariable::*,
    random::*,
    dialog::shopyesno::{ShopBuyYesNoDialog, ShopBuyYesNoDialogYesHandler},
    menu::{
        BasicMenuResult,
        content::common::ShopContent,
        menu_item::MenuItemContent,
        menus::{shop::{ShopCore, weapon_buy::*, item_buy::*, weapon_buy::WeaponShopBuyMenuItem},},
        MenuItem,
    },
    gamemessage::GameMessage,
    gamesound::GameSound,
    titlebar::TitleBar,
    transporter::Transporter,
    unit::Unit,
};
use crate::config::DVCFlags;
use crate::{randomizer, DVCVariables};
use crate::ironman::vtable_edit;
use crate::randomizer::Randomizer;
use crate::utils::dlc_check;

pub fn random_shop_install() {
    if let Some(class)= Il2CppClass::from_name("App", "ItemShopBuyMenuItem").ok() {
        vtable_edit(class, "OnBuildMenuItemContent", shop_menu_item_on_build as _);
        vtable_edit(class, "OnBuild", shop_menu_item_on_build as _);
    }
    if let Some(class)= Il2CppClass::from_name("App", "WeaponShopBuyMenuItem").ok() {
        vtable_edit(class, "OnBuildMenuItemContent", weapon_but_item_on_build as _);
        vtable_edit(class, "OnBuild", weapon_but_item_on_build as _);
    }
    if let Some(class)= Il2CppClass::from_name("App", "SkillInheritanceMenuItemContent").ok() {
        vtable_edit(class, "Build", randomizer::emblem::menu::skill_inheritance_menu_item_content_build as _);
    }
}

#[unity::hook("App", "ItemShopBuyMenu", "CreateMenuItem")]
pub fn item_buy_item_create_menu_item(
    content_array: &Array<ShopContent>,
    unit: Option<&Unit>,
    select: Option<&ItemShopBuySelectHandler>,
    decided: Option<&ItemShopBuyDecideHandler>,
    change_prev: Option<&ItemShopBuyChangeUnitPrevHandler>,
    change_next: Option<&ItemShopBuyChangeUnitNextHandler>,
    flea_market: bool, method_info: OptionalMethod) -> &'static mut List<ItemShopBuyMenuItem>
{
    let list: &'static mut List<ItemShopBuyMenuItem> = call_original!(content_array, unit, select, decided, change_prev, change_next, flea_market, method_info);
    if !flea_market && DVCFlags::AddedShopItems.get_value() {
        let new_decided = ItemShopBuyDecideHandler::instantiate().unwrap();
        if let Some(decided) = decided {
            new_decided.method_ptr = random_item_shop_buy_decide as _;
            new_decided.target_obj = decided.target_obj;
            new_decided.method = decided.method;
        }
        for x in 0..10 {
            if let Some(item) = ItemData::try_get_hash(GameVariableManager::get_number(&format!("G_DVC_I{}", x))) {
                let iid = item.iid.to_string();
                if list.iter().find(|x| x.iid.str_contains(iid.as_str())).is_none() {
                    let c = ItemShopBuyMenuItem::instantiate().unwrap();
                    let shop_content = ShopContent::instantiate().unwrap();
                    shop_content.stock = 1;
                    shop_content.iid = iid.as_str().into();
                    shop_content.new_arrival = false;
                    c.ctor(shop_content, unit, select, Some(new_decided), change_prev, change_next, flea_market);
                    c.decide_event.as_mut().map(|m| m.method_ptr = random_item_shop_buy_decide as _ );
                    c.padding = item.parent.hash;
                    list.add(c);
                }
            }
        }
    }
    list
}
pub fn random_item_shop_buy_decide(this: &mut ItemShopBuyRoot, item: &'static ItemData, _optional_method: OptionalMethod) -> bool {
    if item.is_inventory() && ShopCore::is_inventory_max(item) {
        GameMessage::create_key_wait_mess(this.item_shop_buy_menu, "MID_MENU_SHOP_BUY_NOTICE_FULL");
        false
    }
    else {
        let yes_handler = ShopBuyYesNoDialogYesHandler::instantiate().unwrap();
        yes_handler.ctor(this, this.klass.get_method_from_name("OnYesToBuy", 0).unwrap());
        yes_handler.method_ptr = item_shop_dialog_on_yes as _;
        let yes_no_dialog = ShopBuyYesNoDialog::create_bind_item2(this.item_shop_buy_menu, item, 1, true,yes_handler);
        this.item_data = Some(item);
        this.yes_no_dialog = Some(yes_no_dialog);
        true
    }
}
pub fn random_weapon_shop_buy_decide(this: &mut WeaponShopBuyRoot, item: &'static ItemData, _optional_method: OptionalMethod) -> bool {
    if item.is_inventory() && ShopCore::is_inventory_max(item) {
        GameMessage::create_key_wait_mess(this.weapon_shop_buy_menu, "MID_MENU_SHOP_BUY_NOTICE_FULL");
        false
    }
    else {
        let yes_handler = ShopBuyYesNoDialogYesHandler::instantiate().unwrap();
        yes_handler.ctor(this, this.klass.get_method_from_name("OnYesToBuy", 0).unwrap());
        yes_handler.method_ptr = weapon_shop_on_yes as _;
        let yes_no_dialog = ShopBuyYesNoDialog::create_bind_item2(this.weapon_shop_buy_menu, item, 1, true,yes_handler);
        this.item_data = Some(item);
        this.yes_no_dialog = Some(yes_no_dialog);
        true
    }
}

#[unity::hook("App", "WeaponShopBuyMenu", "CreateMenuItem")]
pub fn weapon_buy_item_create_menu_item(
    content_array: &Array<ShopContent>,
    kind: i32,
    unit: Option<&Unit>,
    select: Option<&WeaponShopBuySelectHandler>,
    decided: Option<&WeaponShopBuyDecideHandler>,
    change_prev: Option<&WeaponShopBuyChangeUnitPrevHandler>,
    change_next: Option<&WeaponShopBuyChangeUnitNextHandler>,
    method_info: OptionalMethod
) -> &'static mut List<WeaponShopBuyMenuItem>{
    let list = call_original!(content_array, kind, unit, select, decided, change_prev, change_next, method_info);
    if DVCFlags::AddedShopItems.get_value() {
        let new_decided = WeaponShopBuyDecideHandler::instantiate().unwrap();
        if let Some(decided) = decided {
            new_decided.method_ptr = random_weapon_shop_buy_decide as _;
            new_decided.target_obj = decided.target_obj;
            new_decided.method = decided.method;
        }
        for x in 0..10 {
            if let Some(item) = ItemData::try_get_hash(GameVariableManager::get_number(&format!("G_DVC_W{}", x)))
                .filter(|i| i.kind == kind as u32 || kind == 0)
            {
                let iid = item.iid.to_string();
                if list.iter().find(|x| x.iid.str_contains(iid.as_str())).is_none() {
                    let c = WeaponShopBuyMenuItem::instantiate().unwrap();
                    let shop_content = ShopContent::instantiate().unwrap();
                    shop_content.stock = 1;
                    shop_content.iid = iid.as_str().into();
                    shop_content.new_arrival = false;
                    c.ctor(shop_content, unit, select, Some(new_decided), change_prev, change_next);
                    c.padding = item.parent.hash;
                    c.decide_event.as_mut().map(|m| m.method_ptr = random_weapon_shop_buy_decide as _);
                    list.add(c);
                }
            }
        }
    }
    list
}


pub fn update_added_shop_items(map_complete: bool) {
    if !DVCVariables::is_main_chapter_complete(4) { return; }
    let rng = Random::get_system();
    let (max_price, rank) =
        match crate::continuous::get_story_chapters_completed() {
            0..5 => { (1250, 2) }
            5..10 => { (1600, 3) }
            10..15 => { (2000, 3) }
            15..20 => { (2400, 4) }
            20..23 => { (5000, 4) }
            _ => { (10000, 6) }
        };
    let mut possible_items: Vec<_> =
        ItemData::get_list().unwrap().iter()
            .filter(|p| p.price > 100 && p.price <= max_price && p.flag.value & 130 == 0 && !p.is_weapon() && p.kind != 17 && p.kind != 18 && can_buy(p))
            .map(|h| h.parent.hash).collect();

    let mut possible_weapons: Vec<_> =
        ItemData::get_list().unwrap().iter()
            .filter(|p| p.price > 100 && p.price <= max_price && p.kind >= 1 && p.kind < 9 && p.is_weapon() && p.flag.value & 130 == 0 && p.get_weapon_level() < rank )
            .map(|h| h.parent.hash).collect();

    for x in 0..20 {
        let key = if x < 10 { format!("G_DVC_I{}", x) } else { format!("G_DVC_W{}", x-10) };
        let hash = GameVariableManager::get_number(&key);
        let list = if x < 10 { &mut possible_items } else { &mut possible_weapons };
        if (!map_complete && (ItemData::try_get_hash(hash).is_none() && hash != -1)) || map_complete {
            if list.len() > 2 {
                let mut count = 0;
                let mut hash = -1;
                while count < 50 {
                    if let Some(item) = list.get_remove(rng).and_then(|h| ItemData::try_get_hash(h)){
                        let iid = item.iid.to_string();
                        let iid_trimmed = iid.trim_start_matches("IID_");
                        let key = format!("G_在庫_{}", iid_trimmed);
                        if GameVariableManager::exist(&key) {
                            let i = GameVariableManager::get_number(key.as_str());
                            if i > 3  { count += 1; }
                        }
                        hash = item.parent.hash;
                        break;
                    }
                    else { break; }
                }
                GameVariableManager::set_number(&key, hash);
            }
        }
    }
}
pub fn item_shop_dialog_on_yes(this: &ItemShopBuyRoot, _optional_method: OptionalMethod) -> BasicMenuResult {
    if let Some(item) = this.item_data.as_ref() {
        let hash = item.parent.hash;
        for x in 0..10 {
            let item_key = format!("G_DVC_I{}", x);
            if GameVariableManager::get_number(&item_key) == hash {
                GameVariableManager::set_number(&item_key, -1);
                break;
            }
        }
        buy_random_item(item);
        this.item_shop_buy_menu.rebuild_menu(true, true);
    }
    BasicMenuResult::close_decide()
}
pub fn weapon_shop_on_yes(this: &WeaponShopBuyRoot, _optional_method: OptionalMethod) -> BasicMenuResult {
    if let Some(item) = this.item_data.as_ref() {
        let hash = item.parent.hash;
        for x in 0..10 {
            let weapon_key = format!("G_DVC_W{}", x);
            if GameVariableManager::get_number(&weapon_key) == hash {
                GameVariableManager::set_number(&weapon_key, -1);
                break;
            }
        }
        buy_random_item(item);
        this.weapon_shop_buy_menu.rebuild_menu(true, true);
    }
    BasicMenuResult::close_decide()
}
pub fn can_buy(item_data: &ItemData) -> bool {
    let count = item_data.get_inventory();
    if item_data.flag.value & 144 != 0 { return false; }
    match item_data.kind {
        18 => { GameUserData::get_gold() + item_data.price < 9999999 }
        17 => { GameUserData::get_instance().piece_of_bond + item_data.price < 9999999 }
        14..18 => { count < 9999 }
        _ => {
            match item_data.use_type {
                40|41 => dlc_check(),
                36..40 => false,
                _ => {
                    if !item_data.is_inventory() { Transporter::can_add() }
                    else { count < 999 }
                }
            }
        }
    }
}
pub fn buy_random_item(item_data: &ItemData) {
    let count = item_data.get_inventory();
    match item_data.kind {
        18 => {
            let gold = GameUserData::get_gold() + item_data.price;
            GameUserData::set_gold(gold);
        }
        17 => { GameUserData::add_bond(item_data.price); },
        16 => if count < 9998 { GameUserData::add_silver(1); },
        15 => if count < 9994 { GameUserData::add_steel(5); },
        14 => if count < 9989 { GameUserData::add_iron(10); },
        _ => {
            if item_data.is_inventory() { item_data.add_inventory(1); }
            else if Transporter::can_add() { Transporter::add_item(item_data); }
        }
    }
    GameUserData::set_gold(GameUserData::get_gold() - item_data.price);
    GameSound::post_event("Shop_BuySell", None);
    TitleBar::update_footer_values();
}

/// Same as WeaponShopBuyMenuItem in terms of fields it's menu item content.
pub fn shop_menu_item_on_build(this: &mut ItemShopBuyMenuItem, _optional_method: OptionalMethod){
    this.set_initial_color();
    if ItemData::try_get_hash(this.padding).is_some() {
        if let Some(game_color) = engage::game::GameColor::get(){
            let disabled = (this.attribute) & 2 != 0;
            if disabled {
                let mut a1 = Color::new();
                a1.r = game_color.unselectable_color.r;
                a1.b = game_color.unselectable_color.b;
                a1.g = 0.5;
                this.cursor_color = a1;
                this.active_text_color = a1;
                this.active_text_color2 = game_color.default_color;
            }
            else {
                this.inactive_text_color = game_color.green_text_command;
                this.cursor_color = game_color.green_text_command;
                this.active_text_color = game_color.green_text_command;
                this.active_text_color2 = game_color.green_text_command;
            }
            if let Some(content) = this.get_menu_item_content::<ItemShopBuyMenuItemContent>() {
                content.parent.text_base = this.inactive_text_color;
                content.text_base_color2 = this.inactive_text_color2;
                content.update_text_color();
            }
        }
    }
}
pub fn weapon_but_item_on_build(this: &mut WeaponShopBuyMenuItem, _optional_method: OptionalMethod){
    this.set_initial_color();
    if ItemData::try_get_hash(this.padding).is_some() {
        if let Some(game_color) = engage::game::GameColor::get(){
            let disabled = (this.attribute) & 2 != 0;
            if disabled {
                let mut a1 = Color::new();
                a1.r = game_color.unselectable_color.r;
                a1.b = game_color.unselectable_color.b;
                a1.g = 0.5;
                this.cursor_color = a1;
                this.active_text_color = a1;
                this.active_text_color2 = game_color.default_color;
            }
            else {
                this.cursor_color = game_color.green_text_command;
                this.inactive_text_color = game_color.green_text_command;
                this.active_text_color = game_color.green_text_command;
                this.active_text_color2 = game_color.green_text_command;
            }
            if let Some(content) = this.get_menu_item_content::<WeaponShopBuyMenuItemContent>() {
                if disabled {
                    content.parent.text_base = this.inactive_text_color;
                    content.text_base_color2 = this.inactive_text_color2;
                }
                else {
                    content.parent.text_base = game_color.default_color;
                    content.text_base_color2 = game_color.default_color;
                }
                content.update_text_color();
            }
        }
    }
}