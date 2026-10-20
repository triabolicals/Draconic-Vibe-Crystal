use std::cmp::max;
use engage::{
    List_1Ext,
    app::{
        basicmenu::*, gamesound::*,
        titlebar::*, itemdata::*, unititem::*, itemevolvedata::*, unit::*,
        refineshoprefinetargetbasemenuitem::*, refineshoprefinetargetevolvemenuitem::*,
        refineshoprefinetargetmenu::*, refineshoprefinetargetroot::*, transporter::*,
        IUnitItemListMethods,
        UnitUtil, GameUserData,
        ISingletonClass_1Methods, IGameUserDataMethods,
        ProcVoidMethod, GameMessage, FunctionCallProc,
        ShopBuyYesNoDialog_YesEventHandler,
        ShopBuyYesNoDialog, ProcInst,
    },
    combat::Character,
    system::IDelegate,
};
use unity::{Cast, Il2CppString, IlNull, OptionalMethod};
use crate::{gamedata::GameData, DVCFlags};
pub fn edit_refine_evolve_items(this: ProcInst) {
    if DVCFlags::EvolveItems.get_value(){
        if let Some(menu) = this.try_cast::<RefineShopRefineTargetMenu>() {
            let items = menu.m_full_menu_item_list();
            items.iter().flat_map(|v| v.try_cast::<RefineShopRefineTargetEvolveMenuItem>())
                .for_each(|i|{
                    i.set_m_is_name_visible(true);
                    let new_item = GameData::random_refine(i.m_base_unit_item().m_item(), i.m_evolve_index());
                    i.m_evolved_unit_item().set_m_item(new_item);
                    if !i.m_decide_event_handler().is_null() {
                        i.m_decide_event_handler()
                            .set_method_ptr(unsafe { std::mem::transmute(on_decide_menu_item_evolve_method_info().method_ptr)});
                    }
                });
        }
    }
}

#[unity::callback]
pub fn on_decide_menu_item_evolve(this: RefineShopRefineTargetRoot, evolve_unit_item: UnitItem, evolve_index: i32, _: OptionalMethod) {
    let base_item = this.m_base_unit_item();
    if !base_item.is_null() && base_item.m_index() != 0 {
        let new_item = GameData::random_refine(base_item.m_item(), evolve_index);
        let evolve = base_item.get_evolve_data_list().get(evolve_index);
        let new = ItemEvolveData::new();
        this.set_m_evolve_index(evolve_index);
        let menu = this.m_refine_shop_refine_target_menu();
        new.set_price(evolve.get_price());
        new.set_iid(new_item.get_iid());
        new.set_silver(evolve.get_silver());
        new.set_iron(evolve.get_iron());
        new.set_steel(evolve.get_steel());
        let yes = ShopBuyYesNoDialog_YesEventHandler::new(this.into(), on_evolve_method_info().into());
        ShopBuyYesNoDialog::create_bind_5(menu, base_item, evolve_unit_item, new, yes);
    }
}
fn evolve_item(unit: Unit, item_index: i32, evolve_index: i32) -> UnitItem{
    let mut unit_item = UnitItem::null();
    if unit.is_null() { // Convoy
        if item_index < 1000 { unit_item = Transporter::get(item_index).m_unit_item(); }
    }
    else if item_index < 9 { unit_item = unit.m_item_list().get_item(item_index); }
    if !unit_item.is_null() && unit_item.m_index() != 0 {
        let data = unit_item.get_evolve_data_list().get(evolve_index);
        let new_item = GameData::random_refine(unit_item.m_item(), evolve_index);
        let evolved = UnitUtil::evolve_weapon(unit, item_index, new_item.get_iid());
        let user_data = GameUserData::get_instance();
        user_data.set_refine_iron(max(user_data.get_refine_iron() - data.get_iron() as i32, 0));
        user_data.set_refine_steel(max(user_data.get_refine_steel() - data.get_steel() as i32, 0));
        user_data.set_refine_silver(max(user_data.get_refine_silver() - data.get_silver() as i32, 0));
        user_data.set_gold(max(user_data.get_gold() - new_item.get_price(), 0));
        evolved
    }
    else { UnitItem::null() }
}
#[unity::callback]
pub fn on_evolve(this: RefineShopRefineTargetRoot, _: OptionalMethod) {
    let evolve = evolve_item(this.m_unit(), this.m_item_index(), this.m_evolve_index());
    TitleBar::get_instance().update_footer_values();
    GameSound::post_event("Shop_BuySell", Character::null());
    GameSound::stop_all_voice(GameSound_FadeSpeedType::immediate());
    GameSound::talk_voice("V_Shop3_Thanks");
    this.set_m_evolve_index(0);
    if evolve.is_null() {
        this.close();
        return;
    }
    if !this.m_unit().is_null() {
        if evolve.get_is_equipped() && evolve.is_weapon() { this.m_unit().item_equip_3(evolve); }
        else { this.m_unit().item_take_off(this.m_item_index()); }
    }
    if !this.m_return_event_handler().is_null() { this.m_return_event_handler().invoke(evolve); }
    let call_back = ProcVoidMethod::new(this.into(), RefineShopRefineTargetRoot::close_method_info().into());
    let proc = FunctionCallProc::create_bind(this.m_refine_shop_refine_target_menu(), call_back);
    GameMessage::create_unit_item_get(proc, evolve, Il2CppString::null(), 1);
}