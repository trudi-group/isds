use super::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub entity: Option<Entity>,
    #[prop_or(true)]
    pub highlight_on_hover: bool,
}

#[function_component(EntityName)]
pub fn entity_name(props: &Props) -> Html {
    let isds_context = get_isds_context!();
    let sim = isds_context.sim.clone();
    let hl = isds_context.highlight;

    let &Props {
        entity,
        highlight_on_hover,
        ..
    } = props;

    let on_mouse_over = {
        if highlight_on_hover && entity.is_some() {
            let hl = hl.clone();
            Callback::from(move |_| hl.set_highlight(entity.unwrap()))
        } else {
            Callback::noop()
        }
    };

    let on_mouse_out = {
        if highlight_on_hover {
            let hl = hl.clone();
            Callback::from(move |_| hl.reset_highlight())
        } else {
            Callback::noop()
        }
    };

    html! {
        <span
            class={
                classes!(
                    highlight_on_hover.then_some("is-unselectable"),
                    entity.map_or(false, |e| hl.is(e)).then_some("has-text-info"),
                    props.class.clone())
            }
            onmouseover={ on_mouse_over }
            onmouseout={ on_mouse_out }
        >
            { entity.map_or("NONE".to_string(), |id| sim.borrow().name(id)) }
        </span>
    }
}
