use super::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub title: String,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(LayerDescription)]
pub fn layer_description(props: &Props) -> Html {
    let is_collapsed = use_state(|| true);
    let on_title_click = {
        let is_collapsed = is_collapsed.clone();
        Callback::from(move |_| is_collapsed.set(!*is_collapsed))
    };
    html! {
        <>
            <div class="level is-mobile is-clickable is-unselectable" onclick={ on_title_click }>
                <div class="level-left">
                    <h2 class="title is-4">{ &props.title }</h2>
                </div>
                <div class="level-right">
                    <span class="icon">
                        <i class={
                            classes!("fas", if *is_collapsed { "fa-plus" } else { "fa-minus" })
                        }></i>
                    </span>
                </div>
            </div>
            if !*is_collapsed {
                { for props.children.iter() }
            }
        </>
    }
}
