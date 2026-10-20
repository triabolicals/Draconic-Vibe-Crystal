use engage::app::{
    BasicMenuItem, BasicMenuItem_Attribute, Gender,
    sortieselectionunitmanager::*,
    sortieunitselect::*,
    IBitField32, IGameUserDataMethods, ISingletonClass_1Methods, IUnitMethods,
    UnitPool,
};
use unity::{Cast, OptionalMethod};
use crate::{DVCVariable, DVCVariables};
use crate::utils::override_vtable;

pub fn sortie_deployment_menu_install() {
    override_vtable("App", "SortieUnitSelect.UnitMenuItem", "BuildAttribute", sortie_unit_select_build_attr as _);
    override_vtable("App", "SortieTopMenu.SelectionUnitMenuItem", "GetMapAttribute", sortie_menu_selection_unit_map_attr as _);
    override_vtable("App", "SortieTopMenu.GodMenuItem", "GetMapAttribute", sortie_menu_god_menu_map_attr as _);
}

pub fn sortie_menu_god_menu_map_attr(_: BasicMenuItem, _: OptionalMethod) -> BasicMenuItem_Attribute {
    let v = DVCVariable::EmblemDeployment.get_value();
    if v == 1 || v == 3 { BasicMenuItem_Attribute::hide() }
    else if UnitPool::get_count(9u32) == 0 { BasicMenuItem_Attribute::disable() }
    else { BasicMenuItem_Attribute::enable()  }
}

pub fn sortie_menu_selection_unit_map_attr(_: BasicMenuItem, _: OptionalMethod) -> BasicMenuItem_Attribute {
    if UnitPool::get_count(9u32) == 0 { BasicMenuItem_Attribute::disable() }
    else {
        let unit_deployment_mode = DVCVariable::UnitDeployment.get_value();
        match unit_deployment_mode {
            1|2|5 => BasicMenuItem_Attribute::hide(),
            3|6|7 => BasicMenuItem_Attribute::enable(),
            _ => {
                let status = DVCVariables::user_data().get_status().m_value();
                if status & 12352 == 64 && DVCVariable::Continuous.get_value() == 0 { BasicMenuItem_Attribute::disable() }
                else { BasicMenuItem_Attribute::enable() }
            }
        }
    }
}

pub fn sortie_unit_select_build_attr(this: SortieUnitSelect_UnitMenuItem, _: OptionalMethod) -> BasicMenuItem_Attribute {
    let mode = DVCVariable::UnitDeployment.get_value();
    if SortieSelectionUnitManager::get_instance().is_mode_sortie() && mode > 5 {
        let gender = if mode == 7 { Gender::female() } else { Gender::male() };
        let unit = this.m_unit();
        if !unit.is_null() { if unit.get_gender() != gender { return BasicMenuItem_Attribute::hide(); } }
    }
    unsafe { unit_menu_item_build_attr(this, None) }
}
#[skyline::from_offset(0x1d7ae30)]
fn unit_menu_item_build_attr(this: SortieUnitSelect_UnitMenuItem, method: OptionalMethod) -> BasicMenuItem_Attribute;