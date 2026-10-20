pub use engage::{
    system::{
        Action_1, IAction_1, IAction_1Methods,
        Action, IAction, IActionMethods,
        collections::generic::{List_1, IList_1Methods, }
    },
    List_1Ext,
    app::{
        Pad, IPadMethods, ISingletonClass_1Methods,
        basicmenu::{*, BasicMenu, IBasicMenuMethods, IBasicMenu, BasicMenu_Result},
        BasicMenuSelect, IBasicMenuSelectMethods,
        BasicMenuItem_Attribute, BasicMenuItem, IBasicMenuItemMethods, IBasicMenuItem,
        IBasicMenuItemContentMethods,
        BasicMenuContent, IBasicMenuContentMethods,
        ProcInst, IProcInstMethods, IProcInst,
    },
    nn::hid::NpadButton,
    unity_engine::IGameObjectMethods,
};
pub use crate::config::{DVCConfig, DVCVariable, DVCVariables, DVCFlags};
pub use unity::{prelude::*, Array};

mod config;
pub mod dialog;
mod unitselect;

pub use config::{CONFIG_TEXT, DVCConfigText, RecruitmentOrder, DVCSettingMenu, DVCBasicMenuItem};
#[cfg(feature = "debug")] pub use config::DVCDebug;
pub use unitselect::UnitRecruitMenuItem;
use crate::menus::unitselect::{RecruitmentMenuItem, UnitRecruitProc};

pub fn inject_menu_classes() {
    if cobapi::injection::register::<DVCBasicMenuItem>().is_ok() { println!("DVCBasicMenuItem is Injected"); }
    if cobapi::injection::register::<DVCSettingMenu>().is_ok()  { println!("DVCMenu is Injected"); }
    if cobapi::injection::register::<UnitRecruitMenuItem>().is_ok() { println!("UnitRecruitMenuItem is Injected"); }
    if cobapi::injection::register::<RecruitmentMenuItem>().is_ok() { println!("RecruitmentMenuItem is Injected"); }
    if cobapi::injection::register::<UnitRecruitProc>().is_ok() { println!("UnitRecruitProc is Injected"); }
}
