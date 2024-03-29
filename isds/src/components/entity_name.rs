use super::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub entity: Option<Entity>,
    #[prop_or(true)]
    pub highlight_on_hover: bool,
    #[prop_or(true)]
    pub highlight_on_click: bool,
}

#[function_component(EntityName)]
pub fn entity_name(props: &Props) -> Html {
    let isds_context = get_isds_context!();
    let sim = isds_context.sim.clone();
    let hl = isds_context.highlight;

    let &Props {
        entity,
        highlight_on_hover,
        highlight_on_click,
        ..
    } = props;

    let (on_mouse_over, on_mouse_out) = if highlight_on_hover && entity.is_some() {
        (
            hl.set_hover_callback(entity.unwrap()),
            hl.reset_hover_callback(),
        )
    } else {
        (Callback::noop(), Callback::noop())
    };

    let on_click = {
        if highlight_on_click && entity.is_some() {
            hl.toggle_select_callback(entity.unwrap())
        } else {
            Callback::noop()
        }
    };

    html! {
        <span
            class={
                classes!(
                    highlight_on_click.then_some("is-clickable"),
                    entity.map_or(false, |e| hl.is(e)).then_some("has-text-info"),
                    props.class.clone())
            }
            onmouseover={ on_mouse_over }
            onmouseout={ on_mouse_out }
            onclick={ on_click }
        >
            { entity.map_or("NONE".to_string(), |id| sim.borrow().name(id)) }
        </span>
    }
}
