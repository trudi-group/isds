use super::*;

#[function_component(Beyond)]
pub fn beyond() -> Html {
    html! {
        <>
            <Header title={ "More places to learn" }>
                { include_markdown_content!("intro.md") }
            </Header>
            <Section>
                { include_markdown_content!("content.md") }
            </Section>
            <Footer />
        </>
    }
}
