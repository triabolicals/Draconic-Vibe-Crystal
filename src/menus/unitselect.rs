use engage::{app::{
    unit::*, force::*, unitinfo::*,
    procinst::*, gameui::*, hubplaytalkafter::*,
    shopunitselectmenuitem::*, shopunitselectmenu::*, shopunitselectroot::*, shopunitselectmenucontent::*,
    ShopUnitSelectStatus,
    JobData, Random_2, ResourceManager_2,
    UnitPool, BackgroundManager,
    GameMessage, GameSound, Mess, Proc,
    ProcBoolMethod, ProcVoidMethod, ProcDesc,
}, ProcBoolMethodExt, ProcVoidMethodExt, TitleBar, system::IDelegate, GameVariableManager};
use crate::dvc::PersonPool;
use super::*;

#[unity::inject(namespace="DVC", name="UnitRecruitProcInst", parent=ProcInst)]
pub struct UnitRecruitProc { pub shop_unit_select_root: ShopUnitSelectRoot, }
impl UnitRecruitProc {
    pub fn create_descs(self) -> Array<ProcDesc> {
        let descs = [
            Proc::call_2(ProcVoidMethod::from_fn(self, Self::load_shop_ui).unwrap()),
            Proc::wait_while_true_2(ProcBoolMethod::from_fn(self, Self::is_loading).unwrap()),
            Proc::call_2(ProcVoidMethod::from_fn(self, Self::create_menu_bind).unwrap()),
            Proc::call_2(ProcVoidMethod::from_fn(self, Self::unload).unwrap()),
            Proc::end()
        ];
        Array::from_slice(&descs).unwrap()
    }
    extern "C" fn load_shop_ui(_: UnitRecruitProc, _: OptionalMethod) { ShopUnitSelectRoot::load_prefab_async() }
    extern "C" fn is_loading(_: UnitRecruitProc, _: OptionalMethod) -> bool { ShopUnitSelectRoot::is_loading_prefab() }
    extern "C" fn unload(proc : UnitRecruitProc, _: OptionalMethod) {
        if !proc.shop_unit_select_root().is_null() {
            TitleBar::close_header();
            if BackgroundManager::is_bind() { BackgroundManager::unbind(); }
            ShopUnitSelectRoot::destroy(proc.shop_unit_select_root());
            Force::get(Force_Type::temporary()).transfer(Force_Type::empty(), true);
        }
        ShopUnitSelectRoot::unload_prefab()
    }
    extern "C" fn create_menu_bind(proc_inst: UnitRecruitProc, _: OptionalMethod) {
        let canvas = GameUI::get_canvas(GameUI_Priority::default());
        let transform = canvas.get_transform();
        let x = ResourceManager_2::instantiate_2("UI/Hub/Shop/Prefabs/ShopUnitSelectRoot", transform);
        if !x.is_null() {
            let root = x.get_component::<ShopUnitSelectRoot>();
            if !root.is_null() {
                proc_inst.set_shop_unit_select_root(root);
                TitleBar::open_header("Custom Recruitment", "Select a unit to recruit.", Il2CppString::null());
                if !BackgroundManager::is_bind() { BackgroundManager::bind_2(); }
                create_menu(root, proc_inst);
            }
        }
    }
}

#[unity::inject(namespace="DVC", name="UnitRecruitMenu", parent=HubPlayTalkAfter_TalkGiftMenu)]
pub struct UnitRecruitMenuItem {}

#[unity::injected_methods]
impl UnitRecruitMenuItem {
    #[override_virtual(name="BuildAttribute")]
    pub fn build_attribute(self) -> BasicMenuItem_Attribute {
        let mut count = 0;
        let playable_count = PersonPool::get().playable.len();
        let start = if GameVariableManager::get_number("G_拠点_裏武器イベント") >= 2 { 36 } else { 41 };
        if playable_count > 41 {
            for i in start..playable_count {
                if let Some(replacement) = PersonPool::get_replacement(i as i32, false, false){
                    if UnitPool::get_from_person(replacement, false).is_null() { count += 1; }
                }
            }
        }
        if count > 0 { BasicMenuItem_Attribute::enable() }
        else { BasicMenuItem_Attribute::hide() }
    }
    #[override_virtual(name="ACall")]
    pub fn a_call(self) -> BasicMenu_Result {
        let proc = UnitRecruitProc::instantiate().unwrap();
        proc.ctor();
        let descs = proc.create_descs();
        proc.create_bind(self.m_menu(), descs, "UnitRecruitProc");
        BasicMenu_Result::close_decide()
    }
    #[override_virtual(name="GetName")]
    pub fn get_name(self) -> Il2CppString { "Recruit Units".into() }
}
#[unity::inject(namespace="DVC", name="RecruitmentMenuItem", parent=ShopUnitSelectMenuItem)]
pub struct RecruitmentMenuItem {}

#[unity::injected_methods]
impl RecruitmentMenuItem {
    #[override_virtual(name="ACall")]
    pub fn a_call(self) -> BasicMenu_Result {
        let action = Action::new(self.into(), unit_recruit_yes_method_info().into());
        dialog::create_dialog_confirm(self.m_menu(), "Recruit this character?", action);
        BasicMenu_Result::se_cursor()
    }
    #[override_virtual(name="BCall")]
    pub fn b_call(self) -> BasicMenu_Result {
        if !self.m_decide_event_handler().is_null() {
            if let Some(root) = self.m_decide_event_handler().m_target().try_cast::<ShopUnitSelectRoot>() {
                unsafe { close_status(root.m_shop_unit_select_status(), None); }
            }
        }
        UnitInfo::set_unit(UnitInfo_Side::left(), Unit::null(), false, true, true, Action::null());
        BasicMenu_Result::close_cancel()
    }
    #[override_virtual(name="GetName")]
    pub fn get_name(self) -> Il2CppString { "Recruit Units".into() }
}


fn create_menu(this: ShopUnitSelectRoot, proc: impl Into<ProcInst>) {
    let game_object = this.m_shop_unit_select_menu_object();
    let menu_item_list = List_1::<RecruitmentMenuItem>::new();
    let menu_content = game_object.get_component::<ShopUnitSelectMenuContent>();
    let max = IShopUnitSelectMenuContentMethods::get_menu_item_content_max(menu_content);
    let select = ShopUnitSelectMenu_SelectEventHandler::new(this.into(), ShopUnitSelectRoot::on_select_menu_item_method_info().into());
    let menu = ShopUnitSelectMenu::instantiate().unwrap();
    let decided = ShopUnitSelectMenu_DecideEventHandler::new(this.into(), ShopUnitSelectRoot::on_decide_menu_item_method_info().into());
    let start = if GameVariableManager::get_number("G_拠点_裏武器イベント") >= 2 { 36 } else { 41 };
    let playable_count = PersonPool::get().playable.len();
    let mut count = 0;
    for i in start..playable_count {
        if let Some(replacement) = PersonPool::get_replacement(i as i32, false, false){
            if UnitPool::get_from_person(replacement, false).is_null() {
                let unit = Force::get(Force_Type::empty()).m_head();
                unit.create(replacement, JobData::null(), 0, Random_2::get_game());
                unit.transfer(Force_Type::temporary(), true);
                let item = RecruitmentMenuItem::instantiate().unwrap();
                IShopUnitSelectMenuItemMethods::ctor(item, unit, decided, select);
                menu_item_list.add(item);
                count += 1;
            }
        }
    }
    if count > 0 {
        unsafe { IBasicMenuMethods::ctor(menu, menu_item_list.cast::<List_1<BasicMenuItem>>(), menu_content.cast::<BasicMenuContent>()); }
        menu.set_m_show_god_ring_icon(false);
        menu.set_m_game_object(game_object);
        let descs = menu.create_default_desc();
        menu.create_bind(proc, descs, Il2CppString::null());
        menu.set_show_row_num(max -1);
        let status = this.m_unit_status_object().get_component::<ShopUnitSelectStatus>();
        this.set_m_shop_unit_select_status(status);
        unsafe { open_status(status, None); }
        UnitInfo::push_mode_model_only();
        UnitInfo::set_visible(true);
    }
    else { GameMessage::create_key_wait(proc,"There are no units to recruit."); }
}

#[unity::callback]
fn unit_recruit_yes(this: RecruitmentMenuItem, _: OptionalMethod) {
    let unit = this.get_unit();
    if !unit.is_null() {
        let menu = this.m_menu();
        unit.transfer(Force_Type::absent(), true);
        Mess::set_argument_2(0, unit.get_name());
        GameMessage::create_key_wait(menu, Mess::get("MID_MSG_JOIN_UNIT"));
        GameSound::join_unit();
        if menu.m_full_menu_item_list().get_count() > 1 {
            unsafe { menu.m_full_menu_item_list().remove(this.cast()); }
            menu.rebuild_instant(true);
        }
        else { menu.close(); }
    }
}
#[skyline::from_offset(0x247a540)] fn open_status(this: ShopUnitSelectStatus, optional_method: OptionalMethod);
#[skyline::from_offset(0x247ab20)] fn close_status(this: ShopUnitSelectStatus, optional_method: OptionalMethod);
